use super::audio_utils::AudioUtils;
use super::text_utils::{CleanedText, TextUtils, CHINESE_LANG};
use ndarray::{s, Array1, Array2, Array3, Array4, Axis};
use ort::{inputs, session::Session};
use std::cmp::Ordering;
use std::f32::consts::PI;
use std::path::Path;
use std::time::Instant;
use tokenizers::Tokenizer;

struct ModelSessions {
    pub bert_model: Session,
    pub ssl_model: Session,
    pub vq_model_latent: Session,
    pub t2s_first_stage_decoder: Session,
    pub t2s_stage_decoder: Session,
    pub vq_model: Session,
}

impl ModelSessions {
    fn load_model(model_path: &str) -> ort::Result<Session> {
        Session::builder()?.commit_from_file(model_path)
    }

    pub fn from_file(
        bert_model_path: &str,
        ssl_model_path: &str,
        vq_model_latent_path: &str,
        t2s_first_stage_decoder_path: &str,
        t2s_stage_decoder_path: &str,
        vq_model_path: &str,
    ) -> Self {
        Self {
            bert_model: Self::load_model(bert_model_path).expect("Failed to load bert_model"),
            ssl_model: Self::load_model(ssl_model_path).expect("Failed to load ssl_model"),
            vq_model_latent: Self::load_model(vq_model_latent_path)
                .expect("Failed to load vq_model_latent"),
            t2s_first_stage_decoder: Self::load_model(t2s_first_stage_decoder_path)
                .expect("Failed to load t2s_first_stage_decoder"),
            t2s_stage_decoder: Self::load_model(t2s_stage_decoder_path)
                .expect("Failed to load t2s_stage_decoder"),
            vq_model: Self::load_model(vq_model_path).expect("Failed to load vq_model"),
        }
    }

    // 添加其他方法来操作这些 Session 对象
}

#[derive(Debug, Clone)]
pub struct BertFeatures {
    pub features: Array2<f32>,
    pub phones_list_unpack: Vec<usize>,
    pub norm_text_str: String,
}

pub struct ChBertUtils {
    pub tokenizer: Tokenizer,
}
pub fn hanning(m: i64) -> Array1<f32> {
    match m.cmp(&1) {
        Ordering::Less => Array1::from_vec(vec![]),
        Ordering::Equal => Array1::ones(1),
        Ordering::Greater => {
            let n: Vec<f32> = (0..m)
                .map(|x| {
                    let v1 = 2.0 * PI * (x as f32) / (m - 1) as f32;
                    0.5 - 0.5 * v1.cos()
                })
                .collect();
            Array1::from_vec(n)
        }
    }
}

impl ChBertUtils {
    pub fn new(tokenizer_json_path: &str) -> Self {
        Self {
            tokenizer: Tokenizer::from_file(tokenizer_json_path).unwrap(),
        }
    }

