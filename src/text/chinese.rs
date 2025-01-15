use super::lazy_pinyin::pinyin::LazyPinyin;
use super::lazy_pinyin::style::Style;
use super::tone_sandhi::ToneSandhi;
use super::zh_normalization::opencpop_strict::OPENCPOP_STRICT;
use super::zh_normalization::text_normalization::TextNormalizer;
use fancy_regex::{Captures, Regex};
use jieba_rs::Jieba;
use lazy_static::lazy_static;
use log::info;
use std::collections::HashMap;

lazy_static! {
    // 常量数据
    static ref V_REP_MAP: HashMap<&'static str, &'static str> = HashMap::from([
        ("uei", "ui"),
        ("iou", "iu"),
        ("uen", "un"),
    ]);

    static ref PINYIN_REP_MAP: HashMap<&'static str, &'static str> = HashMap::from([
        ("ing", "ying"),
        ("i", "yi"),
        ("in", "yin"),
        ("u", "wu"),
    ]);

    static ref SINGLE_REP_MAP: HashMap<&'static str, &'static str> = HashMap::from([
        ("v", "yu"),
        ("e", "e"),
        ("i", "y"),
        ("u", "w"),
    ]);

    static ref PUNCTUATION: [&'static str; 6] = ["!", "?", "…", ",", ".", "-"];

    // 正则表达式
    static ref ESCAPE_PATTERN: Regex = Regex::new(r"[\\^$.?*+{}[|]()#/]").unwrap();
    static ref RE_NON_CHINESE_OR_PUNCTUATION: Regex = Regex::new(&format!(
        r"[^\u4e00-\u9fa5{}]+",
        PUNCTUATION.join("")
    )).unwrap();
    static ref RE_SENTENCE_SPLIT: Regex = Regex::new(r"[?<=[!?…,.-]]\s*").unwrap();
    static ref RE_ENGLISH_LETTER: Regex = Regex::new(r"[a-zA-Z]+").unwrap();
}

pub struct Chinese {
    rep_map: HashMap<String, String>,
    pinyin_to_symbol_map: HashMap<String, String>,
    pattern: Regex,
    text_normalizer: TextNormalizer,
    jieba_util: Jieba,
    tone_modifier: ToneSandhi,
    lazy_pinyin: LazyPinyin,
}

impl Chinese {
    pub fn new(rep_map_json_path: &str, phrases_dict_path: &str, pinyin_dict_path: &str) -> Self {
        let rep_map: HashMap<String, String> =
            serde_json::from_reader(std::fs::File::open(rep_map_json_path).unwrap()).unwrap();

        let pinyin_to_symbol_map = OPENCPOP_STRICT
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        // 生成转义的正则
        let escaped_keys = rep_map
            .keys()
            .map(|key| ESCAPE_PATTERN.replace_all(key, "\\$0").to_string())
            .collect::<Vec<_>>()
            .join("|");
        let pattern = Regex::new(&escaped_keys).unwrap();

        Self {
            rep_map,
            pinyin_to_symbol_map,
            pattern,
            text_normalizer: TextNormalizer::new("../data/zh_dict.json"),
            jieba_util: Jieba::new(),
            tone_modifier: ToneSandhi::init(),
            lazy_pinyin: LazyPinyin::init(phrases_dict_path, pinyin_dict_path).unwrap(),
        }
    }

    pub fn text_normalize(&self, text: &str) -> String {
        let replaced_text = self.replace_symbol(text);
        self.text_normalizer
            .normalize(&replaced_text)
            .into_iter()
            .map(|sentence| self.replace_punctuation(&sentence))
            .collect()
    }

    /// 符号统一替换为英文输入下的符号
    pub fn replace_symbol(&self, text: &str) -> String {
        self.pattern
            .replace_all(text, |caps: &Captures| {
                self.rep_map
                    .get(&caps[0])
                    .cloned()
                    .unwrap_or_else(|| caps[0].to_string())
            })
            .to_string()
    }

    pub fn g2p(&self, text: &str) -> (Vec<String>, Vec<usize>) {
        let sentences: Vec<_> = RE_SENTENCE_SPLIT
            .replace_all(text, |caps: &Captures| format!("{}\n", &caps[0]))
            .split('\n')
            .filter(|line| !line.trim().is_empty())
            .map(String::from)
            .collect();

        let (phones, word2ph) = self._g2p(&sentences);

        (phones, word2ph)
    }

    fn extract_initials_and_finals(&self, word: &str) -> (Vec<String>, Vec<String>) {
        let initials = self
            .lazy_pinyin
            .lazy_pinyin(word, Style::Initials, true)
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(); // Flatten and collect into a Vec<String>

        let finals = self
            .lazy_pinyin
            .lazy_pinyin(word, Style::InitialsTone3, true)
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(); // Flatten and collect into a Vec<String>

        (initials, finals)
    }

    /// 标点符号替换
    fn replace_punctuation(&self, text: &str) -> String {
        let cleaned_text = text.replace("嗯", "恩").replace("呣", "母");
        RE_NON_CHINESE_OR_PUNCTUATION
            .replace_all(&cleaned_text, "")
            .to_string()
    }

    fn _g2p(&self, segments: &[String]) -> (Vec<String>, Vec<usize>) {
        let mut phones_list: Vec<String> = vec![];
        let mut word2ph: Vec<usize> = vec![];

        for seg in segments {
            // 移除英文字符
            let rp_seg = RE_ENGLISH_LETTER.replace_all(seg, "").to_string();

            // 分词并处理
            let seg_cut = self
                .tone_modifier
                .pre_merge_for_modify(&self.jieba_util.tag(&rp_seg, false));

            let mut initials = Vec::new();
            let mut finals = Vec::new();

            for (word, pos) in seg_cut {
                if pos == "eng" {
                    continue;
                }

                let (mut sub_initials, sub_finals) = self.extract_initials_and_finals(&word);
                let sub_finals =
                    self.tone_modifier
                        .modified_tone(&word, &pos, sub_finals, &self.jieba_util);

                initials.append(&mut sub_initials);
                finals.extend(sub_finals);
            }

            for (c, v) in initials.into_iter().zip(finals) {
                if c == v {
                    // 符号处理
                    if !PUNCTUATION.contains(&c.as_str()) {
                        info!("Unexpected non-punctuation character: {}", c);
                    }
                    phones_list.push(c);
                    word2ph.push(1);
                } else {
                    // 拼音处理
                    let tone = &v[v.len() - 1..];
                    let mut pinyin = format!("{}{}", c, &v[..v.len() - 1]);
                    if !c.is_empty() {
                        if let Some(new_v) = V_REP_MAP.get(&v[..v.len() - 1]) {
                            pinyin = format!("{}{}", c, new_v);
                        }
                    } else if let Some(new_pinyin) = PINYIN_REP_MAP.get(pinyin.as_str()) {
                        pinyin = new_pinyin.to_string();
                    } else if let Some(first_char) = pinyin.chars().next() {
                        if let Some(new_char) = SINGLE_REP_MAP.get(&first_char.to_string().as_str())
                        {
                            pinyin = format!("{}{}", new_char, &pinyin[1..]);
                        }
                    }

                    if let Some(new_cv) = self.pinyin_to_symbol_map.get(&pinyin) {
                        let mut parts: Vec<String> =
                            new_cv.split_whitespace().map(String::from).collect();
                        if let Some(last_part) = parts.last_mut() {
                            *last_part += tone; // 加入声调
                        }
                        word2ph.push(parts.len());
                        phones_list.extend(parts);
                    } else {
                        info!("Pinyin not found in symbol map: {}", pinyin);
                    }
                }
            }
        }

        (phones_list, word2ph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_samples() {
        let num = Chinese::new(
            "../data/rep_map.json",
            "../data/PHRASES_DICT.json",
            "../data/PINYIN_DICT.json",
        );

        let text = "每个人的理想不一样，扎出来的风筝也不一样。所有的风筝中，要数小音乐家根子的最棒了，那是一架竖琴。让她到天上去好好想想吧！哈，风筝的后脑勺上还拖着一条马尾巴似的长辫子！在地面上，我们一边放线一边跑着，手里的线越放越长，风筝也带着我们的理想越飞越远，越飞越高如果把眼前的一池荷花看作一大幅活的画，那画家的本领可真了不起。".to_string();

        let text = num.text_normalize(&text);
        let text = num.text_normalizer.normalize(&text);

        let expected_results = vec![
            (
                vec![
                    "m", "ei3", "g", "e5", "r", "en2", "d", "e5", "l", "i2", "x", "iang3", "b",
                    "u4", "y", "i2", "y", "ang4", ",",
                ],
                vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 1],
            ),
            (
                vec![
                    "zh", "a1", "ch", "u1", "l", "ai5", "d", "e5", "f", "eng1", "zh", "eng5", "y",
                    "E3", "b", "u4", "y", "i2", "y", "ang4", ".", "s", "uo2", "y", "ou3", "d",
                    "e5", "f", "eng1", "zh", "eng5", "zh", "ong1", ",",
                ],
                vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1],
            ),
            (
                vec![
                    "y", "ao4", "sh", "u4", "x", "iao3", "y", "in1", "y", "ve4", "j", "ia1", "g",
                    "en1", "z", "i05", "d", "e5", "z", "ui4", "b", "ang4", "l", "e5", ",",
                ],
                vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1],
            ),
            (
                vec![
                    "n", "a4", "sh", "ir4", "y", "i2", "j", "ia4", "sh", "u4", "q", "in2", ".",
                    "r", "ang4", "t", "a1", "d", "ao4", "t", "ian1", "sh", "ang5", "q", "v4", "h",
                    "ao2", "h", "ao3", "x", "iang2", "x", "iang3", "b", "a5", "!",
                ],
                vec![2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1],
            ),
            (vec!["h", "a5", ","], vec![2, 1]),
            (
                vec![
                    "f", "eng1", "zh", "eng5", "d", "e5", "h", "ou4", "n", "ao3", "sh", "ao2",
                    "sh", "ang4", "h", "ai2", "t", "uo1", "zh", "e5", "y", "i4", "t", "iao2", "m",
                    "a3", "w", "ei3", "b", "a5", "sh", "ir4", "d", "e5", "zh", "ang3", "b", "ian4",
                    "z", "i05", "!",
                ],
                vec![
                    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1,
                ],
            ),
            (
                vec!["z", "ai4", "d", "i4", "m", "ian4", "sh", "ang4", ","],
                vec![2, 2, 2, 2, 1],
            ),
            (
                vec![
                    "w", "o3", "m", "en5", "y", "i4", "b", "ian1", "f", "ang4", "x", "ian4", "y",
                    "i4", "b", "ian1", "p", "ao3", "zh", "e5", ",",
                ],
                vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1],
            ),
            (
                vec![
                    "sh", "ou3", "l", "i5", "d", "e5", "x", "ian4", "y", "ve4", "f", "ang4", "y",
                    "ve4", "zh", "ang3", ",",
                ],
                vec![2, 2, 2, 2, 2, 2, 2, 2, 1],
            ),
            (
                vec![
                    "f", "eng1", "zh", "eng5", "y", "E3", "d", "ai4", "zh", "e5", "w", "o3", "m",
                    "en5", "d", "e5", "l", "i2", "x", "iang3", "y", "ve4", "f", "ei1", "y", "ve4",
                    "y", "van3", ",",
                ],
                vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1],
            ),
            (
                vec![
                    "y", "ve4", "f", "ei1", "y", "ve4", "g", "ao1", "r", "u2", "g", "uo3", "b",
                    "a3", "y", "En3", "q", "ian2", "d", "e5", "y", "i4", "ch", "ir2", "h", "e2",
                    "h", "ua1", "k", "an4", "z", "uo4", "y", "i2", "d", "a4", "f", "u2", "h",
                    "uo2", "d", "e5", "h", "ua4", ",",
                ],
                vec![
                    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1,
                ],
            ),
            (
                vec![
                    "n", "a4", "h", "ua4", "j", "ia1", "d", "e5", "b", "en2", "l", "ing3", "k",
                    "e3", "zh", "en1", "l", "iao3", "b", "u5", "q", "i3", ".",
                ],
                vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1],
            ),
        ];

        for (t, (expected_phones, expected_word2ph)) in text.iter().zip(expected_results) {
            let (phones_list, word2ph) = num.g2p(t);
            assert_eq!(
                phones_list, expected_phones,
                "Mismatch in phones_list for segment: {}",
                t
            );
            assert_eq!(
                word2ph, expected_word2ph,
                "Mismatch in word2ph for segment: {}",
                t
            );
        }
    }

    #[test]
    fn test_text_normalization() {
        let chinese = Chinese::new(
            "../data/rep_map.json",
            "../data/PHRASES_DICT.json",
            "../data/PINYIN_DICT.json",
        );
        let text = "上山90%不一样，下山1/10也不一样，都是受伤了的";
        assert_eq!(
            chinese.text_normalize(text),
            "上山百分之九十不一样,下山十分之一也不一样,都是受伤了的"
        );
    }

    #[test]
    fn test_g2p() {
        let chinese = Chinese::new(
            "../data/rep_map.json",
            "../data/PHRASES_DICT.json",
            "../data/PINYIN_DICT.json",
        );
        let (phones, word2ph) = chinese.g2p("我喜欢学习");
        assert_eq!(
            phones,
            vec!["w", "o3", "x", "i3", "h", "uan5", "x", "ve2", "x", "i2"]
        );
        assert_eq!(word2ph, vec![2, 2, 2, 2, 2]);
    }
}
