use super::convert::{convert_finals, FINALS};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use substring::Substring;

lazy_static! {
    static ref PHONETIC_SYMBOL_DICT: HashMap<&'static str, &'static str> = {
        HashMap::from([
            ("ā", "a1"),
            ("á", "a2"),
            ("ǎ", "a3"),
            ("à", "a4"),
            ("ē", "e1"),
            ("é", "e2"),
            ("ě", "e3"),
            ("è", "e4"),
            ("ō", "o1"),
            ("ó", "o2"),
            ("ǒ", "o3"),
            ("ò", "o4"),
            ("ī", "i1"),
            ("í", "i2"),
            ("ǐ", "i3"),
            ("ì", "i4"),
            ("ū", "u1"),
            ("ú", "u2"),
            ("ǔ", "u3"),
            ("ù", "u4"),
            ("ü", "v"),
            ("ǖ", "v1"),
            ("ǘ", "v2"),
            ("ǚ", "v3"),
            ("ǜ", "v4"),
            ("ń", "n2"),
            ("ň", "n3"),
            ("ǹ", "n4"),
            ("m̄", "m1"),
            ("ḿ", "m2"),
            ("m̀", "m4"),
            ("ê̄", "ê1"),
            ("ế", "ê2"),
            ("ê̌", "ê3"),
            ("ề", "ê4"),
        ])
    };
    static ref PHONETIC_SYMBOL_DICT_KEY_LENGTH_NOT_ONE: HashMap<&'static str, &'static str> =
        HashMap::from([("m̄", "m1"), ("m̀", "m4"), ("ê̄", "ê1"), ("ê̌", "ê3")]);
    static ref RE_PHONETIC_SYMBOL: Regex =
        Regex::new(r"[āáǎàēéěèōóǒòīíǐìūúǔùüǖǘǚǜńňǹḿếề]").unwrap();
    static ref RE_NUMBER: Regex = Regex::new(r"\d").unwrap();
    static ref INITIALS: [&'static str; 21] = {
        [
            "b", "p", "m", "f", "d", "t", "n", "l", "g", "k", "h", "j", "q", "x", "zh", "ch", "sh",
            "r", "z", "c", "s",
        ]
    };
    static ref INITIALS_NOT_STRICT: [&'static str; 23] = {
        [
            "b", "p", "m", "f", "d", "t", "n", "l", "g", "k", "h", "j", "q", "x", "zh", "ch", "sh",
            "r", "z", "c", "s", "y", "w",
        ]
    };
}

#[derive(Clone, Copy, PartialEq)]
pub enum Style {
    // 拼音风格

    //: 普通风格，不带声调。如： 中国 -> ``zhong guo``
    // NORMAL = 0,
    //: 标准声调风格，拼音声调在韵母第一个字母上（默认风格）。如： 中国 -> ``zhōng guó``
    // TONE = 1,
    //: 声调风格2，即拼音声调在各个韵母之后，用数字 [1-4] 进行表示。如： 中国 -> ``zho1ng guo2``
    // TONE2 = 2,
    //: 声调风格3，即拼音声调在各个拼音之后，用数字 [1-4] 进行表示。如： 中国 -> ``zhong1 guo2``
    Tone3 = 8,
    //: 声母风格，只返回各个拼音的声母部分（注：有的拼音没有声母，详见 `//27`_）。如： 中国 -> ``zh g``
    Initials = 3,
    //: 首字母风格，只返回拼音的首字母部分。如： 中国 -> ``z g``
    // FIRST_LETTER = 4,
    //: 韵母风格，只返回各个拼音的韵母部分，不带声调。如： 中国 -> ``ong uo``
    // FINALS = 5,
    //: 标准韵母风格，带声调，声调在韵母第一个字母上。如：中国 -> ``ōng uó``
    // FINALS_TONE = 6,
    //: 韵母风格2，带声调，声调在各个韵母之后，用数字 [1-4] 进行表示。如： 中国 -> ``o1ng uo2``
    // FINALS_TONE2 = 7,
    //: 韵母风格3，带声调，声调在各个拼音之后，用数字 [1-4] 进行表示。如： 中国 -> ``ong1 uo2``
    InitialsTone3 = 9,
    //: 注音风格，带声调，阴平（第一声）不标。如： 中国 -> ``ㄓㄨㄥ ㄍㄨㄛˊ``
    // BOPOMOFO = 10,
    //: 注音风格，仅首字母。如： 中国 -> ``ㄓ ㄍ``
    // BOPOMOFO_FIRST = 11,
    //: 汉语拼音与俄语字母对照风格，声调在各个拼音之后，用数字 [1-4] 进行表示。如： 中国 -> ``чжун1 го2``
    // CYRILLIC = 12,
    //: 汉语拼音与俄语字母对照风格，仅首字母。如： 中国 -> ``ч г``
    // CYRILLIC_FIRST = 13,
    //: 威妥玛拼音/韦氏拼音/威式拼音风格，无声调
    // WADEGILES = 14,
}