    // 返回最终的混合中英文句子features
    pub fn get_bert_features(
        tokenizer: &Tokenizer,
        bert_model: &Session,
        phones_list: &mut [Vec<usize>],
        word2ph_list: &[Vec<usize>],
        norm_text_list: &[String],
        language_list: &[String],
    ) -> BertFeatures {
        let mut features = Vec::new();
        let mut phones_list_unpack = Vec::new();
        let mut norm_text_str = String::new();
        for i in 0..language_list.len() {
            let phones_len = phones_list[i].len();
            let word2ph = &word2ph_list[i];

            norm_text_str += &norm_text_list[i];
            phones_list_unpack.append(&mut phones_list[i]);

            let feature = if language_list[i] == CHINESE_LANG {
                let encoding = tokenizer
                    .encode(norm_text_list[i].as_str(), true)
                    .expect("Failed to encode text");

                let input_ids: Array2<i64> = ndarray::Array1::from_vec(encoding.get_ids().to_vec())
                    .insert_axis(Axis(0))
                    .mapv(|x| x as i64);
                let attention_mask: Array2<i64> =
                    ndarray::Array1::from_vec(encoding.get_attention_mask().to_vec())
                        .insert_axis(Axis(0))
                        .mapv(|x| x as i64);
                let token_type_ids: Array2<i64> =
                    ndarray::Array1::from_vec(encoding.get_type_ids().to_vec())
                        .insert_axis(Axis(0))
                        .mapv(|x| x as i64);

                let input_tensor_value = inputs![input_ids, attention_mask, token_type_ids]
                    .expect("Failed to create input tensor");
                let generator_source = bert_model
                    .run(input_tensor_value)
                    .expect("Failed to run bert model");

                let hidden_states = generator_source["hidden_states"]
                    .try_extract_tensor()
                    .unwrap();
                // [1, 32, 1024] -> [0,1:-1,:]
                let hidden_states: Array2<f32> =
                    hidden_states.view().slice(s![0, 1.., ..]).to_owned();

                let phone_level_feature = word2ph
                    .iter()
                    .enumerate()
                    .map(|(i, &w2)| {
                        let repeat_features: Vec<_> = (0..w2)
                            .map(|_| hidden_states.slice(s![i, ..;1]).to_owned())
                            .collect();

                        ndarray::stack(
                            Axis(0),
                            &repeat_features.iter().map(|v| v.view()).collect::<Vec<_>>(),
                        )
                        .unwrap()
                    })
                    .collect::<Vec<_>>();

                let phone_level_feature = ndarray::concatenate(
                    Axis(0),
                    &phone_level_feature
                        .iter()
                        .map(|v| v.view())
                        .collect::<Vec<_>>(),
                )
                .unwrap();
                ndarray::ArrayBase::t(&phone_level_feature).to_owned()
            } else {
                Array2::zeros((1024, phones_len))
            };
            features.push(feature);
        }
        let features: Array2<f32> = ndarray::concatenate(
            Axis(1),
            &features.iter().map(|v| v.view()).collect::<Vec<_>>(),
        )
        .expect("Failed to concatenate features");
        BertFeatures {
            features,
            phones_list_unpack,
            norm_text_str,
        }
    }
}

