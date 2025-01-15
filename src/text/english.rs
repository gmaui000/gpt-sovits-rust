use super::symbols::SYMBOLS;
use english_numbers::Formatting;
use grapheme_to_phoneme::Model;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

lazy_static! {
    static ref RE_COMMA_NUMBER: Regex = Regex::new(r"([0-9][0-9\,]+[0-9])").unwrap();
    static ref RE_DECIMAL_NUMBER: Regex = Regex::new(r"([0-9]+\.[0-9]+)").unwrap();
    static ref RE_POUNDS: Regex = Regex::new(r"£([0-9\.\,]*[0-9]+)").unwrap();
    static ref RE_DOLLARS: Regex = Regex::new(r"\$([0-9\.\,]*[0-9]+)").unwrap();
    static ref RE_ORDINAL: Regex = Regex::new(r"([0-9]+)(st|nd|rd|th)").unwrap();
    static ref RE_NUMBER: Regex = Regex::new(r"[0-9]+").unwrap();
    static ref RE_DELIMITER: Regex = Regex::new(r"([,，；;.。？！\-\?\!\s+])").unwrap();
}

pub struct English {
    eng_dict: HashMap<String, Vec<Vec<String>>>,
    rep_map: HashMap<String, String>,
    pho_model: Model,
}

