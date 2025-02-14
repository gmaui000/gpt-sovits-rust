use super::text::{self, symbols::SYMBOLS};
use lazy_static::lazy_static;
use lingua::Language::{Chinese, English};
use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};
use regex::{Captures, Regex};
use std::collections::HashMap;

pub(crate) const ENGLISH_LANG: &str = "English";
pub(crate) const CHINESE_LANG: &str = "Chinese";
pub(crate) const JAPANESE_LANG: &str = "Japanese";

lazy_static! {
    static ref PATTERN_ALPHA_RANGE: Regex =
        Regex::new(r"([a-zA-Z]+)([—\->～~])([a-zA-Z]+)").unwrap();
    static ref PATTERN_ALPHA_RANGE2: Regex = Regex::new(r"([a-zA-Z]+)([—\->～~])([0-9]+)").unwrap();
    static ref PATTERN_AZ: Regex = Regex::new(r"[a-zA-Z]+").unwrap();
    static ref PATTERN_2: Regex = Regex::new(r"[a-zA-Z0-9|.%]+").unwrap();
    static ref PATTERN_ZH: Regex = Regex::new(r"[\u4e00-\u9fa5]+").unwrap();
}

pub struct LangSegment {
    pub _splits: Vec<String>,
    pub detector: LanguageDetector,
}

pub struct TextUtils {
    pub lang_seg: LangSegment,
    pub lang_chinese: text::chinese::Chinese,
    pub lang_english: text::english::English,
    pub symbol_to_id: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct CleanedText {
    pub phones_list: Vec<Vec<usize>>,
    pub word2ph_list: Vec<Vec<usize>>,
    pub lang_list: Vec<String>,
    pub norm_text_list: Vec<String>,
}

/// 语言分割
impl LangSegment {
    pub fn new(languages: Vec<Language>) -> Self {
        let detector: LanguageDetector =
            LanguageDetectorBuilder::from_languages(&languages).build();
        let _splits: Vec<String> = vec![
            "，", "。", "？", "！", ",", ".", "?", "!", "~", ":", "：", "—", "…",
        ]
        .iter()
        .map(|&s| s.to_string())
        .collect();

        LangSegment { _splits, detector }
    }

    /// "包含AC到BZ" -> 被识别为中文，需要拆分为 中文（包含）、英文(A C)、中文（到）、英文(B Z)
    fn zh_en_seg(&self, sentence: &str, lang: &str) -> Vec<(String, String)> {
        // 一个字母都没有：直接返回原始
        // 一个中文都没有
        if !PATTERN_AZ.is_match(sentence) || !PATTERN_ZH.is_match(sentence) {
            vec![(lang.to_string(), sentence.to_string())]
        } else {
            // 包含中文、a-Z：z-Z的单独拆分
            let replacement = |caps: &Captures| format!("\n{}\n", caps.get(0).unwrap().as_str());

            let caps = PATTERN_2.replace_all(sentence, replacement).to_string();
            caps.split("\n")
                .filter(|s| !s.is_empty())
                .flat_map(|s| self.lang_seg_texts(s))
                .collect()
        }
    }

    // "包含a-b"："包含a至b"
    fn replae_az_range(&self, sentence: &str, lang: &str) -> String {
        let (zhi, gan) = if lang == CHINESE_LANG {
            ("至", "杠")
        } else {
            (" to ", " ")
        };
        let replacement = |caps: &Captures| -> String {
            let a1 = caps.get(1).map_or("", |m| m.as_str());
            let first = caps.get(2).map_or("", |_| zhi);
            let a2 = caps.get(3).map_or("", |m| m.as_str());
            format!("{}{}{}", a1, first, a2)
        };

        let caps = PATTERN_ALPHA_RANGE
            .replace_all(sentence, replacement)
            .to_string();

        let replacement = |caps: &Captures| -> String {
            let a1 = caps.get(1).map_or("", |m| m.as_str());
            let first = caps.get(2).map_or("", |_| gan);
            let a2 = caps.get(3).map_or("", |m| m.as_str());
            format!("{}{}{}", a1, first, a2)
        };

        PATTERN_ALPHA_RANGE2
            .replace_all(&caps, replacement)
            .to_string()
    }