fn infer_wav(
    sessions: &ModelSessions,
    wav16k_arr: &Array2<f32>,
    wav32k_arr: &Array2<f32>,
    bert_features1: &Array2<f32>,
    bert_features2: &Array2<f32>,
    phones_list_unpack1: &[usize],
    phones_list_unpack2: &[usize],
) -> Vec<i16> {
    //float32[batch_sie:1, W:113104]
    let input_wav16k = inputs![wav16k_arr.clone()].expect("Failed to create input_wav16k input");
    let ssl_content = sessions
        .ssl_model
        .run(input_wav16k)
        .expect("Failed to run ssl_model");
    let ssl_content = ssl_content["output"]
        .try_extract_tensor::<f32>()
        .expect("Failed to extract ssl_content tensor");
    let hop_length = 640;
    let win_length = 2048;
    let hann_window = hanning(win_length);

    // float32[batch_size:1, 768, H:383]
    let ssl_content: Array3<f32> = ssl_content.view().slice(s![.., .., ..]).to_owned();

    let input_ssl_content = inputs![ssl_content].expect("Failed to create input_ssl_content input");
    let codes = sessions
        .vq_model_latent
        .run(input_ssl_content)
        .expect("Failed to run vq_model_latent");
    let codes = codes["output"]
        .try_extract_tensor::<i64>()
        .expect("Failed to extract codes tensor");
    //[1, 191]
    let prompt: Array2<i64> = codes.view().slice(s![0, .., ..]).to_owned();

    let top_k: Array1<i64> = ndarray::Array1::from(vec![20]);
    let temperature: Array1<f32> = ndarray::Array1::from(vec![0.8]);
    //  合并参考的声音
    let bert: Array3<f32> =
        ndarray::concatenate(Axis(1), &[bert_features1.view(), bert_features2.view()])
            .expect("Failed to concatenate bert features")
            .insert_axis(Axis(0));

    // 会清空
    let all_phoneme_ids = Array1::from_vec({
        let mut combined = phones_list_unpack1.to_owned();
        combined.extend_from_slice(phones_list_unpack2);
        combined
    })
    .insert_axis(Axis(0))
    .mapv(|x| x as i64);

    let text = Array1::from_vec(phones_list_unpack2.to_owned())
        .insert_axis(Axis(0))
        .mapv(|x| x as i64);

    let x_example: Array2<f32> =
        Array2::zeros((all_phoneme_ids.shape()[0], all_phoneme_ids.shape()[1]));

    let start_loop = Instant::now();

    let first_stage_decoder_input = inputs![
        "all_phoneme_ids" => all_phoneme_ids.view(),
        "bert" => bert.view(),
        "prompt" => prompt.view(),
        "top_k" => top_k.view(),
        "temperature" => temperature.view(),
    ]
    .expect("Failed to create first_stage_decoder_input");
    let start_loop1 = Instant::now();
    let t2s_first_stage_out = sessions
        .t2s_first_stage_decoder
        .run(first_stage_decoder_input)
        .expect("Failed to run t2s_first_stage_decoder");
    println!(
        "t2s_first_stage time: {}ms",
        start_loop1.elapsed().as_millis()
    );

    let mut y: Array2<i64> = t2s_first_stage_out["y"]
        .try_extract_tensor::<i64>()
        .unwrap()
        .view()
        .slice(s![.., ..])
        .into_owned();
    let mut k: Array4<f32> = t2s_first_stage_out["k"]
        .try_extract_tensor::<f32>()
        .unwrap()
        .view()
        .slice(s![.., .., .., ..])
        .into_owned();
    let mut v: Array4<f32> = t2s_first_stage_out["v"]
        .try_extract_tensor::<f32>()
        .unwrap()
        .view()
        .slice(s![.., .., .., ..])
        .into_owned();
    let mut y_emb: Array3<f32> = t2s_first_stage_out["y_emb"]
        .try_extract_tensor::<f32>()
        .unwrap()
        .view()
        .slice(s![.., .., ..])
        .into_owned();

    let mut y_example: Array2<f32> = Array2::zeros((1, y_emb.shape()[1]));
    let y_example_0: Array2<f32> = Array2::zeros((1, 1));

    let mut loop_idx = 0;
    for idx in 1..1500 {
        y_example = ndarray::concatenate(Axis(1), &[y_example.view(), y_example_0.view()])
            .expect("Failed to concatenate y_example");
        let xy_attn_mask: Array4<f32> =
            ndarray::concatenate(Axis(1), &[x_example.view(), y_example.view()])
                .unwrap()
                .insert_axis(Axis(0))
                .insert_axis(Axis(0));

        let t2s_stage_decoder_input = inputs![
        "y" => y.view(),
        "k" => k.view(),
        "v" => v.view(),
        "y_emb" => y_emb.view(),
        "xy_attn_mask" => xy_attn_mask.view(),
        "top_k" => top_k.view(),
        "temperature" => temperature.view(),
        ]
        .expect("Failed to create t2s_stage_decoder_input");

        let start_loop_t2s_stage_decoder = Instant::now();
        let t2s_stage_decoder_out = sessions
            .t2s_stage_decoder
            .run(t2s_stage_decoder_input)
            .expect("Failed to run t2s_stage_decoder");
        println!(
            "stage_decoder: {}ms",
            start_loop_t2s_stage_decoder.elapsed().as_millis()
        );

        k = t2s_stage_decoder_out["o_k"]
            .try_extract_tensor::<f32>()
            .unwrap()
            .view()
            .slice(s![.., .., .., ..])
            .into_owned();
        v = t2s_stage_decoder_out["o_v"]
            .try_extract_tensor::<f32>()
            .unwrap()
            .view()
            .slice(s![.., .., .., ..])
            .into_owned();
        y_emb = t2s_stage_decoder_out["o_y_emb"]
            .try_extract_tensor::<f32>()
            .unwrap()
            .view()
            .slice(s![.., .., ..])
            .into_owned();
        let logits: Array1<i64> = t2s_stage_decoder_out["logits"]
            .try_extract_tensor::<i64>()
            .unwrap()
            .view()
            .slice(s![..])
            .into_owned();
        let samples: Array2<i64> = t2s_stage_decoder_out["samples"]
            .try_extract_tensor::<i64>()
            .unwrap()
            .view()
            .slice(s![.., ..])
            .into_owned();

        y = ndarray::concatenate(Axis(1), &[y.view(), samples.view()])
            .expect("Failed to concatenate y");

        if *samples.get((0, 0)).expect("Failed to get sample") == 1024
            || *logits.get(0).expect("Failed to get logit") == 1024
        {
            loop_idx = idx;
            break;
        }
    }
    println!(
        "{}ms , loop_idx:{}",
        start_loop.elapsed().as_millis(),
        loop_idx
    );

    *y.get_mut((0, y.shape()[1] - 1)).unwrap() = 0;

    let pred_semantic: Array3<i64> = y
        .slice(s![.., y.shape()[1] - loop_idx..])
        .into_owned()
        .insert_axis(Axis(0));

    let y_len = (pred_semantic.shape()[2] * 2) as i64;
    let y_lengths: Array1<i64> = ndarray::Array1::from(vec![y_len]);
    let text_lengths: Array1<i64> = ndarray::Array1::from(vec![text.shape()[0] as i64]);
    let t = (wav32k_arr.shape()[1] - hop_length) / hop_length + 1;
    let refer_mask: Array3<i64> =
        Array3::ones((pred_semantic.shape()[0], pred_semantic.shape()[1], t));

    let vq_model_input = inputs![
        "pred_semantic"=>pred_semantic.view(),
        "text"=>text.view(),
        "org_audio"=>wav32k_arr.view(),
        "hann_window"=>hann_window.view(),
        "refer_mask"=>refer_mask.view(),
        "y_lengths"=>y_lengths.view(),
        "text_lengths"=>text_lengths.view(),
    ]
    .expect("Failed to create vq_model_input");
    let start_vq_model = Instant::now();
    let vq_model_out = sessions
        .vq_model
        .run(vq_model_input)
        .expect("Failed to run vq_model");
    let start_vq_model2 = Instant::now();

    let audio: Array1<f32> = vq_model_out["audio"]
        .try_extract_tensor::<f32>()
        .unwrap()
        .view()
        .slice(s![0, 0, ..])
        .into_owned();
    println!(
        "time:{}, audio:{:?}",
        (start_vq_model2 - start_vq_model).as_millis(),
        audio.shape()
    );

    let audio: Vec<f32> = audio.to_vec();
    let max_audio = audio.iter().map(|&v| v.abs()).fold(0.0, f32::max);
    let audio_norm = if max_audio > 1.0 {
        audio
            .iter()
            .map(|&x| ((x / max_audio) * 32768.0) as i16)
            .collect()
    } else {
        audio.iter().map(|&x| (x * 32768.0) as i16).collect()
    };
    // 保存结果
    // AudioUtils::decode_data_to_path(&audio_norm, "./make_32k.wav", 32000, true).unwrap();
    audio_norm
}