impl English {
    pub fn new(eng_dict_json_path: &str, ph_model_path: &str) -> Result<Self, String> {
        let eng_dict: HashMap<String, Vec<Vec<String>>> =
            serde_json::from_reader(fs::File::open(eng_dict_json_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;

        let pho_model =
            Model::load_from_npz_file(Path::new(ph_model_path)).map_err(|e| e.to_string())?;

        let rep_map: HashMap<String, String> = [(";", ","), (":", ","), ("'", "-"), ("\"", "-")]
            .iter()
            .map(|&(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Ok(Self {
            eng_dict,
            rep_map,
            pho_model,
        })
    }

    pub fn text_normalize(&self, text: &str) -> String {
        self.normalize_numbers(text)
    }

    fn normalize_numbers(&self, text: &str) -> String {
        let text = RE_COMMA_NUMBER
            .replace_all(text, |caps: &Captures| caps[1].replace(",", ""))
            .to_string();

        let text = RE_POUNDS
            .replace_all(&text, |caps: &Captures| format!("{} pounds", &caps[1]))
            .to_string();
        let text = RE_DOLLARS
            .replace_all(&text, |caps: &Captures| {
                let num = &caps[1];
                let parts: Vec<&str> = num.split('.').collect();
                let dollars = parts
                    .first()
                    .and_then(|&d| d.parse::<i32>().ok())
                    .unwrap_or(0);
                let cents = parts
                    .get(1)
                    .map(|&c| c.trim_end_matches('0'))
                    .filter(|c| !c.is_empty())
                    .map(|c| {
                        c[..c.len().min(2)].parse::<i32>().unwrap_or(0)
                            * 10_i32.pow(2 - c.len().min(2) as u32)
                    })
                    .unwrap_or(0);

                match (dollars, cents) {
                    (0, 0) => "zero dollars".to_string(),
                    (d, 0) => format!("{} dollar{}", d, if d == 1 { "" } else { "s" }),
                    (0, c) => format!("{} cent{}", c, if c == 1 { "" } else { "s" }),
                    (d, c) => format!(
                        "{} dollar{}, {} cent{}",
                        d,
                        if d == 1 { "" } else { "s" },
                        c,
                        if c == 1 { "" } else { "s" }
                    ),
                }
            })
            .to_string();
        let text = RE_DECIMAL_NUMBER
            .replace_all(&text, |caps: &Captures| caps[1].replace(".", " point "))
            .to_string();
        let text = RE_ORDINAL
            .replace_all(&text, |caps: &Captures| {
                english_numbers::convert(
                    caps[1].trim().parse::<i64>().unwrap_or(0),
                    Formatting {
                        spaces: true,
                        conjunctions: true,
                        ..Default::default()
                    },
                )
            })
            .to_string();
        RE_NUMBER
            .replace_all(&text, |caps: &Captures| {
                let formatting = Formatting {
                    spaces: true,
                    conjunctions: true,
                    ..Default::default()
                };
                let number = caps
                    .get(0)
                    .and_then(|m| m.as_str().trim().parse::<i64>().ok())
                    .unwrap_or(0);

                match number {
                    2000 => "two thousand".to_string(),
                    2001..=2009 => format!(
                        "two thousand {}",
                        english_numbers::convert(number % 100, formatting)
                    ),
                    2010..=2999 if number % 100 == 0 => {
                        format!(
                            "{} hundred",
                            english_numbers::convert(number / 100, formatting)
                        )
                    }
                    1000..=2999 => english_numbers::convert(number, formatting).replace(", ", " "),
                    _ => english_numbers::convert(number, formatting),
                }
            })
            .to_string()
    }

    fn replace_phonemes(&self, phones: Vec<String>) -> Vec<String> {
        phones
            .into_iter()
            .filter_map(|ph| {
                if SYMBOLS.contains(&&*ph) {
                    Some(ph)
                } else {
                    self.rep_map.get(&ph).cloned()
                }
            })
            .collect()
    }

    pub fn g2p(&self, text: &str) -> Vec<String> {
        let words = self.split_with_delimiter(text);
        let mut phones = vec![];

        for mut w in words {
            if let Some(phns) = self.eng_dict.get(&w.to_uppercase()) {
                phones.extend(phns.iter().flat_map(|ph| ph.iter().cloned()));
            } else if !w.trim().is_empty() {
                println!("w 0: {}", w);
                // 去除首尾非字母数字字符，保留单独的符号
                if w.len() > 1 {
                    w = w
                        .trim_start_matches(|c: char| !c.is_alphanumeric())
                        .trim_end_matches(|c: char| !c.is_alphanumeric())
                        .to_string();
                }
                println!("w 1: {}", w);
                // 根据清理后的单词生成音素
                let phone_list = if w.chars().next().map_or(false, char::is_alphanumeric)
                    && w.chars().next_back().map_or(false, char::is_alphanumeric)
                {
                    self.pho_model.predict_phonemes_strs(&w).ok()
                } else {
                    Some(vec![w.as_str()])
                };

                if let Some(phone_list) = phone_list {
                    phones.extend(
                        phone_list
                            .into_iter()
                            .filter(|ph| !ph.is_empty())
                            .map(String::from),
                    );
                }
                println!("phones: {:?}", phones);
            }
        }

        self.replace_phonemes(phones)
    }

    fn split_with_delimiter(&self, input: &str) -> Vec<String> {
        let mut result = vec![];
        let mut last_end = 0;
        for cap in RE_DELIMITER.captures_iter(input) {
            if let Some(m) = cap.get(1) {
                let (start, end) = (m.start(), m.end());
                if last_end < start {
                    result.push(input[last_end..start].to_string());
                }
                result.push(m.as_str().to_string());
                last_end = end;
            }
        }
        if last_end < input.len() {
            result.push(input[last_end..].to_string());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_normalize() {
        let eng = English::new("../data/eng_dict.json", "../data/model.npz").unwrap();

        // 边界情况：空字符串
        assert_eq!(eng.text_normalize(""), "");

        // 单个数字
        assert_eq!(eng.text_normalize("123"), "one hundred and twenty three");

        // 数字带逗号
        assert_eq!(eng.text_normalize("2,500"), "twenty five hundred");

        // 数字带小数点
        assert_eq!(eng.text_normalize("25.3"), "twenty five point three");

        // 金额
        assert_eq!(eng.text_normalize("$0.001"), "zero dollars");
        assert_eq!(eng.text_normalize("$0.01"), "one cent");
        assert_eq!(eng.text_normalize("$0.15"), "fifteen cents");
        assert_eq!(eng.text_normalize("$2.15"), "two dollars, fifteen cents");
        assert_eq!(eng.text_normalize("$0.1"), "ten cents");
        assert_eq!(eng.text_normalize("$0.5"), "fifty cents");
        assert_eq!(eng.text_normalize("$1"), "one dollar");
        assert_eq!(eng.text_normalize("$1.01"), "one dollar, one cent");
        assert_eq!(eng.text_normalize("$1.5"), "one dollar, fifty cents");
        assert_eq!(eng.text_normalize("$2500"), "twenty five hundred dollars");
        assert_eq!(eng.text_normalize("£23"), "twenty three pounds");
        assert_eq!(eng.text_normalize("£9.99"), "nine point ninety nine pounds");

        // 序数词
        assert_eq!(eng.text_normalize("23rd"), "twenty three");

        // 温度数
        assert_eq!(eng.text_normalize("22.5°C"), "twenty two point five°C");
    }

    #[test]
    fn test_g2p() {
        let eng: English = English::new("../data/eng_dict.json", "../data/model.npz").unwrap();

        // 单词分词
        let phs = eng.g2p("hello world");
        assert_eq!(phs, vec!["HH", "AH0", "L", "OW1", "W", "ER1", "L", "D"]);

        // 空字符串
        let phs = eng.g2p("");
        assert!(phs.is_empty());

        // 含标点符号的句子
        let phs = eng.g2p("hello, world!");
        assert_eq!(
            phs,
            vec!["HH", "AH0", "L", "OW1", ",", "W", "ER1", "L", "D", "!"]
        );

        // 数字
        let phs = eng.g2p("123");
        assert_eq!(phs, vec!["EH1", "F", "Y", "UW0", "EY1", "D"]);

        // 混合输入
        let phs = eng.g2p("hello $25 world");
        assert_eq!(
            phs,
            vec!["HH", "AH0", "L", "OW1", "EH1", "F", "Y", "UW0", "W", "ER1", "L", "D"]
        );

        // 边界情况：只有标点符号
        let phs = eng.g2p(",,,!!!");
        assert_eq!(phs, vec![",", ",", ",", "!", "!", "!"]);

        // 特殊字符混合
        let phs = eng.g2p("a$12b,c?");
        assert_eq!(phs, vec!["EY1", "B", ",", "S", "IY1", "?"]);
    }

    #[test]
    fn test_edge_cases() {
        let eng = English::new("../data/eng_dict.json", "../data/model.npz").unwrap();

        // 单个字符
        assert_eq!(eng.text_normalize("a"), "a");

        // 大量标点符号
        let phs = eng.g2p("!!!,,,???");
        assert_eq!(phs, vec!["!", "!", "!", ",", ",", ",", "?", "?", "?"]);

        // 空格与混合输入
        let text = "hello   world  !";
        let phs = eng.g2p(text);
        assert_eq!(
            phs,
            vec!["HH", "AH0", "L", "OW1", "W", "ER1", "L", "D", "!"]
        );

        // 混合输入
        assert_eq!(
            eng.g2p("Hello, world! 123."),
            vec![
                "HH", "AH0", "L", "OW1", ",", "W", "ER1", "L", "D", "!", "W", "AH0", "N", "T",
                "UW1", "TH", "R", "IY0", "."
            ]
        );
        // 未知单词
        assert_eq!(eng.g2p("qwerty"), vec!["K", "W", "ER1", "T", "IY0"]); // 如果 `qwerty` 不在词典中，则返回原始单词
                                                                          // 混合大小写
        assert_eq!(
            eng.g2p("HeLLo WoRLD"),
            vec!["HH", "AH0", "L", "OW1", "W", "ER1", "L", "D"]
        );

        // 无效输入
        let text = " ";
        let phs = eng.g2p(text);
        assert!(phs.is_empty());
    }

    #[test]
    fn test_samples() {
        let eng = English::new("../data/eng_dict.json", "../data/model.npz").unwrap();

        let text = "Hello! Today is January 15th, 2025, and the time is 3:45 PM. The temperature is 22.5℃, and it feels like 20℃. You owe me $12.34, or £9.99, which you can pay by 6:00 AM tomorrow. Can you read this email address: test@example.com? What about this URL: https://www.openai.com? Finally, here's a math equation: 3.14 × 2 = 6.28, and a phone number: (123) 456-7890.";
        let phs = eng.g2p(text);
        assert_eq!(
            phs,
            vec![
                "HH", "AH0", "L", "OW1", "!", "T", "AH0", "D", "EY1", "IH1", "Z", "JH", "AE1", "N",
                "Y", "UW0", "EH2", "R", "IY0", "DH", "EY1", ",", "EH1", "F", "Y", "UW0", "Z", ",",
                "AE1", "N", "D", "DH", "AH0", "T", "AY1", "M", "IH1", "Z", "EH1", "F", "Y", "UW0",
                "Z", "P", "IY1", "EH1", "M", ".", "DH", "AH0", "T", "EH1", "M", "P", "R", "AH0",
                "CH", "ER0", "IH1", "Z", "EH1", "F", "Y", "UW0", ".", "EH1", "N", "AY1", "T",
                "UH0", "R", "AE1", "N", ",", "AE1", "N", "D", "IH1", "T", "F", "IY1", "L", "Z",
                "L", "AY1", "K", "EH1", "F", "Y", "UW0", ".", "Y", "UW1", "OW1", "M", "IY1", "EH1",
                "F", "Y", "UW0", ".", "EH1", "F", "Y", "UW0", ",", "AO1", "R", "EH1", "N", "AY1",
                "T", "UH0", "R", "AE1", "N", ".", "EH1", "F", "Y", "UW0", ",", "W", "IH1", "CH",
                "Y", "UW1", "K", "AE1", "N", "P", "EY1", "B", "AY1", "EH1", "F", "Y", "UW0", "Z",
                "AE1", "M", "T", "AH0", "M", "AA1", "R", "OW2", ".", "K", "AE1", "N", "Y", "UW1",
                "R", "EH1", "D", "DH", "IH1", "S", "IY0", "M", "EY1", "L", "AH0", "D", "R", "EH1",
                "S", "T", "EH1", "S", "T", "AH0", "S", "K", "AE2", "P", "AH0", "L", ".", "K",
                "AA1", "M", "?", "W", "AH1", "T", "AH0", "B", "AW1", "T", "DH", "IH1", "S", "EH1",
                "F", "Y", "UW0", "EY1", "D", "T", "AE1", "V", "P", "S", "W", "AA1", ".", "OW0",
                "P", "EY0", "N", "AA1", ".", "K", "AA1", "M", "?", "F", "AY1", "N", "AH0", "L",
                "IY0", ",", "HH", "IH1", "R", "Z", "AH0", "M", "AE1", "TH", "IH0", "K", "W", "EY1",
                "SH", "AH0", "N", "EH1", "N", "AY1", "T", "UH0", "R", "AE1", "N", ".", "EH1", "F",
                "Y", "UW0", "EH1", "N", "AY1", "T", "UH0", "R", "AE1", "N", "EH1", "N", "AY1", "T",
                "UH0", "R", "AE1", "N", ".", "EH1", "F", "Y", "UW0", ",", "AE1", "N", "D", "AH0",
                "F", "OW1", "N", "N", "AH1", "M", "B", "ER0", "EH1", "F", "Y", "UW0", "EY1", "D",
                "EH1", "F", "Y", "UW0", "EY1", "D", "-", "EH1", "F", "Y", "UW0", "Z", "."
            ]
        );
    }
}
