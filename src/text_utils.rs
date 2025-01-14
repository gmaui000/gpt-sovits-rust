use super::text::{self, symbols::SYMBOLS};
use lingua::Language::{Chinese, English};
use lingua::{DetectionResult, Language, LanguageDetector, LanguageDetectorBuilder};
use regex::{Captures, Regex};
use std::collections::HashMap;
use substring::Substring;

pub(crate) const ENGLISH_LANG: &str = "English";
pub(crate) const CHINESE_LANG: &str = "Chinese";
pub(crate) const JAPANESE_LANG: &str = "Japanese";

pub struct LangSegment {
    pub splits: Vec<String>,
    pub detector: LanguageDetector,
    pub pattern_alpha_range: Regex,
    pub pattern_alpha_range2: Regex,
    pub pattern_az: Regex,
    pub pattern_zh: Regex,
    pub pattern2: Regex,
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
    pub fn init(languages: Vec<Language>) -> Self {
        let detector: LanguageDetector =
            LanguageDetectorBuilder::from_languages(&languages).build();
        let splits: Vec<String> = vec![
            "，", "。", "？", "！", ",", ".", "?", "!", "~", ":", "：", "—", "…",
        ]
        .iter()
        .map(|&s| s.to_string())
        .collect();
        let pattern_alpha_range = Regex::new(r"([a-zA-Z]+)([—\->～~])([a-zA-Z]+)").unwrap();
        let pattern_alpha_range2 = Regex::new(r"([a-zA-Z]+)([—\->～~])([0-9]+)").unwrap();

        let pattern_az = Regex::new(r"[a-zA-Z]+").unwrap();
        let pattern2 = Regex::new(r"[a-zA-Z0-9|.%]+").unwrap();

        let pattern_zh = Regex::new(r"[\u4e00-\u9fa5]+").unwrap();

        LangSegment {
            splits,
            detector,
            pattern_alpha_range,
            pattern_alpha_range2,
            pattern_az,
            pattern_zh,
            pattern2,
        }
    }

    /// "包含AC到BZ" -> 被识别为中文，需要拆分为 中文（包含）、英文(A C)、中文（到）、英文(B Z)
    fn zh_en_seg(&self, sentence: &str, lang: &str) -> Vec<(String, String)> {
        let mut result = vec![];

        // 一个字母都没有：直接返回原始
        if !self.pattern_az.is_match(sentence) {
            result.push((lang.to_string(), sentence.to_string()));
            return result;
        }

        // 一个中文都没有
        if !self.pattern_zh.is_match(sentence) {
            result.push((lang.to_string(), sentence.to_string()));
            return result;
        }

        // 包含中文、a-Z：z-Z的单独拆分
        let replacement = |caps: &Captures| -> String {
            let a1 = caps.get(0).map_or("", |m| m.as_str());
            let result = format!("\n{}\n", a1);
            result
        };

        let caps = self.pattern2.replace_all(sentence, replacement).to_string();
        let cap_splits: Vec<&str> = caps.split("\n").collect();

        for cap_split in cap_splits {
            if !cap_split.is_empty() {
                let mut r = self.lang_seg_texts(cap_split);
                result.append(&mut r);
            }
        }
        result
    }

    // "包含a-b"："包含a至b"
    fn replae_az_range(&self, sentence: &str, lang: &str) -> String {
        let zhi = {
            if lang == CHINESE_LANG {
                "至"
            } else {
                " to "
            }
        };
        let gan = {
            if lang == CHINESE_LANG {
                "杠"
            } else {
                " "
            }
        };
        let replacement = |caps: &Captures| -> String {
            let a1 = caps.get(1).map_or("", |m| m.as_str());
            let first = caps.get(2).map_or("", |_| zhi);
            let a2 = caps.get(3).map_or("", |m| m.as_str());
            let result = format!("{}{}{}", a1, first, a2);
            result
        };

        let caps = self
            .pattern_alpha_range
            .replace_all(sentence, replacement)
            .to_string();

        let replacement = |caps: &Captures| -> String {
            let a1 = caps.get(1).map_or("", |m| m.as_str());
            let first = caps.get(2).map_or("", |_| gan);
            let a2 = caps.get(3).map_or("", |m| m.as_str());
            let result = format!("{}{}{}", a1, first, a2);
            result
        };

        let caps = self
            .pattern_alpha_range2
            .replace_all(&caps, replacement)
            .to_string();

        caps
    }

