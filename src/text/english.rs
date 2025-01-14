use super::symbols::SYMBOLS;
use english_numbers::Formatting;
use fancy_regex::Regex as Regex2;
use grapheme_to_phoneme::Model;
use log::info;
use regex::{Captures, Regex};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct English {
    pub eng_dict: HashMap<String, Vec<Vec<String>>>,
    pub rep_map: HashMap<String, String>,
    pub pho_model: Model,
    pub _comma_number_re: Regex,
    pub _decimal_number_re: Regex,
    pub _pounds_re: Regex,
    pub _dollars_re: Regex,
    pub _ordinal_re: Regex,
    pub _number_re: Regex,
}

/// ,,hello, world -> ["", ",", "", ",", "hello", ",", "", " ", "world"]
pub fn split_with_delimiter(re: &Regex, input: &str) -> Vec<String> {
    let mut result = vec![];
    let mut before_idx = 0;
    for cap in re.captures_iter(input) {
        if let Some(matched) = cap.get(1) {
            let delimiter = matched.as_str();

            let start = matched.start();
            let end = matched.end();
            let before = &input[before_idx..start];
            result.push(before.to_string());
            result.push(delimiter.to_string());
            before_idx = end;
        } else {
            break;
        }
    }
    if before_idx <= input.as_bytes().len() {
        let before = std::str::from_utf8(&input.as_bytes()[before_idx..]).expect("Invalid UTF-8");
        result.push(before.to_string());
    }
    let re = Regex2::new(r"(?<!\b)([A-Z])").unwrap();

    let mut result2 = vec![];
    for res in result {
        if !res.is_empty() {
            if re.is_match(&res).unwrap() {
                let mut new_res: Vec<String> = re
                    .replace(&res, r" ${1}")
                    .split(" ")
                    .map(|x| x.to_string())
                    .collect();
                result2.append(&mut new_res);
            } else {
                result2.push(res);
            }
        }
    }
    result2
}

impl English {
    pub fn init(eng_dict_json_path: &str, ph_model_path: &str) -> Result<Self, String> {
        let file_op = fs::File::open(eng_dict_json_path);
        if file_op.is_err() {
            return Err(file_op.err().unwrap().to_string());
        }
        let file = file_op.unwrap();
        let eng_dict_op = serde_json::from_reader(&file);
        if eng_dict_op.is_err() {
            return Err(eng_dict_op.err().unwrap().to_string());
        }
        let eng_dict: HashMap<String, Vec<Vec<String>>> = eng_dict_op.unwrap();

        let pho_model_op = Model::load_from_npz_file(Path::new(ph_model_path));
        if pho_model_op.is_err() {
            return Err(pho_model_op.err().unwrap().to_string());
        }

        let pho_model = pho_model_op.unwrap();

        let _comma_number_re = Regex::new(r"([0-9][0-9\,]+[0-9])").unwrap();
        let _decimal_number_re = Regex::new(r"([0-9]+\.[0-9]+)").unwrap();
        let _pounds_re = Regex::new(r"£([0-9\,]*[0-9]+)").unwrap();
        let _dollars_re = Regex::new(r"\$([0-9\.\,]*[0-9]+)").unwrap();
        let _ordinal_re = Regex::new(r"([0-9]+)(st|nd|rd|th)").unwrap();
        let _number_re = Regex::new(r"[0-9]+").unwrap();

        let rep_map = HashMap::from([
            (";".to_string(), ",".to_string()),
            (":".to_string(), ",".to_string()),
            ("'".to_string(), "-".to_string()),
            ('"'.to_string(), "-".to_string()),
        ]);

        Ok(English {
            eng_dict,
            rep_map,
            pho_model,
            _comma_number_re,
            _decimal_number_re,
            _pounds_re,
            _dollars_re,
            _ordinal_re,
            _number_re,
        })
    }
    /// 2,50.1 -> 250.1
    fn _remove_commas(&self, value_string: String) -> String {
        let replacement = |caps: &Captures| -> String {
            let number = caps.get(1).map_or("", |m| m.as_str());

            number.replace(",", "")
        };

        let caps = self
            ._comma_number_re
            .replace_all(&value_string, replacement)
            .to_string();
        caps
    }

