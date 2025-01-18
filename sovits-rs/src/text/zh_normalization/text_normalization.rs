use super::chronology::Chronology;
use super::num::Num;
use super::phonecode::Phonecode;
use super::quantifier::Quantifier;
use fnv::FnvHashMap;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde::Deserialize;
use std::collections::HashMap;

lazy_static! {
    static ref RE_SENTENCE_SPLITOR: Regex = Regex::new(r"([：、，；。？！,;?!][”’]?)").unwrap();
    static ref RE_SPECIAL_SYMBOL: Regex =
        Regex::new(r"[——《》【】<=>{}()（）#&@“”^_|…\\]").unwrap();
    static ref RE_SPECIAL_SYMBOL2: Regex =
        Regex::new(r"[-——《》【】<=>{}()（）#&@“”^_|…\\]").unwrap();
}

#[derive(Debug, Deserialize)]
struct ZhDict {
    t2s_mapping: T2SMapping,
    special_symbol_mapping: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct T2SMapping {
    traditional: String,
    simplified: String,
}

pub struct TextNormalizer {
    f2h_ascii_letters: FnvHashMap<char, char>,
    f2h_digits: FnvHashMap<char, char>,
    f2h_space: FnvHashMap<char, char>,
    t2s_dict: HashMap<char, char>,
    special_symbol_mapping: HashMap<String, String>,
    chronology: Chronology,
    quantifier: Quantifier,
    phonecode: Phonecode,
    num: Num,
}

impl TextNormalizer {
    pub(crate) fn new(dict_path: &str) -> Self {
        // 从 JSON 文件加载数据
        let zh_dict: ZhDict = match std::fs::read_to_string(dict_path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
        {
            Some(dict) => dict,
            None => panic!("Failed to load dictionary from path: {}", dict_path),
        };

        let t2s_dict = zh_dict
            .t2s_mapping
            .traditional
            .chars()
            .zip(zh_dict.t2s_mapping.simplified.chars())
            .collect();

        let (f2h_ascii_letters, f2h_digits, f2h_space) = Self::translate_table();

        Self {
            f2h_ascii_letters,
            f2h_digits,
            f2h_space,
            t2s_dict,
            special_symbol_mapping: zh_dict.special_symbol_mapping,
            chronology: Chronology::new(),
            quantifier: Quantifier::new(),
            phonecode: Phonecode::new(),
            num: Num,
        }
    }

    pub(crate) fn normalize(&self, sentence: &str) -> Vec<String> {
        let sentences = self.split(sentence, "zh");
        sentences
            .into_iter()
            .map(|sent| self.normalize_sentence(&sent))
            .collect()
    }

    fn split(&self, sentence: &str, lang: &str) -> Vec<String> {
        let mut sentences: Vec<String> = vec![];
        let mut text = sentence.to_string();
        if lang == "zh" {
            text = text.replace(" ", "");
            text = RE_SPECIAL_SYMBOL.replace_all(&text, ",").to_string();
            text = Regex::new(r",+")
                .unwrap()
                .replace_all(&text, ",")
                .to_string();
        }
        let replacement = |caps: &Captures| -> String {
            let v = &caps[0];
            v.to_string() + "\n"
        };
        text = RE_SENTENCE_SPLITOR
            .replace_all(&text, replacement)
            .to_string()
            .trim()
            .to_string();

        for split in text.split_inclusive('\n') {
            let s = split.trim().to_string();
            if !s.is_empty() {
                sentences.push(s);
            }
        }
        sentences
    }

    fn post_replace(&self, sentence: &str) -> String {
        let mut sentence = sentence.to_string();
        for (key, value) in &self.special_symbol_mapping {
            sentence = sentence.replace(key, value);
        }
        RE_SPECIAL_SYMBOL2.replace_all(&sentence, "").to_string()
    }

    fn tranditional_to_simplified(&self, sentence: &str) -> String {
        sentence
            .chars()
            .map(|c| *self.t2s_dict.get(&c).unwrap_or(&c))
            .collect()
    }

    fn translate_table() -> (
        FnvHashMap<char, char>,
        FnvHashMap<char, char>,
        FnvHashMap<char, char>,
    ) {
        let mut f2h_ascii_letters = FnvHashMap::default();
        let mut f2h_digits = FnvHashMap::default();
        let mut f2h_space = FnvHashMap::default();

        // 全角 ASCII 字母映射到半角
        for c in 'A'..='Z' {
            if let Some(fullwidth) = std::char::from_u32(c as u32 + 65248) {
                f2h_ascii_letters.insert(fullwidth, c);
            }
        }
        for c in 'a'..='z' {
            if let Some(fullwidth) = std::char::from_u32(c as u32 + 65248) {
                f2h_ascii_letters.insert(fullwidth, c);
            }
        }

        // 全角数字映射到半角
        for c in '0'..='9' {
            if let Some(fullwidth) = std::char::from_u32(c as u32 + 65248) {
                f2h_digits.insert(fullwidth, c);
            }
        }

        f2h_space.insert('\u{3000}', ' ');
        (f2h_ascii_letters, f2h_digits, f2h_space)
    }

    fn translate(&self, sentence: &str) -> String {
        sentence
            .chars()
            .map(|c| {
                self.f2h_ascii_letters
                    .get(&c)
                    .or_else(|| self.f2h_digits.get(&c))
                    .or_else(|| self.f2h_space.get(&c))
                    .copied()
                    .unwrap_or(c)
            })
            .collect()
    }

    fn normalize_sentence(&self, sentence: &str) -> String {
        let mut sentence = sentence.to_string();

        sentence = self.tranditional_to_simplified(&sentence);
        sentence = self.translate(&sentence);
        // 日期2021年5月26日
        // 日期2021/5/26日、21/5/26
        // 8:30-12:30
        // 12:30
        sentence = self.chronology.normalize(&sentence);
        // -3°C
        // 2cm²
        sentence = self.quantifier.normalize(&sentence);

        // 13813910908
        // 带区号的电话号码 021-12345678
        // 标准 400 开头号码 400-123-4567
        sentence = self.phonecode.normalize(&sentence);

        // 分数：1/2 、-1/3
        // 百分数 45%
        // 12.34- 15.2
        // 小数点的数字 12.34
        // 负数-123
        // 我有300+块钱
        // 我有00078块钱
        sentence = self.num.normalize(&sentence);
        sentence = self.post_replace(&sentence);

        sentence
    }
}

#[cfg(test)]
mod tests {
    use super::TextNormalizer;