    /// 二次规则分割
    pub fn lang_seg_texts2(&self, sentence: &str, lang: &str) -> Vec<(String, String)> {
        self.zh_en_seg(&self.replae_az_range(sentence, lang), lang)
    }

    /// 获取文本中的多语言
    ///
    /// "hello，Google.。我们中出了一个叛徒"
    ///
    /// English: hello，Google.。
    ///
    /// Chinese: 我们中出了一个叛徒
    pub fn lang_seg_texts(&self, sentence: &str) -> Vec<(String, String)> {
        let results = self.detector.detect_multiple_languages_of(sentence);
        let mut out = results
            .iter()
            .map(|res| {
                (
                    res.language().to_string(),
                    sentence[res.start_index()..res.end_index()].to_string(),
                )
            })
            .collect::<Vec<_>>();
        // 123344 -> 纯数字、数字+标点，无法识别
        if out.is_empty() {
            // 默认中文
            out.push((CHINESE_LANG.to_string(), sentence.to_string()));
        }
        out
    }

    fn _split(&self, todo_text: &str) -> Vec<String> {
        let mut todo_text = todo_text.replace("……", "。").replace("——", "，");

        if !todo_text.ends_with(|c: char| self._splits.contains(&c.to_string())) {
            todo_text.push('。');
        }

        let mut result = Vec::new();
        let mut current_segment = String::new();

        for c in todo_text.chars() {
            current_segment.push(c);
            if self._splits.contains(&c.to_string()) {
                result.push(std::mem::take(&mut current_segment));
            }
        }

        result
    }

    fn _cut2(&self, inp: &str, max_num: usize) -> String {
        let inp = inp.trim_matches('\n').to_string();
        let inps = self._split(&inp);
        if inps.len() < 2 {
            return inp;
        }

        let mut opts = Vec::new();
        let mut summ = 0;
        let mut tmp_str = String::new();

        for segment in inps {
            summ += segment.chars().count();
            tmp_str.push_str(&segment);

            if summ > max_num {
                summ = 0;
                opts.push(std::mem::take(&mut tmp_str));
            }
        }

        if !tmp_str.is_empty() {
            opts.push(tmp_str);
        }

        // let opts_len = opts.len();
        // if opts_len > 1 && opts.last().unwrap().chars().count() < max_num {
        //     let last = opts.pop().unwrap();
        //     opts.last_mut().unwrap().push_str(&last);
        // }

        opts.join("\n")
    }

    // 以中文句号
    fn cut3(&self, inp: &str, max_num: usize) -> String {
        let inp = inp.trim_matches('\n');
        let mut inps: Vec<String> = inp
            .trim_matches('。')
            .split("。")
            .map(String::from)
            .collect();
        inps.iter_mut()
            .filter(|s| s.chars().count() > max_num)
            .for_each(|s| *s = s.split("，").collect::<Vec<&str>>().join("\n"));
        inps.join("\n")
    }

    fn merge_short_text_in_array(&self, texts: Vec<String>, threshold: usize) -> Vec<String> {
        if texts.len() < 2 {
            return texts;
        }

        let mut result = Vec::new();
        let mut buffer = String::new();

        for text in texts {
            buffer.push_str(&text);
            if buffer.chars().count() >= threshold {
                result.push(std::mem::take(&mut buffer)); // 交换空字符串，避免重新分配
            }
        }

        // 处理最后的 buffer
        if !buffer.is_empty() {
            if let Some(last) = result.last_mut() {
                last.push_str(&buffer);
            } else {
                result.push(buffer);
            }
        }

        result
    }