    /// 二次规则分割
    pub fn lang_seg_texts2(&self, sentence: &str, lang: &str) -> Vec<(String, String)> {
        let sentence = self.replae_az_range(sentence, lang);

        self.zh_en_seg(&sentence, lang)
    }

    /// 获取文本中的多语言
    ///
    /// "hello，Google.。我们中出了一个叛徒"
    ///
    /// English: hello，Google.。
    ///
    /// Chinese: 我们中出了一个叛徒
    pub fn lang_seg_texts(&self, sentence: &str) -> Vec<(String, String)> {
        let results: Vec<DetectionResult> = self.detector.detect_multiple_languages_of(sentence);
        let mut out: Vec<(String, String)> = Vec::new();
        for res in &results {
            let lang = res.language();
            let text = sentence[res.start_index()..res.end_index()].to_string();
            out.push((lang.to_string(), text));
        }
        // 123344 -> 纯数字、数字+标点，无法识别
        if results.is_empty() {
            // 默认中文
            out.push((CHINESE_LANG.to_string(), sentence.to_string()));
        }
        out
    }

    fn split(&self, todo_text: &str) -> Vec<String> {
        let mut todo_text = todo_text.replace("……", "。").replace("——", "，");
        let todo_text_len = todo_text.chars().count();
        let todo_text_last = todo_text
            .chars()
            .nth(todo_text_len - 1)
            .unwrap()
            .to_string();
        if !self.splits.contains(&todo_text_last) {
            todo_text += "。"
        }
        let mut i_split_head = 0;
        let mut i_split_tail = 0;
        let len_text = todo_text.chars().count();
        let mut todo_texts: Vec<String> = vec![];
        loop {
            if i_split_head >= len_text {
                break;
            }
            if self
                .splits
                .contains(&todo_text.chars().nth(i_split_head).unwrap().to_string())
            {
                i_split_head += 1;
                todo_texts.push(todo_text.substring(i_split_tail, i_split_head).to_string());
                i_split_tail = i_split_head
            } else {
                i_split_head += 1
            }
        }
        todo_texts
    }

    fn cut2(&self, inp: &str, max_num: usize) -> String {
        let inp = inp.trim_matches('\n').to_string();
        let inps = self.split(&inp);
        if inps.len() < 2 {
            return inp;
        }

        let mut opts = vec![];
        let mut summ = 0;
        let mut tmp_str = "".to_string();
        for c in inps {
            summ += c.chars().count();
            tmp_str = tmp_str + &c;
            if summ > max_num {
                summ = 0;
                opts.push(tmp_str);
                tmp_str = "".to_string();
            }
        }
        if !tmp_str.is_empty() {
            opts.push(tmp_str);
        }
        let opts_len = opts.len();
        if opts_len > 1 && opts[opts_len - 1].chars().count() < max_num {
            opts[opts_len - 2] = opts[opts_len - 2].to_string() + &opts[opts_len - 1];
            opts = opts[..opts_len - 1].to_vec();
        }
        opts.join("\n")
    }

    // 以中文句号
    fn cut3(&self, inp: &str, max_num: usize) -> String {
        let inp = inp.trim_matches('\n');
        let mut inps: Vec<String> = inp
            .trim_matches('。')
            .split("。")
            .map(|x| x.to_string())
            .collect();
        (0..inps.len()).for_each(|i| {
            if inps[i].chars().count() > max_num {
                let new_ips: Vec<&str> = inps[i].split("，").collect();
                let new_ipstr = new_ips.join("\n");
                inps[i] = new_ipstr;
            }
        });
        inps.join("\n")
    }

    fn merge_short_text_in_array(texts: Vec<String>, threshold: usize) -> Vec<String> {
        if texts.len() < 2 {
            return texts;
        }

        let mut result = vec![];
        let mut text = "".to_string();
        for ele in texts {
            text = text + &ele;
            if text.chars().count() >= threshold {
                result.push(text);
                text = "".to_string();
            }
        }
        if text.chars().count() > 0 {
            if result.is_empty() {
                result.push(text);
            } else {
                let r_len = result.len();
                result[r_len - 1] = result[r_len - 1].to_string() + &text;
            }
        }
        result
    }

