use super::mmseg::MMSeg;
use super::style::{convert_styles, get_finals, get_initials, Style};
use lazy_static::lazy_static;
use pinyin::ToPinyin;
use regex::Regex;
use std::collections::HashMap;

lazy_static! {
    static ref RE_HANS: Regex = Regex::new(r"^(?:[\u3007\ue815-\ue864\ufa18\u3400-\u4dbf\u4e00-\u9fff\uf900-\ufaff\\U00020000-\\U0002A6DF\\U0002A703-\\U0002B73F\\U0002B740-\\U0002B81D\\U0002B825-\\U0002BF6E\\U0002C029-\\U0002CE93\\U0002D016\\U0002D11B-\\U0002EBD9\\U0002F80A-\\U0002FA1F\\U000300F7-\\U00031288\\U00030EDD\\U00030EDE'])+$").unwrap();
}
pub struct LazyPinyin {
    mmseg: MMSeg,
    phrases_dict: HashMap<String, Vec<Vec<String>>>,
    pinyin_dict: HashMap<String, String>,
}

impl LazyPinyin {
    pub fn new(phrases_dict_path: &str, pinyin_dict_path: &str) -> Result<Self, String> {
        let phrases_dict: HashMap<String, Vec<Vec<String>>> = serde_json::from_reader(
            &std::fs::File::open(phrases_dict_path).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let pinyin_dict: HashMap<String, String> = serde_json::from_reader(
            &std::fs::File::open(pinyin_dict_path).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let mmseg = MMSeg::new(true, &phrases_dict);

        Ok(Self {
            mmseg,
            phrases_dict,
            pinyin_dict,
        })
    }

    pub fn lazy_pinyin(&self, hans: &str, style: Style, strict: bool) -> Vec<Vec<String>> {
        let han_list = self.seg(hans, &self.phrases_dict);
        let mut pys = Vec::new();
        for words in han_list {
            let mut p = self.convert(&words, style, strict);
            if p.is_empty() {
                let (initials, finals) = words
                    .as_str()
                    .to_pinyin()
                    .enumerate()
                    .map(|(i, p)| {
                        if let Some(p) = p {
                            let pn = p.with_tone_num();
                            let py = get_initials(pn, true);
                            let py2 = get_finals(pn, true);
                            let (non_num, num): (String, String) =
                                py2.chars().partition(|c| !c.is_ascii_digit());
                            let py2 = format!(
                                "{}{}",
                                non_num,
                                if num.trim().is_empty() { "5" } else { &num }
                            );
                            (py.to_string(), py2.to_string())
                        } else {
                            let wc = words.chars().nth(i).unwrap();
                            (wc.to_string(), wc.to_string())
                        }
                    })
                    .unzip();
                if style == Style::Initials {
                    p.push(initials);
                } else if style == Style::InitialsTone3 {
                    p.push(finals);
                }
            }
            pys.append(&mut p);
        }
        pys
    }

    /// 根据参数把汉字转成相应风格的拼音结果。
    //
    //         :param words: 汉字字符串
    //         :type words: unicode
    //         :param style: 拼音风格
    //         :param heteronym: 是否启用多音字
    //         :type heteronym: bool
    //         :param errors: 如果处理没有拼音的字符
    //         :param strict: 只获取声母或只获取韵母相关拼音风格的返回结果
    //                        是否严格遵照《汉语拼音方案》来处理声母和韵母，
    //                        详见 :ref:`strict`
    //         :type strict: bool
    //         :return: 按风格转换后的拼音结果
    //         :rtype: list
    fn convert(&self, words: &str, style: Style, strict: bool) -> Vec<Vec<String>> {
        let mut pys = if RE_HANS.is_match(words) {
            self.phrase_pinyin(words, style)
        } else {
            Vec::new()
        };
        pys = convert_styles(pys, words, style, strict);
        pys.into_iter()
            .map(|lst| {
                let lst = lst
                    .into_iter()
                    .filter(|item| !item.is_empty())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                if lst.is_empty() {
                    vec!["".to_string()]
                } else {
                    lst
                }
            })
            .collect()
    }

    fn phrase_pinyin(&self, phrase: &str, style: Style) -> Vec<Vec<String>> {
        if let Some(py) = self.phrases_dict.get(phrase).cloned() {
            py
        } else {
            phrase
                .chars()
                .flat_map(|han| self.single_pinyin(&han.to_string(), style))
                .collect()
        }
    }

    fn single_pinyin(&self, han: &String, _style: Style) -> Vec<Vec<String>> {
        if let Some(pys) = self.pinyin_dict.get(han) {
            vec![pys.split(",").map(|x| x.to_string()).collect()]
        } else {
            Vec::new()
        }
    }

    fn seg(&self, hans: &str, phrases_dict: &HashMap<String, Vec<Vec<String>>>) -> Vec<String> {
        self.mmseg.seg(hans, phrases_dict)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_pinyin_basic() {
        let m =
            LazyPinyin::new("../assets/PHRASES_DICT.json", "../assets/PINYIN_DICT.json").unwrap();
        let result = m.lazy_pinyin("你一走，我就好害怕", Style::Initials, true);
        assert_eq!(
            result,
            vec![["n"], [""], ["z"], ["，"], [""], ["j"], ["h"], ["h"], ["p"]]
        );
    }

    #[test]
    fn test_lazy_pinyin_basic_false() {
        let m =
            LazyPinyin::new("../assets/PHRASES_DICT.json", "../assets/PINYIN_DICT.json").unwrap();
        let result = m.lazy_pinyin("你一走，我就好害怕", Style::Initials, false);
        assert_eq!(
            result,
            vec![
                ["n"],
                ["y"],
                ["z"],
                ["，"],
                ["w"],
                ["j"],
                ["h"],
                ["h"],
                ["p"]
            ]
        );
    }

    #[test]
    fn test_lazy_pinyin_basic3() {
        let m =
            LazyPinyin::new("../assets/PHRASES_DICT.json", "../assets/PINYIN_DICT.json").unwrap();
        let result = m.lazy_pinyin("你好", Style::InitialsTone3, true);
        assert_eq!(result, vec![["i3"], ["ao3"]]);
    }

    #[test]
    fn test_lazy_pinyin_empty_string() {
        let m =
            LazyPinyin::new("../assets/PHRASES_DICT.json", "../assets/PINYIN_DICT.json").unwrap();
        let result = m.lazy_pinyin("", Style::InitialsTone3, true);
        assert_eq!(result, Vec::<Vec<String>>::new());
    }

    #[test]
    fn test_lazy_pinyin_single_character() {
        let m =
            LazyPinyin::new("../assets/PHRASES_DICT.json", "../assets/PINYIN_DICT.json").unwrap();
        let result = m.lazy_pinyin("你", Style::InitialsTone3, true);
        assert_eq!(result, vec![["i3"]]);
    }

    #[test]
    fn test_lazy_pinyin_multiple_characters() {
        let m =
            LazyPinyin::new("../assets/PHRASES_DICT.json", "../assets/PINYIN_DICT.json").unwrap();
        let result = m.lazy_pinyin("你好世界", Style::InitialsTone3, true);
        assert_eq!(result, vec![["i3"], ["ao3"], ["i4"], ["ie4"]]);
    }

    #[test]
    fn test_lazy_pinyin_multiple_inv_characters() {
        let m =
            LazyPinyin::new("../assets/PHRASES_DICT.json", "../assets/PINYIN_DICT.json").unwrap();
        let result = m.lazy_pinyin("你!好@世#界", Style::InitialsTone3, true);
        assert_eq!(
            result,
            vec![["i3"], ["!"], ["ao3"], ["@"], ["i4"], ["#"], ["ie4"]]
        );
    }

    #[test]
    fn test_lazy_pinyin_multiple_inv_false_characters() {
        let m =
            LazyPinyin::new("../assets/PHRASES_DICT.json", "../assets/PINYIN_DICT.json").unwrap();
        let result = m.lazy_pinyin("你!好@世#界", Style::InitialsTone3, false);
        assert_eq!(
            result,
            vec![["i3"], ["!"], ["ao3"], ["@"], ["i4"], ["#"], ["ie4"]]
        );
    }

    #[test]
    fn test_lazy_pinyin_non_hanzi_characters() {
        let m =
            LazyPinyin::new("../assets/PHRASES_DICT.json", "../assets/PINYIN_DICT.json").unwrap();
        let result = m.lazy_pinyin("Hello!", Style::InitialsTone3, true);
        assert_eq!(result, vec![["H"], ["e"], ["l"], ["l"], ["o"], ["!"]]);
    }

    #[test]
    fn test_lazy_pinyin_with_multi_pronunciations() {
        let m =
            LazyPinyin::new("../assets/PHRASES_DICT.json", "../assets/PINYIN_DICT.json").unwrap();
        let result = m.lazy_pinyin("重", Style::InitialsTone3, true);
        assert_eq!(result, vec![["ong4"]]);
    }

    #[test]
    fn test_lazy_pinyin_with_mixed_input() {
        let m =
            LazyPinyin::new("../assets/PHRASES_DICT.json", "../assets/PINYIN_DICT.json").unwrap();
        let result = m.lazy_pinyin("你好123", Style::InitialsTone3, true);
        assert_eq!(result, vec![["i3"], ["ao3"], ["1"], ["2"], ["3"]]);
    }
}