    /// £23 -> 23 pounds
    fn _pounds(&self, value_string: String) -> String {
        let replacement = |caps: &Captures| -> String {
            let number = caps.get(1).map_or("", |m| m.as_str());

            number.to_string() + " pounds"
        };

        let caps = self
            ._pounds_re
            .replace_all(&value_string, replacement)
            .to_string();
        caps
    }

    fn _expand_dollars(&self, value_string: String) -> String {
        let replacement = |caps: &Captures| -> String {
            let num_match = caps.get(1).map_or("", |m| m.as_str());
            let parts: Vec<&str> = num_match.split('.').collect();
            if parts.len() > 2 {
                return num_match.to_string() + " dollars";
            }

            let dollars = {
                if !parts.is_empty() && !parts[0].is_empty() {
                    let a = parts[0].parse::<i32>();
                    a.unwrap_or(0)
                } else {
                    0
                }
            };

            let cents = {
                if parts.len() >= 2 && !parts[1].is_empty() {
                    let a = parts[1].parse::<i32>();
                    a.unwrap_or(0)
                } else {
                    0
                }
            };
            if dollars != 0 && cents != 0 {
                let dollar_unit = {
                    if dollars == 1 {
                        "dollar"
                    } else {
                        "dollars"
                    }
                };
                let cent_unit = {
                    if cents == 1 {
                        "cent"
                    } else {
                        "cents"
                    }
                };
                format!("{} {}, {} {}", dollars, dollar_unit, cents, cent_unit)
            } else if dollars != 0 {
                let dollar_unit = {
                    if dollars == 1 {
                        "dollar"
                    } else {
                        "dollars"
                    }
                };
                let res = format!("{} {}", dollars, dollar_unit);
                return res;
            } else if cents != 0 {
                let cent_unit = {
                    if cents == 1 {
                        "cent"
                    } else {
                        "cents"
                    }
                };
                let res = format!("{} {}", cents, cent_unit);
                return res;
            } else {
                return "zero dollars".to_string();
            }
        };

        let caps = self
            ._dollars_re
            .replace_all(&value_string, replacement)
            .to_string();
        caps
    }

    // 23.3 -> 23 point 3
    fn _expand_decimal_point(&self, value_string: String) -> String {
        let replacement = |caps: &Captures| -> String {
            let number = caps.get(1).map_or("", |m| m.as_str());

            number.replace(".", " point ")
        };

        let caps = self
            ._decimal_number_re
            .replace_all(&value_string, replacement)
            .to_string();
        caps
    }

    // 23th -> twenty three
    fn _expand_ordinal(&self, value_string: String) -> String {
        let replacement = |caps: &Captures| -> String {
            let number = caps.get(1).map_or("", |m| m.as_str());
            let number_num_res = number.trim().parse::<i64>();
            if number_num_res.is_ok() {
                let number_num = number_num_res.unwrap();
                let number_words = english_numbers::convert(
                    number_num,
                    Formatting {
                        title_case: false,
                        spaces: true,
                        conjunctions: true,
                        commas: false,
                        dashes: false,
                    },
                );
                return number_words;
            }
            number.to_string()
        };

        let caps = self
            ._ordinal_re
            .replace_all(&value_string, replacement)
            .to_string();
        caps
    }