    /// 切割文本成小断
    pub fn cut_texts(&self, text: &str, max_num: usize) -> Vec<String> {
        let text = self.cut3(text, max_num);
        let text = self.cut2(&text, max_num);
        let texts: Vec<String> = text.split("\n").map(|s| s.to_string()).collect();

        LangSegment::merge_short_text_in_array(texts, 5)
    }
}

impl TextUtils {
    /// 英语处理需要的文本
    pub(crate) fn init(
        eng_dict_json_path: &str,
        rep_map_json_path: &str,
        ph_model_path: &str,
        phrases_dict_path: &str,
        pinyin_dict_path: &str,
    ) -> Result<Self, String> {
        // let languages = vec![English, Chinese, Japanese];
        let languages = vec![English, Chinese];
        let lang_seg: LangSegment = LangSegment::init(languages);
        let lang_chinese =
            text::chinese::Chinese::init(rep_map_json_path, phrases_dict_path, pinyin_dict_path);
        let lang_english_op = text::english::English::init(eng_dict_json_path, ph_model_path);
        if lang_english_op.is_err() {
            return Err(lang_english_op.err().unwrap());
        }

        let lang_english = lang_english_op.unwrap();

        let mut symbol_to_id: HashMap<String, usize> = HashMap::new();
        (0..SYMBOLS.len()).for_each(|i| {
            let s = SYMBOLS[i].to_string();
            symbol_to_id.insert(s, i);
        });

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
        let mut norm_text = "".to_string();
        let mut phones: (Vec<String>, Vec<usize>) = (vec![], vec![]);

        if language == CHINESE_LANG {
            norm_text = self.lang_chinese.text_normalize(text);
            phones = self.lang_chinese.g2p(&norm_text);
        }

        let mut new_ph = vec![];
        for ph in phones.0 {
            if SYMBOLS.contains(&&*ph) {
                if ph == "," {
                    new_ph.push(target_symbol.to_string());
                } else {
                    new_ph.push(ph.to_string());
                }
            }
        }
        (new_ph, phones.1, norm_text)
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
            norm_text = self.lang_chinese.text_normalize(text);
            println!(
                "norm_text:{}, len:{}",
                norm_text,
                norm_text.trim().chars().count()
            );

            (phones, word2ph) = self.lang_chinese.g2p(&norm_text);
        } else if language == ENGLISH_LANG {
            // 英文中可能多余符号
            text = self.lang_english.text_normalize(text);
            norm_text = self.lang_chinese.replace_symbol(text);
            phones = self.lang_english.g2p(&norm_text);
        } else if language == JAPANESE_LANG {
            // todo
        }

        (phones, word2ph, norm_text)
    }

    /// Converts a string of text to a sequence of IDs corresponding to the symbols in the text
    fn cleaned_text_to_sequence(&self, cleaned_texts: &Vec<String>) -> Vec<usize> {
        let mut phones: Vec<usize> = vec![];
        for symbol in cleaned_texts {
            let v = self.symbol_to_id.get(symbol);
            if let Some(v) = v {
                phones.push(*v);
            } else {
                phones.push(0);
            }
        }
        phones
    }

    /// 可以是混合中英文的原始文本
    pub fn get_cleaned_text_final(&self, short_text: &str) -> CleanedText {
        let seg_texts = self.lang_seg.lang_seg_texts(short_text);
        let mut phones_list: Vec<Vec<usize>> = vec![];
        let mut lang_list: Vec<String> = vec![];
        let mut word2ph_list: Vec<Vec<usize>> = vec![];
        let mut norm_text_list: Vec<String> = vec![];
        (0..seg_texts.len()).for_each(|i| {
            let (lang, text) = &seg_texts[i];

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
        });

        CleanedText {
            phones_list,
            word2ph_list,
            lang_list,
            norm_text_list,
        }
    }
}

#[test]
pub fn chinese_test0() {
    // let a="a一个";
    let text_util = TextUtils::init(
        "../data/eng_dict.json",
        "../data/rep_map.json",
        "../data/model.npz",
        "../data/PHRASES_DICT.json",
        "../data/PINYIN_DICT.json",
    )
    .unwrap();

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

        println!("{:?}", phones_list);
        println!("{:?}", word2ph_list);
        println!("{:?}", lang_list);
        println!("{:?}", norm_text_list);
    });
}
