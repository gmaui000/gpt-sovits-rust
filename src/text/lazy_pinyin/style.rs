use super::convert::{convert_finals, _FINALS};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
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
    let replacement = |caps: &Captures| -> &str {
        let symbol = &caps[0];
        PHONETIC_SYMBOL_DICT.get(symbol).unwrap()
    };
    // 返回使用数字标识声调的字符
    let mut value = RE_PHONETIC_SYMBOL
        .replace_all(pinyin, &replacement)
        .to_string();
    for (&symbol, &to) in PHONETIC_SYMBOL_DICT_KEY_LENGTH_NOT_ONE.iter() {
        value = value.replace(symbol, to);
    }
    value
}

fn replace_symbol_to_no_symbol(pinyin: &str) -> String {
    let value = replace_symbol_to_number(pinyin);
    let replacement = |_: &Captures| -> &str {
        // let b = &caps[0];
        ""
    };
    let value = RE_NUMBER.replace_all(&value, &replacement).to_string();
    value
}

/// 获取单个拼音中的声母.
//
//     :param pinyin: 单个拼音
//     :type pinyin: unicode
//     :param strict: 是否严格遵照《汉语拼音方案》来处理声母和韵母
//     :return: 声母
//     :rtype: unicode
pub fn get_initials(pinyin: &str, strict: bool) -> String {
    let initials = {
        match strict {
            true => INITIALS.to_vec(),
            false => INITIALS_NOT_STRICT.to_vec(),
        }
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
    if strict && !_FINALS.contains(finals) {
        initials = get_initials(&pinyin, false);
        finals = pinyin.substring(initials.chars().count(), pinyin.chars().count());
        if _FINALS.contains(finals) {
            return finals.to_string();
        }
        return "".to_string();
    }
    if finals.is_empty() && !strict {
        return pinyin;
    }
    finals.to_string()
}

fn _v_to_u(pinyin: &str, replace: bool) -> String {
    if !replace {
        return pinyin.to_string();
    }
    pinyin.replace("v", "ü")
}

fn _fix_v_u(_: &str, new_py: &str, v_to_u: bool) -> String {
    if !v_to_u {
        return new_py.replace("ü", "v");
    }
    _v_to_u(new_py, true)
}

pub fn to_finals(pinyin: &str, strict: bool, v_to_u: bool) -> String {
    let new_pinyin = replace_symbol_to_no_symbol(pinyin).replace("v", "ü");
    let finals = get_finals(&new_pinyin, strict);

    _fix_v_u(&finals, &finals, v_to_u)
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

    let mut numbers: Vec<&str> = RE_NUMBER
        .find_iter(&pinyin_with_num)
        .map(|m| m.as_str())
        .collect();
    if numbers.is_empty() {
        if neutral_tone_with_five {
            numbers = vec!["5"];
        } else {
            return finals;
        }
    }

    let number = numbers[0];
    finals += number;
    finals
}

fn right_mark_index(pinyin_no_tone: &str) -> usize {
    if pinyin_no_tone.contains("iou") {
        let idx = pinyin_no_tone.find("u").unwrap();
        return idx;
    }
    if pinyin_no_tone.contains("uei") {
        let idx = pinyin_no_tone.find("i").unwrap();
        return idx;
    }
    if pinyin_no_tone.contains("uen") {
        let idx = pinyin_no_tone.find("u").unwrap();
        return idx;
    }
    // 有 ɑ 不放过, 没 ɑ 找 o、e
    for c in ["a", "o", "e"] {
        if pinyin_no_tone.contains(c) {
            let idxc = pinyin_no_tone.find(c).unwrap();
            return idxc + c.chars().count() - 1;
        }
    }
    // i、u 若是连在一起，谁在后面就标谁
    for c in ["a", "o", "e"] {
        if pinyin_no_tone.contains(c) {
            let idxc = pinyin_no_tone.find(c).unwrap();
            return idxc + c.chars().count() - 1;
        }
    }

    // ɑ、o、e、i、u、ü
    for c in ["i", "u", "v", "ü"] {
        if pinyin_no_tone.contains(c) {
            let idxc = pinyin_no_tone.find(c).unwrap();
            return idxc + c.chars().count() - 1;
        }
    }
    // n, m, ê
    for c in ["n", "m", "ê"] {
        if pinyin_no_tone.contains(c) {
            let idxc = pinyin_no_tone.find(c).unwrap();
            return idxc + c.chars().count() - 1;
        }
    }

    0
}

fn post_convert_style(
    converted_pinyin: &str,
    style: Style,
    _neutral_tone_with_five: bool,
) -> String {
    if style == Style::Tone3 || style == Style::InitialsTone3 {
    } else {
        return converted_pinyin.to_string();
    }
    // 是否开启 5声轻声
    if _neutral_tone_with_five {
        // 有声调，跳过
        if RE_NUMBER.is_match(converted_pinyin) {
            return converted_pinyin.to_string();
        }
        if style == Style::Tone3 || style == Style::InitialsTone3 {
            let v = format!("{}5", converted_pinyin);
            return v;
        }
        // 找到应该在哪个字母上标声调
        let mark_index = right_mark_index(converted_pinyin);
        let before = converted_pinyin.substring(0, mark_index + 1);
        let after = converted_pinyin.substring(mark_index + 1, converted_pinyin.chars().count());
        let v = format!("{}5{}", before, after);
        return v;
    }

    converted_pinyin.to_string()
}

fn convert_style(orig_pinyin: &str, style: Style, strict: bool) -> String {
    let mut converted_pinyin = "".to_string();
    if style == Style::InitialsTone3 {
        converted_pinyin = to_initials_tone3(orig_pinyin, strict, false, false);
    } else if style == Style::Initials {
        converted_pinyin = get_initials(orig_pinyin, strict);
    }
    let mut post_data = post_convert_style(&converted_pinyin, style, true);
    if post_data.is_empty() {
        post_data = converted_pinyin;
    }
    post_data
}

pub fn convert_styles(
    pinyin_list: Vec<Vec<String>>,
    _phrase: &str,
    style: Style,
    strict: bool,
) -> Vec<Vec<String>> {
    let mut pinyin_list = pinyin_list;
    let p_len = pinyin_list.len();
    (0..p_len).for_each(|idx| {
        let item = &pinyin_list[idx];
        // let han = phrase.chars().nth(idx).unwrap();
        let orig_pinyin = &item[0];
        let p_list = vec![convert_style(orig_pinyin, style, strict)];
        pinyin_list[idx] = p_list;
    });

    pinyin_list
}