    /// 切割文本成小断
    pub fn cut_texts(&self, text: &str, max_num: usize) -> Vec<String> {
        let text = self.cut3(text, max_num);
        // let text = self.cut2(&text, max_num);
        let texts: Vec<String> = text.split("\n").map(|s| s.to_string()).collect();

        self.merge_short_text_in_array(texts, 5)
    }
}

impl TextUtils {
    /// 英语处理需要的文本
    pub(crate) fn new(
        eng_dict_json_path: &str,
        rep_map_json_path: &str,
        ph_model_path: &str,
        phrases_dict_path: &str,
        pinyin_dict_path: &str,
    ) -> Result<Self, String> {
        // let languages = vec![English, Chinese, Japanese];
        let languages = vec![English, Chinese];
        let lang_seg: LangSegment = LangSegment::new(languages);
        let lang_chinese =
            text::chinese::Chinese::new(rep_map_json_path, phrases_dict_path, pinyin_dict_path)
                .unwrap();
        let lang_english = text::english::English::new(eng_dict_json_path, ph_model_path).unwrap();

        let symbol_to_id = SYMBOLS
            .iter()
            .enumerate()
            .map(|(i, s)| (s.to_string(), i))
            .collect();

        Ok(TextUtils {
            lang_seg,
            lang_chinese,
            lang_english,
            symbol_to_id,
        })
    }

    // 有特殊符号的处理，仅针对中文
    fn clean_special(
        &self,
        text: &str,
        language: &String,
        special_s: &str,
        target_symbol: &str,
    ) -> (Vec<String>, Vec<usize>, String) {
        let text = text.replace(special_s, ",");
        let (phones, word2ph) = if language == CHINESE_LANG {
            let norm_text = self.lang_chinese.text_normalize(&text);
            let (phones, word2ph) = self.lang_chinese.g2p(&norm_text);
            (phones, word2ph)
        } else {
            (vec![], vec![])
        };
        let new_ph = phones
            .iter()
            .map(|ph| {
                if SYMBOLS.contains(&ph.as_str()) {
                    if ph == "," {
                        target_symbol.to_string()
                    } else {
                        ph.to_string()
                    }
                } else {
                    ph.to_string()
                }
            })
            .collect();
        (new_ph, word2ph, text.to_string())
    }

    /// 单一语言的处理
    fn clean_text_inf(&self, text: &str, language: &String) -> (Vec<String>, Vec<usize>, String) {
        let (mut text, language) = {
            if language != ENGLISH_LANG && language != CHINESE_LANG && language != JAPANESE_LANG {
                (" ".to_string(), ENGLISH_LANG.to_string())
            } else {
                (text.to_string(), language.clone())
            }
        };
        let special = vec![("￥", CHINESE_LANG, "SP2"), ("^", CHINESE_LANG, "SP3")];
        for (special_s, special_l, target_symbol) in special {
            if text.contains(special_s) && language == special_l {
                let (phones, word2ph, norm_text) =
                    self.clean_special(&text, &language, special_s, target_symbol);
                return (phones, word2ph, norm_text);
            }
        }

        let mut norm_text = "".to_string();
        let mut phones: Vec<String> = vec![];
        let mut word2ph: Vec<usize> = vec![];

        if language == CHINESE_LANG {
            norm_text = self.lang_chinese.text_normalize(&text);
            (phones, word2ph) = self.lang_chinese.g2p(&norm_text);
        } else if language == ENGLISH_LANG {
            // 英文中可能多余符号
            text = self.lang_english.text_normalize(&text);
            norm_text = self.lang_chinese.replace_symbol(&text);
            phones = self.lang_english.g2p(&norm_text);
        } else if language == JAPANESE_LANG {
            // todo
        }

        (phones, word2ph, norm_text)
    }

    /// Converts a string of text to a sequence of IDs corresponding to the symbols in the text
    fn cleaned_text_to_sequence(&self, cleaned_texts: &[String]) -> Vec<usize> {
        cleaned_texts
            .iter()
            .map(|symbol| self.symbol_to_id.get(symbol).cloned().unwrap_or(0))
            .collect()
    }

