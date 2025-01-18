use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::{HashMap, HashSet};

lazy_static! {
    static ref U_TONES: HashSet<char> = "ūuǔúù".chars().collect();
    static ref I_TONES: HashSet<char> = "iǐíīì".chars().collect();
    pub static ref FINALS: HashSet<&'static str> = vec![
        "i", "u", "ü", "a", "ia", "ua", "o", "uo", "e", "ie", "üe", "ai", "uai", "ei", "uei", "ao",
        "iao", "ou", "iou", "an", "ian", "uan", "üan", "en", "in", "uen", "ün", "ang", "iang",
        "uang", "eng", "ing", "ueng", "ong", "iong", "er", "ê"
    ]
    .into_iter()
    .collect();
    static ref UV_RE: Regex = Regex::new(r"^(j|q|x)(u|ū|ú|ǔ|ù)(.*)$").unwrap();
    static ref IU_RE: Regex = Regex::new(r"^([a-z]+)(iǔ|iū|iu|iù|iú)$").unwrap();
    static ref UI_RE: Regex = Regex::new(r"([a-z]+)(ui|uí|uì|uǐ|uī)$").unwrap();
    static ref UN_RE: Regex = Regex::new(r"([a-z]+)(ǔn|ún|ùn|un|ūn)$").unwrap();
    // iu -> iou
    static ref IU_MAP: HashMap<&'static str, &'static str> = vec![
        ("iu", "iou"),
        ("iū", "ioū"),
        ("iú", "ioú"),
        ("iǔ", "ioǔ"),
        ("iù", "ioù")
    ]
    .into_iter()
    .collect();
    // ui -> uei
    static ref UI_MAP: HashMap<&'static str, &'static str> = vec![
        ("ui", "uei"),
        ("uī", "ueī"),
        ("uí", "ueí"),
        ("uǐ", "ueǐ"),
        ("uì", "ueì")
    ]
    .into_iter()
    .collect();
    // un -> uen
    static ref UN_MAP: HashMap<&'static str, &'static str> = vec![
        ("un", "uen"),
        ("ūn", "ūen"),
        ("ún", "úen"),
        ("ǔn", "ǔen"),
        ("ùn", "ùen"),
    ]
    .into_iter()
    .collect();
    // u -> ü
    static ref UV_MAP: HashMap<&'static str, &'static str> =
        vec![("u", "ü"), ("ū", "ǖ"), ("ú", "ǘ"), ("ǔ", "ǚ"), ("ù", "ǜ")]
            .into_iter()
            .collect();
}

/// iou 转换，还原原始的韵母
//
//     iou，uei，uen前面加声母的时候，写成iu，ui，un。
//     例如niu(牛)，gui(归)，lun(论)。
/// uei 转换，还原原始的韵母
/// uen 转换，还原原始的韵母
/// ü 转换，还原原始的韵母
//     ü行的韵跟声母j，q，x拼的时候，写成ju(居)，qu(区)，xu(虚)，
//     ü上两点也省略；但是跟声母n，l拼的时候，仍然写成nü(女)，lü(吕)。
fn replace_with_map(re: &Regex, map: &HashMap<&'static str, &'static str>, input: &str) -> String {
    re.replace_all(input, |caps: &Captures| {
        let mut result = String::new();
        let m1 = &caps[1];
        let m2 = &caps[2];
        let replacement = map.get(m2).unwrap();
        result.push_str(m1);
        result.push_str(replacement);
        if caps.len() > 3 {
            let m3 = &caps[3];
            result.push_str(m3);
        }
        result
    })
    .to_string()
}

/// 零声母转换，还原原始的韵母
//
//     i行的韵母，前面没有声母的时候，写成yi(衣)，ya(呀)，ye(耶)，yao(腰)，
//     you(忧)，yan(烟)，yin(因)，yang(央)，ying(英)，yong(雍)。
//
//     u行的韵母，前面没有声母的时候，写成wu(乌)，wa(蛙)，wo(窝)，wai(歪)，
//     wei(威)，wan(弯)，wen(温)，wang(汪)，weng(翁)。
//
//     ü行的韵母，前面没有声母的时候，写成yu(迂)，yue(约)，yuan(冤)，
//     yun(晕)；ü上两点省略。
fn convert_zero_consonant(pinyin: &str) -> String {
    let mut pinyin = pinyin.to_string();
    let raw_pinyin = pinyin.clone();
    match pinyin.chars().next() {
        Some('y') => {
            let no_y_py = pinyin[1..].to_string();
            if let Some(fc) = no_y_py.chars().next() {
                match fc {
                    c if U_TONES.contains(&c) => {
                        let uv = UV_MAP.get(&c.to_string()[..]).unwrap().to_string();
                        pinyin = uv + &no_y_py[1..];
                    }
                    c if I_TONES.contains(&c) => pinyin = no_y_py,
                    _ => pinyin = "i".to_string() + &no_y_py,
                }
            } else {
                pinyin = "i".to_string() + &no_y_py;
            }
        }
        Some('w') => {
            let no_w_py = pinyin[1..].to_string();
            if let Some(fc) = no_w_py.chars().next() {
                if U_TONES.contains(&fc) {
                    pinyin = no_w_py;
                } else {
                    pinyin = "u".to_string() + &no_w_py;
                }
            } else {
                pinyin = "u".to_string() + &no_w_py;
            }
        }
        _ => (),
    }
    if FINALS.contains(pinyin.as_str()) {
        pinyin
    } else {
        raw_pinyin
    }
}

/// 还原原始的韵母
pub fn convert_finals(pinyin: &str) -> String {
    let pinyin = convert_zero_consonant(pinyin);
    let pinyin = replace_with_map(&UV_RE, &UV_MAP, &pinyin);
    let pinyin = replace_with_map(&IU_RE, &IU_MAP, &pinyin);
    let pinyin = replace_with_map(&UI_RE, &UI_MAP, &pinyin);
    replace_with_map(&UN_RE, &UN_MAP, &pinyin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_pinyin() {
        assert_eq!(convert_finals("niu"), "niou");
        assert_eq!(convert_finals("gui"), "guei");
        assert_eq!(convert_finals("lun"), "luen");
        assert_eq!(convert_finals("ju"), "jü");
        assert_eq!(convert_finals("qu"), "qü");
        assert_eq!(convert_finals("xu"), "xü");
    }

    #[test]
    fn test_zero_consonant() {
        assert_eq!(convert_finals("you"), "iou");
        assert_eq!(convert_finals("wu"), "u");
        assert_eq!(convert_finals("yi"), "i");
        assert_eq!(convert_finals("ya"), "ia");
        assert_eq!(convert_finals("we"), "we");
    }

    #[test]
    fn test_special_cases() {
        assert_eq!(convert_finals("yue"), "üe");
        assert_eq!(convert_finals("yuan"), "üan");
        assert_eq!(convert_finals("lü"), "lü");
        assert_eq!(convert_finals("nü"), "nü");
    }

    #[test]
    fn test_edge_cases() {
        assert_eq!(convert_finals(""), "");
        assert_eq!(convert_finals("a"), "a");
        assert_eq!(convert_finals("i"), "i");
        assert_eq!(convert_finals("u"), "u");
        assert_eq!(convert_finals("zzz"), "zzz");
    }

    #[test]
    fn test_mixed_cases() {
        assert_eq!(convert_finals("jia"), "jia");
        assert_eq!(convert_finals("yan"), "ian");
        assert_eq!(convert_finals("wun"), "wuen");
        assert_eq!(convert_finals("jui"), "jüi");
        assert_eq!(convert_finals("xun"), "xün");
    }
}