    /// 123_456_789 、 123456789 is different
    fn _expand_number(&self, value_string: String) -> String {
        let replacement = |caps: &regex::Captures| -> String {
            let number_res = caps
                .get(0)
                .map_or(Ok(0), |m| m.as_str().trim().parse::<i64>());
            let number = number_res.unwrap_or(0);

            let w = if number > 1000 && number < 3000 {
                if number == 2000 {
                    "two thousand".to_string()
                } else if number > 2000 && number < 2010 {
                    let w = english_numbers::convert(
                        number % 100,
                        english_numbers::Formatting {
                            title_case: false,
                            spaces: true,
                            conjunctions: true,
                            commas: false,
                            dashes: false,
                        },
                    );
                    "two thousand ".to_string() + &w
                } else if number % 100 == 0 {
                    let num = number / 100;
                    let mut w = english_numbers::convert(
                        num,
                        english_numbers::Formatting {
                            title_case: false,
                            spaces: true,
                            conjunctions: true,
                            commas: false,
                            dashes: false,
                        },
                    );
                    w += " hundred";
                    w
                } else {
                    let w = english_numbers::convert(
                        number,
                        english_numbers::Formatting {
                            title_case: false,
                            spaces: true,
                            conjunctions: false,
                            commas: false,
                            dashes: false,
                        },
                    );
                    w.replace(", ", " ")
                }
            } else {
                english_numbers::convert(
                    number,
                    english_numbers::Formatting {
                        title_case: false,
                        spaces: true,
                        conjunctions: false,
                        commas: false,
                        dashes: false,
                    },
                )
            };
            format!(" {} ", w)
        };

        let caps = self
            ._number_re
            .replace_all(&value_string, replacement)
            .to_string();
        caps
    }

    fn normalize_numbers(&self, text: String) -> String {
        let text = self._remove_commas(text);
        let text = self._pounds(text);
        let text = self._expand_dollars(text);
        let text = self._expand_decimal_point(text);
        let text = self._expand_ordinal(text);

        self._expand_number(text)
    }

    // 文本规范化
    pub fn text_normalize(&self, text: String) -> String {
        self.normalize_numbers(text)
    }

    // 确保只要在表里面的ph
    fn replace_phs(&self, phones: Vec<String>) -> Vec<String> {
        let mut phs_new: Vec<String> = vec![];
        for ph in phones {
            if SYMBOLS.contains(&&*ph) {
                phs_new.push(ph);
            } else if self.rep_map.contains_key(&ph) {
                phs_new.push(self.rep_map.get(&ph).unwrap().to_string());
            } else {
                info!("ph:{} not in symbols", ph);
            }
        }
        phs_new
    }

    pub fn g2p(&self, text: &str) -> Vec<String> {
        let mut phones: Vec<String> = vec![];
        // 中英文混合
        // let re = Regex::new(r"([,，；;.。？)）(（】\]\[【！\-\?\!\s+])").unwrap();
        let re = Regex::new(r"([,，；;.。？！\-\?\!\s+])").unwrap();
        let words = split_with_delimiter(&re, text);
        for mut w in words {
            if self.eng_dict.contains_key(&w.to_uppercase()) {
                let phns = self.eng_dict.get(&w.to_uppercase()).unwrap();
                for ph in phns {
                    for pi in ph {
                        phones.push(pi.clone());
                    }
                }
            } else {
                // 空的一定跳过
                if w.trim() != "" {
                    // num or a-Z
                    let mut w_len = w.chars().count();
                    // 防止前面没有正常 norm English ，否则会报错: 首位不是正常字母数字，则要移除
                    if w_len > 1 && !w.chars().nth(0).unwrap().is_alphanumeric() {
                        w = w[1..w_len].to_string();
                        w_len = w.chars().count();
                    }
                    if w_len > 1 && !w.chars().nth(w_len - 1).unwrap().is_alphanumeric() {
                        w = w[0..w_len - 1].to_string();
                        w_len = w.chars().count();
                    }
                    let phone_list = if w_len > 0
                        && w.chars().nth(0).unwrap().is_alphanumeric()
                        && w.chars().nth(w_len - 1).unwrap().is_alphanumeric()
                    {
                        self.pho_model.predict_phonemes_strs(&w).ok()
                    } else {
                        Some(vec![w.as_str()])
                    };
                    if let Some(phone_list) = phone_list {
                        for ph in phone_list {
                            if !ph.is_empty() {
                                phones.push(ph.to_string());
                            }
                        }
                    }
                }
            }
        }

        self.replace_phs(phones)
    }
}

#[test]
fn test0() {
    let num = English::init("../data/eng_dict.json", "../data/model.npz").unwrap();
    let text = num.text_normalize(",,$25 hello, world".to_string());
    let new_phs = num.g2p(&text);
    println!("{:?}", new_phs);
}