    /// 可以是混合中英文的原始文本
    pub fn get_cleaned_text_final(&self, short_text: &str) -> CleanedText {
        let seg_texts = self.lang_seg.lang_seg_texts(short_text);
        let mut phones_list: Vec<Vec<usize>> = vec![];
        let mut lang_list: Vec<String> = vec![];
        let mut word2ph_list: Vec<Vec<usize>> = vec![];
        let mut norm_text_list: Vec<String> = vec![];
        for seg in seg_texts {
            let (lang, text) = &seg;

            let seg_texts2 = self.lang_seg.lang_seg_texts2(text, lang);
            for (ei, (lang2, text2)) in seg_texts2.iter().enumerate() {
                if text2.is_empty() {
                    continue;
                }
                let mut text2 = text2.clone();
                // 添加标题
                if ei == 0 && !text2.chars().nth(0).unwrap().is_numeric() {
                    if lang2 == CHINESE_LANG {
                        text2 = "。".to_string() + &text2;
                    } else if lang2 == ENGLISH_LANG {
                        text2 = ". ".to_string() + &text2;
                    }
                }
                let (phones, mut word2ph, norm_text) = self.clean_text_inf(&text2, lang2);
                let mut phones = self.cleaned_text_to_sequence(&phones);
                // todo : 合并同语言
                let p_len = phones_list.len();
                let lang_len = lang_list.len();
                let norm_lang_len = norm_text_list.len();
                if p_len > 0 && lang_len > 0 && norm_lang_len > 0 {
                    // 同语言
                    if &lang_list[lang_len - 1] == lang2 {
                        phones_list[p_len - 1].append(&mut phones);
                        word2ph_list[norm_lang_len - 1].append(&mut word2ph);
                        // lang_list[lang_len - 1] = lang_list[lang_len - 1].to_string() + text2;
                        norm_text_list[lang_len - 1] =
                            norm_text_list[norm_lang_len - 1].to_string() + &norm_text;
                        continue;
                    }
                }
                if norm_text.trim().chars().count() > 0 {
                    phones_list.push(phones);
                    lang_list.push(lang2.to_string());

                    word2ph_list.push(word2ph);

                    norm_text_list.push(norm_text);
                }
            }
        }

        CleanedText {
            phones_list,
            word2ph_list,
            lang_list,
            norm_text_list,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_text_utils() -> TextUtils {
        TextUtils::new(
            "../assets/eng_dict.json",
            "../assets/rep_map.json",
            "../assets/model.npz",
            "../assets/PHRASES_DICT.json",
            "../assets/PINYIN_DICT.json",
        )
        .expect("Failed to create TextUtils")
    }

    #[test]
    pub fn chinese_test0() {
        // let a="a一个";
        let text_util = create_text_utils();

        let text = "IT的我们是搞Google的".to_string();

        let texts = text_util.lang_seg.cut_texts(&text, 30);
        texts.iter().for_each(|text| {
            println!("{:?}", text);
            let CleanedText {
                phones_list,
                word2ph_list,
                lang_list,
                norm_text_list,
            } = text_util.get_cleaned_text_final(text);

            assert_eq!(
                phones_list,
                vec![
                    vec![3, 55, 80],
                    vec![127, 134, 316, 232, 225, 144, 251, 214, 156, 119],
                    vec![50, 88, 50, 12, 62],
                    vec![127, 134]
                ]
            );
            assert_eq!(
                word2ph_list,
                vec![vec![], vec![2, 2, 2, 2, 2], vec![], vec![2]]
            );
            assert_eq!(lang_list, ["English", "Chinese", "English", "Chinese"]);
            assert_eq!(norm_text_list, [". IT", "的我们是搞", "Google", "的"]);
        });
    }

    #[test]
    pub fn chinese_test1() {
        // let a="a一个";
        let text_util = create_text_utils();

        let text = "这是一段纯中文文本，没有任何其他语言的字符。".to_string();

        let texts = text_util.lang_seg.cut_texts(&text, 30);
        texts.iter().for_each(|text| {
            println!("{:?}", text);
            let CleanedText {
                phones_list,
                word2ph_list,
                lang_list,
                norm_text_list,
            } = text_util.get_cleaned_text_final(text);

            assert_eq!(
                phones_list,
                vec![vec![
                    3, 320, 133, 251, 214, 318, 167, 127, 273, 125, 286, 320, 235, 316, 141, 316,
                    141, 122, 142, 1, 225, 136, 318, 242, 248, 143, 158, 131, 247, 167, 252, 97,
                    318, 298, 318, 45, 127, 134, 319, 164, 155, 256
                ],]
            );
            assert_eq!(
                word2ph_list,
                vec![vec![
                    1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2
                ],]
            );
            assert_eq!(lang_list, ["Chinese",]);
            assert_eq!(
                norm_text_list,
                [".这是一段纯中文文本,没有任何其他语言的字符"]
            );
        });
    }

    #[test]
    pub fn chinese_test2() {
        // let a="a一个";
        let text_util = create_text_utils();

        let text = "This is a pure English text without any other language characters.".to_string();

        let texts = text_util.lang_seg.cut_texts(&text, 30);
        texts.iter().for_each(|text| {
            println!("{:?}", text);
            let CleanedText {
                phones_list,
                word2ph_list,
                lang_list,
                norm_text_list,
            } = text_util.get_cleaned_text_final(text);

            assert_eq!(
                phones_list,
                vec![vec![
                    3, 27, 55, 75, 55, 93, 12, 73, 92, 84, 74, 55, 65, 50, 62, 54, 76, 80, 35, 61,
                    75, 80, 91, 54, 81, 19, 80, 35, 64, 57, 13, 27, 38, 62, 10, 65, 50, 91, 12, 60,
                    61, 10, 74, 54, 61, 80, 38, 93, 3
                ],]
            );
            assert_eq!(word2ph_list, vec![Vec::<usize>::new()]);
            assert_eq!(lang_list, ["English",]);
            assert_eq!(
                norm_text_list,
                [". This is a pure English text without any other language characters."]
            );
        });
    }

    #[test]
    pub fn chinese_test3() {
        // let a="a一个";
        let text_util = create_text_utils();

        let text = "1234567890".to_string();

        let texts = text_util.lang_seg.cut_texts(&text, 30);
        texts.iter().for_each(|text| {
            println!("{:?}", text);
            let CleanedText {
                phones_list,
                word2ph_list,
                lang_list,
                norm_text_list,
            } = text_util.get_cleaned_text_final(text);

            assert_eq!(
                phones_list,
                vec![vec![
                    251, 212, 33, 153, 318, 169, 250, 107, 247, 176, 250, 164, 122, 104, 316, 257,
                    251, 212, 224, 219, 316, 110, 247, 166, 247, 176, 122, 97, 122, 104, 221, 218,
                    251, 212
                ],]
            );
            assert_eq!(
                word2ph_list,
                vec![vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],]
            );
            assert_eq!(lang_list, ["Chinese",]);
            assert_eq!(norm_text_list, ["十二亿三千四百五十六万七千八百九十"]);
        });
    }

    #[test]
    pub fn chinese_test4() {
        // let a="a一个";
        let text_util = create_text_utils();

        let text = "This is a text with special characters ^.".to_string();

        let texts = text_util.lang_seg.cut_texts(&text, 30);
        texts.iter().for_each(|text| {
            println!("{:?}", text);
            let CleanedText {
                phones_list,
                word2ph_list,
                lang_list,
                norm_text_list,
            } = text_util.get_cleaned_text_final(text);

            assert_eq!(
                phones_list,
                vec![vec![
                    3, 27, 55, 75, 55, 93, 12, 80, 35, 61, 75, 80, 91, 55, 27, 75, 73, 35, 76, 12,
                    62, 61, 10, 74, 54, 61, 80, 38, 93, 3
                ],]
            );
            assert_eq!(word2ph_list, vec![Vec::<usize>::new()]);
            assert_eq!(lang_list, ["English",]);
            assert_eq!(
                norm_text_list,
                [". This is a text with special characters ^."]
            );
        });
    }

    #[test]
    pub fn chinese_test5() {
        // let a="a一个";
        let text_util = create_text_utils();

        let text = "这是，一段；中文：文本？".to_string();

        let texts = text_util.lang_seg.cut_texts(&text, 30);
        texts.iter().for_each(|text| {
            println!("{:?}", text);
            let CleanedText {
                phones_list,
                word2ph_list,
                lang_list,
                norm_text_list,
            } = text_util.get_cleaned_text_final(text);

            assert_eq!(
                phones_list,
                vec![vec![
                    3, 320, 133, 251, 214, 1, 318, 167, 127, 273, 1, 320, 235, 316, 141, 1, 316,
                    141, 122, 142, 4
                ],]
            );
            assert_eq!(
                word2ph_list,
                vec![vec![1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1],]
            );
            assert_eq!(lang_list, ["Chinese",]);
            assert_eq!(norm_text_list, [".这是,一段,中文,文本?"]);
        });
    }

    #[test]
    pub fn chinese_test6() {
        // let a="a一个";
        let text_util = create_text_utils();

        let text = "从 a 到 z，从 A 到 Z 的范围表示。".to_string();

        let texts = text_util.lang_seg.cut_texts(&text, 30);
        texts.iter().for_each(|text| {
            println!("{:?}", text);
            let CleanedText {
                phones_list,
                word2ph_list,
                lang_list,
                norm_text_list,
            } = text_util.get_cleaned_text_final(text);

            assert_eq!(
                phones_list,
                vec![
                    vec![3, 124, 236],
                    vec![12],
                    vec![127, 120],
                    vec![93, 58],
                    vec![1, 124, 236],
                    vec![12],
                    vec![127, 120],
                    vec![93, 58],
                    vec![127, 134, 155, 110, 316, 136, 122, 188, 251, 214],
                ]
            );
            assert_eq!(
                word2ph_list,
                vec![
                    vec![1, 2],
                    vec![],
                    vec![2],
                    vec![],
                    vec![1, 2],
                    vec![],
                    vec![2],
                    vec![],
                    vec![2, 2, 2, 2, 2]
                ]
            );
            assert_eq!(
                lang_list,
                [
                    "Chinese", "English", "Chinese", "English", "Chinese", "English", "Chinese",
                    "English", "Chinese"
                ]
            );
            assert_eq!(
                norm_text_list,
                [".从", "a", "到", "z", ",从", "A", "到", "Z", "的范围表示"]
            );
        });
    }

    #[test]
    pub fn chinese_test7() {
        // let a="a一个";
        let text_util = create_text_utils();

        let text = "乘客朋友，您好，您现在即将体验和参观的是无人之境项目，无人之境示范体验区是国家智能网联汽车上海试点示范区的重要组成部分，可支撑无人化高级别自动驾驶技术测试验证。目前已实现无人驾驶小巴，robot taxi，无人清扫等多业态无人驾驶应用场景。同时也欢迎您乘坐体验酷哇科技无人驾驶小巴，我们具备完善的功能配置，可完成十余项自动驾驶场景展示。包括路径规划，智能避障，站点停泊，临时起停，自动返场，自主泊车等，360度全景智能交互。在感知，控制，底盘，供电等各个环节，执行冗余式安全策略，切实保障乘客安全，后续将以预约形式逐步开放给社会公众。本车由上海汽车博物馆站，开往一维诶爱智行港终点站，下一站，房车中国上海基地站，车辆离站，请系好安全带。".to_string();
        let expected_phones = [
            vec![vec![
                3, 125, 146, 222, 133, 245, 146, 318, 244, 1, 227, 197, 158, 119, 1, 227, 197, 317,
                179, 319, 105, 221, 167, 221, 181, 252, 168, 318, 47, 158, 131, 124, 107, 156, 270,
                127, 134, 251, 214, 316, 256, 248, 141, 320, 211, 221, 204, 317, 184, 225, 258, 1,
                316, 256, 248, 141, 320, 211, 221, 204, 251, 214, 155, 110, 252, 168, 318, 47, 247,
                296, 251, 214, 156, 291, 221, 171, 320, 214, 227, 146, 316, 114, 224, 177, 247,
                169, 125, 130, 251, 115, 158, 104, 251, 214, 127, 178, 251, 214, 155, 110, 247,
                296, 127, 134, 320, 238, 318, 120, 319, 257, 125, 146, 122, 258, 155, 144, 1, 222,
                132, 320, 211, 125, 145, 316, 256, 248, 141, 158, 263, 156, 117, 221, 167, 122,
                192, 319, 164, 127, 238, 221, 174, 251, 213, 221, 169, 251, 258, 124, 133, 251,
                214, 318, 47, 320, 148,
            ]],
            vec![
                vec![
                    3, 225, 258, 247, 177, 318, 168, 251, 212, 317, 179, 316, 256, 248, 141, 221,
                    174, 251, 213, 317, 188, 122, 97, 1,
                ],
                vec![3, 74, 68, 24, 8, 80, 80, 10, 61, 75, 57, 1],
                vec![
                    3, 316, 256, 248, 141, 247, 201, 250, 119, 127, 147, 127, 290, 318, 31, 252,
                    105, 316, 256, 248, 141, 221, 174, 251, 213, 318, 204, 318, 238, 125, 113, 221,
                    203,
                ],
            ],
            vec![vec![
                3, 252, 236, 251, 212, 318, 30, 158, 270, 318, 202, 227, 197, 125, 146, 319, 293,
                252, 168, 318, 47, 222, 258, 316, 101, 222, 130, 221, 169, 316, 256, 248, 141, 221,
                174, 251, 213, 317, 188, 122, 97, 1, 316, 232, 225, 144, 221, 299, 122, 138, 316,
                108, 251, 110, 127, 134, 156, 235, 227, 146, 245, 138, 320, 214, 1, 222, 132, 316,
                108, 125, 146, 251, 212, 318, 297, 317, 184, 319, 164, 127, 238, 221, 174, 251,
                213, 125, 113, 221, 203, 320, 109, 251, 214,
            ]],
            vec![vec![
                3, 122, 117, 222, 293, 224, 258, 221, 204, 156, 280, 158, 263, 1, 320, 214, 227,
                146, 122, 169, 320, 115, 1, 320, 110, 127, 178, 252, 202, 122, 231, 1, 224, 197,
                251, 212, 247, 168, 252, 202, 1, 319, 164, 127, 238, 155, 109, 125, 114, 1, 319,
                164, 320, 257, 122, 231, 125, 130, 127, 147, 1, 250, 107, 122, 104, 224, 219, 251,
                212, 127, 258, 247, 302, 221, 203, 320, 214, 227, 146, 221, 186, 158, 258,
            ]],
            vec![vec![
                3, 319, 105, 156, 109, 320, 211, 1, 222, 238, 320, 214, 1, 127, 168, 245, 108, 1,
                156, 235, 127, 179, 127, 147, 156, 133, 156, 134, 158, 271, 221, 192, 1, 320, 212,
                317, 202, 248, 237, 318, 297, 251, 214, 5, 107, 247, 302, 124, 133, 224, 309, 1,
                247, 194, 251, 212, 122, 119, 320, 115, 125, 146, 222, 133, 5, 107, 247, 302, 1,
                158, 243, 317, 299, 221, 181, 318, 168, 318, 299, 318, 306, 317, 202, 251, 214,
                320, 256, 122, 258, 222, 102, 155, 115, 156, 137, 251, 133, 158, 283, 156, 235,
                320, 238,
            ]],
            vec![vec![
                3, 122, 142, 125, 130, 318, 241, 251, 115, 158, 104, 247, 169, 125, 130, 122, 231,
                316, 258, 156, 272, 320, 110, 1, 222, 102, 316, 114, 318, 169, 316, 136, 33, 139,
                5, 105, 320, 214, 317, 202, 156, 114, 320, 235, 127, 178, 320, 110, 1, 317, 174,
                318, 167, 320, 110, 1, 155, 113, 125, 130, 320, 235, 156, 291, 251, 115, 158, 104,
                221, 166, 127, 170, 320, 110, 1, 125, 130, 224, 184, 224, 167, 320, 110, 1, 247,
                203, 317, 169, 158, 119, 5, 107, 247, 302, 127, 105,
            ]],
        ];

        let expected_word2phs = [
            vec![vec![
                1, 2, 2, 2, 2, 1, 2, 2, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1,
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            ]],
            vec![
                vec![1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1],
                vec![],
                vec![1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
            ],
            vec![vec![
                1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            ]],
            vec![vec![
                1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 1, 2, 2, 2, 2, 1, 2, 2, 2, 2, 1, 2, 2, 2, 2, 1,
                2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            ]],
            vec![vec![
                1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                1, 2, 2, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            ]],
            vec![vec![
                1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 2, 2,
                2, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2,
            ]],
        ];

        let langs = [
            vec!["Chinese"],
            vec!["Chinese", "English", "Chinese"],
            vec!["Chinese"],
            vec!["Chinese"],
            vec!["Chinese"],
            vec!["Chinese"],
        ];
        let norm_texts=  [vec![".乘客朋友,您好,您现在即将体验和参观的是无人之境项目,无人之境示范体验区是国家智能网联汽车上海试点示范区的重要组成部分,可支撑无人化高级别自动驾驶技术测试验证"], 
                                              vec![".目前已实现无人驾驶小巴,", ". robot taxi,", ".无人清扫等多业态无人驾驶应用场景"],
                                              vec![".同时也欢迎您乘坐体验酷哇科技无人驾驶小巴,我们具备完善的功能配置,可完成十余项自动驾驶场景展示"],
                                              vec![".包括路径规划,智能避障,站点停泊,临时起停,自动返场,自主泊车等,三百六十度全景智能交互"],
                                              vec![".在感知,控制,底盘,供电等各个环节,执行冗余式安全策略,切实保障乘客安全,后续将以预约形式逐步开放给社会公众"],
                                              vec![".本车由上海汽车博物馆站,开往一维诶爱智行港终点站,下一站,房车中国上海基地站,车辆离站,请系好安全带"]];
        let max_chars = "乘客朋友，您好，您现在即将体验和参观的是无人之境项目，无人之境示范体验区是国家智能网联汽车上海试点示范区的重要组成部分，可支撑无人化高级别自动驾驶技术测试验证。".chars().count();
        println!("ref max chars: {:?}", max_chars);
        let texts = text_util.lang_seg.cut_texts(&text, max_chars);
        texts.iter().enumerate().for_each(|(index, text)| {
            println!("{:?}", text);
            let CleanedText {
                phones_list,
                word2ph_list,
                lang_list,
                norm_text_list,
            } = text_util.get_cleaned_text_final(text);

            assert_eq!(
                phones_list,
                expected_phones
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| vec![vec![0]])
            );
            assert_eq!(
                word2ph_list,
                expected_word2phs
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| vec![vec![0]])
            );
            assert_eq!(
                lang_list,
                langs.get(index).cloned().unwrap_or_else(|| vec![""])
            );
            assert_eq!(
                norm_text_list,
                norm_texts.get(index).cloned().unwrap_or_else(|| vec![""])
            );
        });
    }
}