/// 把声调替换为数字
fn replace_symbol_to_number(pinyin: &str) -> String {
    let mut value = pinyin.to_string();
    for (symbol, to) in PHONETIC_SYMBOL_DICT.iter() {
        value = value.replace(symbol, to);
    }
    for (symbol, to) in PHONETIC_SYMBOL_DICT_KEY_LENGTH_NOT_ONE.iter() {
        value = value.replace(symbol, to);
    }
    value
}

fn replace_symbol_to_no_symbol(pinyin: &str) -> String {
    RE_NUMBER
        .replace_all(&replace_symbol_to_number(pinyin), "")
        .to_string()
}

/// 获取单个拼音中的声母.
//
//     :param pinyin: 单个拼音
//     :type pinyin: unicode
//     :param strict: 是否严格遵照《汉语拼音方案》来处理声母和韵母
//     :return: 声母
//     :rtype: unicode
pub fn get_initials(pinyin: &str, strict: bool) -> String {
    let initials = match strict {
        true => INITIALS.to_vec(),
        false => INITIALS_NOT_STRICT.to_vec(),
    };
    for i in initials {
        if pinyin.starts_with(i) {
            return i.to_string();
        }
    }
    "".to_string()
}

pub fn get_finals(pinyin: &str, strict: bool) -> String {
    let mut pinyin = pinyin.to_string();
    if strict {
        pinyin = convert_finals(&pinyin);
    }
    let mut initials = get_initials(&pinyin, strict);
    let mut finals = pinyin.substring(initials.chars().count(), pinyin.chars().count());
    if strict && !FINALS.contains(finals) {
        initials = get_initials(&pinyin, false);
        finals = pinyin.substring(initials.chars().count(), pinyin.chars().count());
        if FINALS.contains(finals) {
            return finals.to_string();
        }
        return "".to_string();
    }
    if finals.is_empty() && !strict {
        return pinyin;
    }
    finals.to_string()
}

pub fn to_finals(pinyin: &str, strict: bool, v_to_u: bool) -> String {
    let new_pinyin = replace_symbol_to_no_symbol(pinyin).replace("v", "ü");
    let finals = get_finals(&new_pinyin, strict);

    if v_to_u {
        finals.replace("v", "ü")
    } else {
        finals.replace("ü", "v")
    }
}

/// 将 :py:attr:`~pypinyin.Style.TONE`、
//     :py:attr:`~pypinyin.Style.TONE2` 或
//     :py:attr:`~pypinyin.Style.Tone3` 风格的拼音转换为
//     :py:attr:`~pypinyin.Style.InitialsTone3` 风格的拼音
//
//     :param pinyin: :py:attr:`~pypinyin.Style.TONE`、
//                    :py:attr:`~pypinyin.Style.TONE2` 或
//                    :py:attr:`~pypinyin.Style.Tone3` 风格的拼音
//     :param strict: 返回结果是否严格遵照《汉语拼音方案》来处理声母和韵母，
//                    详见 :ref:`strict`
//     :param v_to_u: 是否使用 ``ü`` 代替原来的 ``v``，
//                    当为 False 时结果中将使用 ``v`` 表示 ``ü``
//     :param neutral_tone_with_five: 是否使用 ``5`` 标识轻声
//     :return: :py:attr:`~pypinyin.Style.InitialsTone3` 风格的拼音
pub fn to_initials_tone3(
    pinyin: &str,
    strict: bool,
    v_to_u: bool,
    neutral_tone_with_five: bool,
) -> String {
    let pinyin = pinyin.replace("5", "");
    let mut finals = to_finals(&pinyin, strict, v_to_u);
    if finals.is_empty() {
        return finals;
    }

    let pinyin_with_num = replace_symbol_to_number(&pinyin);

    let numbers: Vec<&str> = RE_NUMBER
        .find_iter(&pinyin_with_num)
        .map(|m| m.as_str())
        .collect();
    if numbers.is_empty() {
        if neutral_tone_with_five {
            finals.push('5');
        } else {
            return finals;
        }
    } else {
        finals.push_str(numbers[0]);
    }

    finals
}

fn post_convert_style(
    converted_pinyin: &str,
    style: Style,
    neutral_tone_with_five: bool,
) -> String {
    if (style == Style::Tone3 || style == Style::InitialsTone3)
        && neutral_tone_with_five
        && !RE_NUMBER.is_match(converted_pinyin)
    {
        format!("{}5", converted_pinyin)
    } else {
        converted_pinyin.to_string()
    }
}