pub fn infer(text: &str) -> Vec<i16> {
    let tokenizer_path = Path::new("../assets/tokenizer.json");

    let ch_bert_util = ChBertUtils::new(tokenizer_path.to_str().unwrap());

    let sampling_rate: i32 = 32000;

    let zero_sampling_len = (sampling_rate as f32 * 0.3) as usize;
    let zero_wav: Array1<f32> = Array1::zeros((zero_sampling_len,));
    println!("zero_wav:{:?}", zero_wav.shape());

    let start = Instant::now();

    // 参考音色音频文件
    let ref_wav_path = "../assets/tts_16_3.wav";
    // 参考音色音频对应的文字
    let prompt_text =
        "今天天气不错，我准备去打篮球。I am going to play basketball today. 我的房间号是 404，希望一切顺利。".to_string();

    let wav16k: Vec<i16> = AudioUtils::decode_path_to_data(ref_wav_path, 16000).unwrap();
    let wav32k: Vec<i16> = AudioUtils::decode_path_to_data(ref_wav_path, 32000).unwrap();

    let wav16k: Vec<f32> = wav16k.iter().map(|&x| x as f32 / 32768.0).collect();
    let wav32k: Vec<f32> = wav32k.iter().map(|&x| x as f32 / 32768.0).collect();

    println!(
        "time_t:{} ms ,16k len:{}, 32k len:{}",
        start.elapsed().as_millis(),
        wav16k.len(),
        wav32k.len()
    );

    let wav16k_arr =
        ndarray::concatenate(Axis(0), &[Array1::from_vec(wav16k).view(), zero_wav.view()])
            .expect("Failed to concatenate wav16k_arr")
            .insert_axis(Axis(0));
    let wav32k_arr = Array1::from_vec(wav32k).insert_axis(Axis(0));
    println!("wav16k_arr:{:?} ", wav16k_arr.shape());

    // let text = "每个人的理想不一样，扎出来的风筝也不一样。所有的风筝中，要数小音乐家根子的最棒了，那是一架竖琴。让她到天上去好好想想吧！哈，风筝的后脑勺上还拖着一条马尾巴似的长辫子！在地面上，我们一边放线一边跑着，手里的线越放越长，风筝也带着我们的理想越飞越远，越飞越高如果把眼前的一池荷花看作一大幅活的画，那画家的本领可真了不起。";
    // let text = "Hello! Today is January 15th, 2025, and the time is 3:45 PM. The temperature is 22.5℃, and it feels like 20℃. You owe me $12.34, or £9.99, which you can pay by 6:00 AM tomorrow. Can you read this email address: test@example.com? What about this URL: https://www.openai.com? Finally, here's a math equation: 3.14 × 2 = 6.28, and a phone number: (123) 456-7890.";

    let text_util = TextUtils::new(
        "../assets/eng_dict.json",
        "../assets/rep_map.json",
        "../assets/model.npz",
        "../assets/PHRASES_DICT.json",
        "../assets/PINYIN_DICT.json",
    )
    .expect("Failed to create text_util");

    let texts = text_util
        .lang_seg
        .cut_texts(text, prompt_text.chars().count());

    println!("texts:{}", texts.join("\n"));

    let start = Instant::now();
    let CleanedText {
        mut phones_list,
        word2ph_list,
        lang_list,
        norm_text_list,
    } = text_util.get_cleaned_text_final(&prompt_text);
    println!("time_t2:{} ms", start.elapsed().as_millis());

    let model_sessons = ModelSessions::from_file(
        "../assets/bert_model.onnx",
        "../assets/ssl_model.onnx",
        "../assets/vq_model_latent.onnx",
        "../assets/t2s_first_stage_decoder.onnx",
        "../assets/t2s_stage_decoder.onnx",
        "../assets/vq_model.onnx",
    );

    let start = Instant::now();
    let BertFeatures {
        features,
        phones_list_unpack,
        norm_text_str,
    } = ChBertUtils::get_bert_features(
        &ch_bert_util.tokenizer,
        &model_sessons.bert_model,
        &mut phones_list,
        &word2ph_list,
        &norm_text_list,
        &lang_list,
    );
    println!(
        "norm_text_str1:{},phones_list_unpack1:{},time_t3:{} ms",
        norm_text_str,
        phones_list_unpack.len(),
        start.elapsed().as_millis()
    );

    println!("bert_features1.shape:{:?}", features.shape());

    texts
        .iter()
        .flat_map(|t| {
            let CleanedText {
                mut phones_list,
                word2ph_list,
                lang_list,
                norm_text_list,
            } = text_util.get_cleaned_text_final(t);
            let BertFeatures {
                features: _features,
                phones_list_unpack: _phones_list_unpack,
                norm_text_str: _norm_text_str,
            } = ChBertUtils::get_bert_features(
                &ch_bert_util.tokenizer,
                &model_sessons.bert_model,
                &mut phones_list,
                &word2ph_list,
                &norm_text_list,
                &lang_list,
            );

            println!("_phones_list_unpack:{:?}", _phones_list_unpack);
            println!("text:{} ->{}", text, _norm_text_str);

            infer_wav(
                &model_sessons,
                &wav16k_arr,
                &wav32k_arr,
                &features,
                &_features,
                &phones_list_unpack,
                &_phones_list_unpack,
            )
        })
        .collect()
}
