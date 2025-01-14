use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::{HashMap, HashSet};
use substring::Substring;

lazy_static! {

static ref  U_TONES: HashSet<char> = HashSet::from(['ū', 'u', 'ǔ', 'ú', 'ù']);

static ref  I_TONES: HashSet<char> = HashSet::from(['i', 'ǐ', 'í', 'ī', 'ì']);


pub static ref  _FINALS: HashSet<&'static str> = {
    HashSet::from(["i", "u", "ü", "a", "ia", "ua", "o", "uo", "e", "ie", "üe", "ai", "uai", "ei", "uei", "ao", "iao", "ou", "iou", "an", "ian", "uan", "üan", "en", "in", "uen", "ün", "ang", "iang", "uang", "eng", "ing", "ueng", "ong", "iong", "er", "ê"])
};

static ref  UV_RE: Regex = Regex::new(r"^(j|q|x)(u|ū|ú|ǔ|ù)(.*)$").unwrap();
static ref  IU_RE: Regex = Regex::new(r"^([a-z]+)(iǔ|iū|iu|iù|iú)$").unwrap();
static ref  UI_RE: Regex = Regex::new(r"([a-z]+)(ui|uí|uì|uǐ|uī)$").unwrap();
static ref  UN_RE: Regex = Regex::new(r"([a-z]+)(ǔn|ún|ùn|un|ūn)$").unwrap();


// iu -> iou
static ref  IU_MAP: HashMap<&'static str, &'static str> = {
    HashMap::from(
        [
            ("iu", "iou"),
            ("iū", "ioū"),
            ("iú", "ioú"),
            ("iǔ", "ioǔ"),
            ("iù", "ioù")
        ]
    )
};

// ui -> uei
static ref  UI_MAP: HashMap<&'static str, &'static str> = {
    HashMap::from(
        [
            ("ui", "uei"),
            ("uī", "ueī"),
            ("uí", "ueí"),
            ("uǐ", "ueǐ"),
            ("uì", "ueì")
        ]
    )
};

// un -> uen
static ref  UN_MAP: HashMap<&'static str, &'static str> = {
    HashMap::from(
        [
            ("un", "uen"),
            ("ūn", "ūen"),
            ("ún", "úen"),
            ("ǔn", "ǔen"),
            ("ùn", "ùen"),
        ]
    )
};

// u -> ü
static ref  UV_MAP: HashMap<&'static str, &'static str> = {
    HashMap::from(
        [
            ("u", "ü"),
            ("ū", "ǖ"),
            ("ú", "ǘ"),
            ("ǔ", "ǚ"),
            ("ù", "ǜ")
        ]
    )
};

    }
/// iou 转换，还原原始的韵母
//
//     iou，uei，uen前面加声母的时候，写成iu，ui，un。
//     例如niu(牛)，gui(归)，lun(论)。
fn convert_iou(pinyin: &str) -> String {
    let replacement = |caps: &Captures| -> String {
        let m1 = &caps[1];
        let m2 = &caps[2];
        let iu = IU_MAP.get(&m2).unwrap();

        let value = format!("{}{}", m1, iu);
        value
    };

    let value = IU_RE.replace_all(pinyin, &replacement).to_string();
    value
}

/// uei 转换，还原原始的韵母
//
//     iou，uei，uen前面加声母的时候，写成iu，ui，un。
//     例如niu(牛)，gui(归)，lun(论)。
fn convert_uei(pinyin: &str) -> String {
    let replacement = |caps: &Captures| -> String {
        let m1 = &caps[1];
        let m2 = &caps[2];
        let iu = UI_MAP.get(&m2).unwrap();

        let value = format!("{}{}", m1, iu);
        value
    };

    let value = UI_RE.replace_all(pinyin, &replacement).to_string();
    value
}

/// uen 转换，还原原始的韵母
//
//     iou，uei，uen前面加声母的时候，写成iu，ui，un。
//     例如niu(牛)，gui(归)，lun(论)。
fn convert_uen(pinyin: &str) -> String {
    let replacement = |caps: &Captures| -> String {
        let m1 = &caps[1];
        let m2 = &caps[2];
        let iu = UN_MAP.get(&m2).unwrap();

        let value = format!("{}{}", m1, iu);
        value
    };

    let value = UN_RE.replace_all(pinyin, &replacement).to_string();
    value
}

/// ü 转换，还原原始的韵母
//     ü行的韵跟声母j，q，x拼的时候，写成ju(居)，qu(区)，xu(虚)，
//     ü上两点也省略；但是跟声母n，l拼的时候，仍然写成nü(女)，lü(吕)。
fn convert_uv(pinyin: &str) -> String {
    let replacement = |caps: &Captures| -> String {
        let m1 = &caps[1];
        let m2 = &caps[2];
        let m3 = &caps[3];
        let uv = UV_MAP.get(&m2).unwrap();

        let value = format!("{}{}{}", m1, uv, m3);
        value
    };
    // ju ->'jü'
    let value = UV_RE.replace_all(pinyin, &replacement).to_string();
    value
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
    // y: yu -> v, yi -> i, y -> i
    if raw_pinyin.starts_with("y") {
        //去除 y 后的拼音
        let no_y_py = pinyin.substring(1, pinyin.chars().count()).to_string();
        let first_char = {
            if !no_y_py.is_empty() {
                no_y_py.chars().next()
            } else {
                None
            }
        };
        if first_char.is_some() {
            let fc = first_char.unwrap();
            if U_TONES.contains(&fc) {
                let uv = fc.clone().to_string();
                let c = UV_MAP.get(uv.as_str()).unwrap().to_string();
                pinyin = c + pinyin.substring(2, pinyin.chars().count());
            } else if I_TONES.contains(&fc) {
                pinyin = no_y_py;
            } else {
                pinyin = "i".to_string() + &no_y_py;
            }
        } else {
            pinyin = "i".to_string() + &no_y_py;
        }
    }

    // w: wu -> u, w -> u
    if raw_pinyin.starts_with("w") {
        // 去除 w 后的拼音
        let no_w_py = pinyin.substring(1, pinyin.chars().count()).to_string();
        let first_char = {
            if !no_w_py.is_empty() {
                no_w_py.chars().next()
            } else {
                None
            }
        };
        if first_char.is_some() {
            let fc = first_char.unwrap();
            //  wu -> u: wu -> u
            if U_TONES.contains(&fc) {
                pinyin = pinyin.substring(1, pinyin.chars().count()).to_string();
            } else {
                pinyin = "u".to_string() + pinyin.substring(1, pinyin.chars().count());
            }
        } else {
            pinyin = "u".to_string() + pinyin.substring(1, pinyin.chars().count());
        }
    }
    if !_FINALS.contains(pinyin.as_str()) {
        return raw_pinyin;
    }

    pinyin
}

/// 还原原始的韵母
pub fn convert_finals(pinyin: &str) -> String {
    let pinyin = convert_zero_consonant(pinyin);
    let pinyin = convert_uv(&pinyin);
    let pinyin = convert_iou(&pinyin);
    let pinyin = convert_uei(&pinyin);

    convert_uen(&pinyin)
}