fn convert_style(orig_pinyin: &str, style: Style, strict: bool) -> String {
    let converted_pinyin = match style {
        Style::InitialsTone3 => to_initials_tone3(orig_pinyin, strict, false, false),
        Style::Initials => get_initials(orig_pinyin, strict),
        _ => "".to_string(),
    };
    post_convert_style(&converted_pinyin, style, true)
}

pub fn convert_styles(
    pinyin_list: Vec<Vec<String>>,
    _phrase: &str,
    style: Style,
    strict: bool,
) -> Vec<Vec<String>> {
    pinyin_list
        .into_iter()
        .map(|item| {
            let orig_pinyin = &item[0];
            vec![convert_style(orig_pinyin, style, strict)]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_symbol_to_number() {
        assert_eq!(replace_symbol_to_number("zhōng"), "zho1ng");
        assert_eq!(replace_symbol_to_number("guó"), "guo2");
        assert_eq!(replace_symbol_to_number("wǒ"), "wo3");
        assert_eq!(replace_symbol_to_number("m̀"), "m4");
        assert_eq!(replace_symbol_to_number("ê̄"), "ê1");
        assert_eq!(replace_symbol_to_number(""), "");
        assert_eq!(replace_symbol_to_number("zhōng guó"), "zho1ng guo2");
    }

    #[test]
    fn test_replace_symbol_to_no_symbol() {
        assert_eq!(replace_symbol_to_no_symbol("zhōng"), "zhong");
        assert_eq!(replace_symbol_to_no_symbol("guó"), "guo");
        assert_eq!(replace_symbol_to_no_symbol("zhōng5"), "zhong");
        assert_eq!(replace_symbol_to_no_symbol("m̀"), "m");
    }

    #[test]
    fn test_get_initials() {
        assert_eq!(get_initials("zhong", true), "zh");
        assert_eq!(get_initials("guo", true), "g");
        assert_eq!(get_initials("ying", false), "y");
        assert_eq!(get_initials("ying", true), "");
        assert_eq!(get_initials("wu", false), "w");
        assert_eq!(get_initials("wu", true), "");
        assert_eq!(get_initials("a", true), ""); // 没有声母
    }

    #[test]
    fn test_get_finals() {
        assert_eq!(get_finals("zhong", true), "ong");
        assert_eq!(get_finals("guo", true), "uo");
        assert_eq!(get_finals("ying", false), "ing");
        assert_eq!(get_finals("wu", true), "u");
        assert_eq!(get_finals("a", true), "a"); // 纯韵母
    }

    #[test]
    fn test_to_finals() {
        assert_eq!(to_finals("zhōng", true, true), "ong");
        assert_eq!(to_finals("guó", true, true), "uo");
        assert_eq!(to_finals("ying", false, true), "ing");
        assert_eq!(to_finals("wu", true, true), "u");
        assert_eq!(to_finals("a", true, true), "a");
        assert_eq!(to_finals("xǘe", true, true), "üe");
        assert_eq!(to_finals("xǘe", true, false), "ve");
    }

    #[test]
    fn test_to_initials_tone3() {
        assert_eq!(to_initials_tone3("zhōng", true, true, true), "ong1");
        assert_eq!(to_initials_tone3("guó", true, true, false), "uo2");
        assert_eq!(to_initials_tone3("wǒ", false, false, true), "o3");
        assert_eq!(to_initials_tone3("m̀", true, false, true), "");
    }

    #[test]
    fn test_convert_style() {
        assert_eq!(convert_style("zhōng", Style::InitialsTone3, true), "ong1");
        assert_eq!(convert_style("guó", Style::InitialsTone3, false), "uo2");
        assert_eq!(convert_style("wǒ", Style::Initials, true), "");
    }

    #[test]
    fn test_convert_styles() {
        let pinyin_list = vec![vec!["zhōng".to_string()], vec!["guó".to_string()]];
        let result = convert_styles(pinyin_list.clone(), "中国", Style::InitialsTone3, true);
        assert_eq!(result, vec![["ong1"], ["uo2"]]);

        let result = convert_styles(pinyin_list.clone(), "中国", Style::Initials, true);
        assert_eq!(result, vec![["zh"], ["g"]]);
    }

    #[test]
    fn test_post_convert_style() {
        assert_eq!(post_convert_style("zhong", Style::Tone3, true), "zhong5");
        assert_eq!(
            post_convert_style("guo", Style::InitialsTone3, true),
            "guo5"
        );
        assert_eq!(
            post_convert_style("zhong1", Style::InitialsTone3, true),
            "zhong1"
        );
    }
}
