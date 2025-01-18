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
    pub splits: Vec<String>,
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
        let splits: Vec<String> = vec![
            "，", "。", "？", "！", ",", ".", "?", "!", "~", ":", "：", "—", "…",
        ]
        .iter()
        .map(|&s| s.to_string())
        .collect();

        LangSegment { splits, detector }
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

    fn split(&self, todo_text: &str) -> Vec<String> {
        let mut todo_text = todo_text.replace("……", "。").replace("——", "，");

        if !todo_text.ends_with(|c: char| self.splits.contains(&c.to_string())) {
            todo_text.push('。');
        }

        let mut result = Vec::new();
        let mut current_segment = String::new();

        for c in todo_text.chars() {
            current_segment.push(c);
            if self.splits.contains(&c.to_string()) {
                result.push(std::mem::take(&mut current_segment));
            }
        }

        result
    }

    fn cut2(&self, inp: &str, max_num: usize) -> String {
        let inp = inp.trim_matches('\n');
        let inps = self.split(inp);

        if inps.len() < 2 {
            return inp.to_string();
        }

        let mut opts = Vec::new();
        let mut current_line = String::new();
        let mut current_len = 0;

        for segment in inps {
            let segment_len = segment.chars().count();
            if current_len + segment_len > max_num {
                opts.push(std::mem::take(&mut current_line));
                current_len = 0;
            }
            current_line.push_str(&segment);
            current_len += segment_len;
        }

        if !current_line.is_empty() {
            opts.push(current_line);
        }

        if opts.len() > 1 && opts.last().unwrap().chars().count() < max_num {
            let last = opts.pop().unwrap();
            opts.last_mut().unwrap().push_str(&last);
        }

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
            if buffer.chars().count() + text.chars().count() >= threshold && !buffer.is_empty() {
                result.push(std::mem::take(&mut buffer));
            }
            buffer.push_str(&text);
        }

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
        let text = self.cut2(&text, max_num);
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
            println!("text:{}, len:{}", text, text.trim().chars().count());
            norm_text = self.lang_chinese.text_normalize(&text);
            println!(
                "norm_text:{}, len:{}",
                norm_text,
                norm_text.trim().chars().count()
            );

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
                    318, 298, 318, 45, 127, 134, 319, 164, 155, 256, 3
                ],]
            );
            assert_eq!(
                word2ph_list,
                vec![vec![
                    1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1
                ],]
            );
            assert_eq!(lang_list, ["Chinese",]);
            assert_eq!(
                norm_text_list,
                [".这是一段纯中文文本,没有任何其他语言的字符."]
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
                    vec![127, 134, 155, 110, 316, 136, 122, 188, 251, 214, 3],
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
                    vec![2, 2, 2, 2, 2, 1]
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
                [".从", "a", "到", "z", ",从", "A", "到", "Z", "的范围表示."]
            );
        });
    }
}
