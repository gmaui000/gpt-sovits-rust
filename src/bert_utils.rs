use super::audio_utils::AudioUtils;
use super::text_utils::{CleanedText, TextUtils, CHINESE_LANG};
use ndarray::{s, Array1, Array2, Array3, Array4, ArrayView1, ArrayView2, Axis};
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
        let bert_model = Self::load_model(bert_model_path).unwrap();
        let ssl_model = Self::load_model(ssl_model_path).unwrap();
        let vq_model_latent = Self::load_model(vq_model_latent_path).unwrap();
        let t2s_first_stage_decoder = Self::load_model(t2s_first_stage_decoder_path).unwrap();
        let t2s_stage_decoder = Self::load_model(t2s_stage_decoder_path).unwrap();
        let vq_model = Self::load_model(vq_model_path).unwrap();
        Self {
            bert_model,
            ssl_model,
            vq_model_latent,
            t2s_first_stage_decoder,
            t2s_stage_decoder,
            vq_model,
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
    pub fn init(tokenizer_json_path: &str) -> Self {
        let tokenizer = Tokenizer::from_file(tokenizer_json_path).unwrap();
        ChBertUtils { tokenizer }
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
        let mut features = vec![];
        let mut phones_list_unpack = vec![];
        let mut norm_text_str = "".to_string();
        for i in 0..language_list.len() {
            let phones_len = phones_list[i].len();
            let word2ph = &word2ph_list[i];

            norm_text_str = norm_text_str + &norm_text_list[i];
            phones_list_unpack.append(&mut phones_list[i]);

            if language_list[i] == CHINESE_LANG {
                let encoding_opt = tokenizer.encode(norm_text_list[i].as_str(), true);
                if encoding_opt.is_err() {
                    continue;
                }
                let encoding = encoding_opt.unwrap();

                let input_ids = encoding.get_ids();
                let attention_mask = encoding.get_attention_mask();
                let token_type_ids = encoding.get_type_ids();

                let input_ids: Array2<i64> = ndarray::Array1::from_vec(input_ids.to_vec())
                    .insert_axis(Axis(0))
                    .mapv(|x| x as i64);
                let attention_mask: Array2<i64> =
                    ndarray::Array1::from_vec(attention_mask.to_vec())
                        .insert_axis(Axis(0))
                        .mapv(|x| x as i64);
                let token_type_ids: Array2<i64> =
                    ndarray::Array1::from_vec(token_type_ids.to_vec())
                        .insert_axis(Axis(0))
                        .mapv(|x| x as i64);

                let input_tensor_value =
                    inputs![input_ids, attention_mask, token_type_ids].unwrap();
                let generator_source = bert_model.run(input_tensor_value);
                let generator_source = generator_source.unwrap();

                let hidden_states = generator_source["hidden_states"]
                    .try_extract_tensor()
                    .unwrap();
                // [1, 32, 1024] -> [0,1:-1,:]
                let hidden_states: Array2<f32> =
                    hidden_states.view().slice(s![0, 1.., ..]).to_owned();

                let mut phone_level_feature = vec![];
                for (i, &w2) in word2ph.iter().enumerate() {
                    let mut repeat_features = vec![];
                    for _ in 0..w2 {
                        let repeat_feature_i: Array1<f32> =
                            hidden_states.slice(s![i,..;1]).to_owned();
                        repeat_features.push(repeat_feature_i);
                    }

                    let repeat_features_view: Vec<ArrayView1<f32>> =
                        repeat_features.iter().map(|v| v.view()).collect();

                    let repeat_feature: Array2<f32> =
                        ndarray::stack(Axis(0), &repeat_features_view).unwrap();
                    phone_level_feature.push(repeat_feature);
                }

                let phone_level_feature: Vec<ArrayView2<f32>> =
                    phone_level_feature.iter().map(|v| v.view()).collect();

                let phone_level_feature =
                    ndarray::concatenate(Axis(0), &phone_level_feature).unwrap();
                let phone_level_feature_t: Array2<f32> =
                    ndarray::ArrayBase::t(&phone_level_feature).to_owned();
                features.push(phone_level_feature_t);
            } else {
                let bert: Array2<f32> = Array2::zeros((1024, phones_len));
                features.push(bert);
            }
        }
        let bert_features_view: Vec<ArrayView2<f32>> = features.iter().map(|v| v.view()).collect();
        let features: Array2<f32> = ndarray::concatenate(Axis(1), &bert_features_view).unwrap();

        BertFeatures {
            features,
            phones_list_unpack,
            norm_text_str,
        }
    }
}