    #[test]
    fn test_normalize() {
        let normalizer = TextNormalizer::new("../assets/zh_dict.json");
        let cases = vec![
            (
                "固话：0595-23865596或23880880。",
                vec!["固话：", "零五九五，二三八六五五九六或二三八八零八八零。"],
            ),
            (
                "手机：+86 19859213959或15659451527或者 +86 18612345678。",
                vec!["手机：", "八六幺九八五九二幺三九五九或幺五六五九四五幺五二七或者八六幺八六幺二三四五六七八。"],
            ),
        ];

        for (input, expected) in cases {
            let result = normalizer.normalize(input);
            assert_eq!(result, expected, "Test failed for input: {}", input);
        }
    }

    #[test]
    fn test_text_normalizer_cases() {
        let normalizer = TextNormalizer::new("../assets/zh_dict.json");
        let cases = vec![
            ("中文句子测试。", vec!["中文句子测试。"]),
            ("特殊符号：①②③，~。", vec!["特殊符号：", "一二三，", "至。"]),
            ("Test中文English测试。", vec!["Test中文English测试。"]),
            ("①②③~～/αβγΓ", vec!["一二三至至每阿尔法贝塔伽玛伽玛"]),
            ("   中文    测试    。", vec!["中文测试。"]),
            ("ＡＢＣＤ１２３４。", vec!["ABCD一千二百三十四。"]),
            ("", vec![]),
            ("汉", vec!["汉"]),
            ("①~！", vec!["一至！"]),
            ("    ", vec![]),
            (
                "这是一个非常非常非常非常非常非常非常非常非常非常长的句子，用来测试系统是否能处理超长句子的情况。",
                vec!["这是一个非常非常非常非常非常非常非常非常非常非常长的句子，", "用来测试系统是否能处理超长句子的情况。"]
            ),
            (
                "这是第一句。这是第二句！这是第三句？",
                vec!["这是第一句。", "这是第二句！", "这是第三句？"]
            ),
            (
                "中文，Test。Hello！测试，Done。",
                vec!["中文，", "Test。", "Hello！", "测试，", "Done。"]
            ),
            (
                "电话：1234567890，金额：￥100.00。",
                vec!["电话：", "十二亿三千四百五十六万七千八百九十，", "金额：", "￥一百。"]
            ),
            (
                "混合测试：①A～Z，123；中文。",
                vec!["混合测试：", "一A至Z，", "一百二十三；", "中文。"]
            ),
            (
                "日期：2025/01/14 ~ 2025/12/31。",
                vec!["日期：", "二零二五年一月十四日至二零二五年十二月三十一日。"]
            ),
            ("时间：10:30:45。", vec!["时间：", "十点半四十五秒。"]),
            ("未知字符：@#￥%……&*。", vec!["未知字符：", ",", "￥%,", "*。"]),
            ("测试测试测试测试测试测试。", vec!["测试测试测试测试测试测试。"]),
            ("嵌套（符号（测试））！", vec!["嵌套,", "符号,", "测试,", "！"]),
            ("这是一句话没有任何标点符号", vec!["这是一句话没有任何标点符号"]),
        ];

        for (input, expected) in cases {
            let result = normalizer.normalize(input);
            assert_eq!(result, expected, "Test failed for input: {}", input);
        }
    }
}