fn test_infer_wav(
    sessions: &ModelSessions,
    wav16k_arr: &Array2<f32>,
    wav32k_arr: &Array2<f32>,
    bert_features1: &Array2<f32>,
    bert_features2: &Array2<f32>,
    phones_list_unpack1: &[usize],
    phones_list_unpack2: &[usize],
) {
    //float32[batch_sie:1, W:113104]
    let input_wav16k = inputs![wav16k_arr.clone()].unwrap();
    let ssl_content = sessions.ssl_model.run(input_wav16k);
    let ssl_content = ssl_content.unwrap();
    let ssl_content = ssl_content["output"].try_extract_tensor::<f32>().unwrap();
    let hop_length = 640;
    let win_length = 2048;
    let hann_window = hanning(win_length);

    // float32[batch_size:1, 768, H:383]
    let ssl_content: Array3<f32> = ssl_content.view().slice(s![.., .., ..]).to_owned();

    let input_ssl_content = inputs![ssl_content].unwrap();
    let codes = sessions.vq_model_latent.run(input_ssl_content);
    let codes = codes.unwrap();
    let codes = codes["output"].try_extract_tensor::<i64>().unwrap();
    //[1, 191]
    let prompt: Array2<i64> = codes.view().slice(s![0, .., ..]).to_owned();

    let top_k: Array1<i64> = ndarray::Array1::from(vec![20]);
    let temperature: Array1<f32> = ndarray::Array1::from(vec![0.8]);
    //  合并参考的声音
    let bert: Array3<f32> =
        ndarray::concatenate(Axis(1), &[bert_features1.view(), bert_features2.view()])
            .unwrap()
            .insert_axis(Axis(0));

    // 会清空
    let mut _phones_list_unpack1 = phones_list_unpack1.to_owned();
    let mut _phones_list_unpack2 = phones_list_unpack2.to_owned();
    _phones_list_unpack1.append(&mut _phones_list_unpack2);

    let all_phoneme_ids: Array2<i64> = Array1::from_vec(_phones_list_unpack1)
        .insert_axis(Axis(0))
        .mapv(|x| x as i64);
    let text: Array2<i64> = Array1::from_vec(phones_list_unpack2.to_owned())
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
    .unwrap();
    let start_loop1 = Instant::now();
    let t2s_first_stage_out = sessions
        .t2s_first_stage_decoder
        .run(first_stage_decoder_input);
    println!(
        "t2s_first_stage time: {}ms",
        start_loop1.elapsed().as_millis()
    );
    let t2s_first_stage_out = t2s_first_stage_out.unwrap();

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
        y_example = ndarray::concatenate(Axis(1), &[y_example.view(), y_example_0.view()]).unwrap();
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
        .unwrap();

        let start_loop_t2s_stage_decoder = Instant::now();
        let t2s_stage_decoder_out = sessions.t2s_stage_decoder.run(t2s_stage_decoder_input);
        println!(
            "stage_decoder: {}ms",
            start_loop_t2s_stage_decoder.elapsed().as_millis()
        );
        let t2s_stage_decoder_out = t2s_stage_decoder_out.unwrap();

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

        y = ndarray::concatenate(Axis(1), &[y.view(), samples.view()]).unwrap();

        let sample = samples.get((0, 0)).unwrap();
        let logit = logits.get(0).unwrap();

        if *logit == 1024 || *sample == 1024 {
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
    .unwrap();
    let start_vq_model = Instant::now();
    let vq_model_out = sessions.vq_model.run(vq_model_input);
    let vq_model_out = vq_model_out.unwrap();
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
    let max_audio = {
        let mut max_v = 0.0;
        for &v in &audio {
            let v = num::abs(v);
            if v > max_v {
                max_v = v;
            }
        }
        max_v
    };
    let audio_norm = {
        if max_audio > 1.0 {
            let v: Vec<i16> = audio
                .iter()
                .map(|&x| ((x / max_audio) * 32768.0) as i16)
                .collect();
            v
        } else {
            let v: Vec<i16> = audio.iter().map(|&x| (x * 32768.0) as i16).collect();
            v
        }
    };
    // 保存结果
    AudioUtils::decode_data_to_path(&audio_norm, "./make_32k.wav", 32000, true).unwrap();
}

pub fn infer() {
    let tokenizer_path = Path::new("../data/tokenizer.json");

    let ch_bert_util = ChBertUtils::init(tokenizer_path.to_str().unwrap());

    let sampling_rate: i32 = 32000;

    let zero_sampling_len = (sampling_rate as f32 * 0.3) as usize;
    let zero_wav: Array1<f32> = Array1::zeros((zero_sampling_len,));
    println!("zero_wav:{:?}", zero_wav.shape());

    let start = Instant::now();

    // 参考音色音频文件
    let ref_wav_path = "../data/tts_16_3.wav";
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

    let wav16k_arr: Array1<f32> = Array1::from_vec(wav16k);
    let wav32k_arr: Array1<f32> = Array1::from_vec(wav32k);

    let wav16k_sum = wav16k_arr.sum();
    let wav32k_sum = wav32k_arr.sum();

    println!("wav16k_sum:{},wav32k_sum:{}", wav16k_sum, wav32k_sum);

    let wav16k_arr: Array2<f32> =
        ndarray::concatenate(Axis(0), &[wav16k_arr.view(), zero_wav.view()])
            .unwrap()
            .insert_axis(Axis(0));
    let wav32k_arr: Array2<f32> = wav32k_arr.insert_axis(Axis(0));
    println!("wav16k_arr:{:?} ", wav16k_arr.shape());

    let text = "每个人的理想不一样，扎出来的风筝也不一样。所有的风筝中，要数小音乐家根子的最棒了，那是一架竖琴。让她到天上去好好想想吧！哈，风筝的后脑勺上还拖着一条马尾巴似的长辫子！在地面上，我们一边放线一边跑着，手里的线越放越长，风筝也带着我们的理想越飞越远，越飞越高如果把眼前的一池荷花看作一大幅活的画，那画家的本领可真了不起。";
    // let text = "Hello! Today is January 15th, 2025, and the time is 3:45 PM. The temperature is 22.5℃, and it feels like 20℃. You owe me $12.34, or £9.99, which you can pay by 6:00 AM tomorrow. Can you read this email address: test@example.com? What about this URL: https://www.openai.com? Finally, here's a math equation: 3.14 × 2 = 6.28, and a phone number: (123) 456-7890.";

    let text_util = TextUtils::init(
        "../data/eng_dict.json",
        "../data/rep_map.json",
        "../data/model.npz",
        "../data/PHRASES_DICT.json",
        "../data/PINYIN_DICT.json",
    )
    .unwrap();

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
        "../data/bert_model.onnx",
        "../data/ssl_model.onnx",
        "../data/vq_model_latent.onnx",
        "../data/t2s_first_stage_decoder.onnx",
        "../data/t2s_stage_decoder.onnx",
        "../data/vq_model.onnx",
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

    for text in texts {
        let CleanedText {
            mut phones_list,
            word2ph_list,
            lang_list,
            norm_text_list,
        } = text_util.get_cleaned_text_final(&text);
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

        test_infer_wav(
            &model_sessons,
            &wav16k_arr,
            &wav32k_arr,
            &features,
            &_features,
            &phones_list_unpack,
            &_phones_list_unpack,
        );
    }
}
