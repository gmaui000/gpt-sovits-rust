use super::lazy_pinyin::pinyin::LazyPinyin;
use super::lazy_pinyin::style::Style;
use super::tone_sandhi::ToneSandhi;
use super::zh_normalization::opencpop_strict::OPENCPOP_STRICT;
use super::zh_normalization::text_normalization::TextNormalizer;
use fancy_regex::{Captures, Regex};
use jieba_rs::{Jieba, Tag};
use log::info;
use std::collections::HashMap;
use std::fs;
use std::iter::zip;
use std::ops::Add;

pub struct Chinese {
    pub rep_map: HashMap<String, String>,
    pub pinyin_to_symbol_map: HashMap<String, String>,
    pub v_rep_map: HashMap<String, String>,
    pub pinyin_rep_map: HashMap<String, String>,
    pub single_rep_map: HashMap<String, String>,
    pub pattern: Regex,
    pub pattern2: Regex,
    pub pattern3: Regex,
    pub pattern4: Regex,
    pub punctuation: [String; 6],
    pub text_normalizer: TextNormalizer,
    pub jieba_util: Jieba,
    pub tone_modifier: ToneSandhi,
    pub lazy_pinyin: LazyPinyin,
}

fn escape_string(input: &str) -> String {
    let pattern = Regex::new("[\\^$.?*+{}[|]()#/]").unwrap();
    return pattern.replace_all(input, "\\$0").to_string();
}

impl Chinese {
    pub fn init(rep_map_json_path: &str, phrases_dict_path: &str, pinyin_dict_path: &str) -> Self {
        let file = fs::File::open(rep_map_json_path).unwrap();
        let rep_map: HashMap<String, String> = serde_json::from_reader(&file).unwrap();

        let opencpop_strict = OPENCPOP_STRICT.map(|x| (x.0.to_string(), x.1.to_string()));
        let pinyin_to_symbol_map = HashMap::from(opencpop_strict);

        let v_rep_map: HashMap<String, String> = HashMap::from([
            ("uei".to_string(), "ui".to_string()),
            ("iou".to_string(), "iu".to_string()),
            ("uen".to_string(), "un".to_string()),
        ]);

        let pinyin_rep_map: HashMap<String, String> = HashMap::from([
            ("ing".to_string(), "ying".to_string()),
            ("i".to_string(), "yi".to_string()),
            ("in".to_string(), "yin".to_string()),
            ("u".to_string(), "wu".to_string()),
        ]);

        let single_rep_map: HashMap<String, String> = HashMap::from([
            ("v".to_string(), "yu".to_string()),
            ("e".to_string(), "e".to_string()),
            ("i".to_string(), "y".to_string()),
            ("u".to_string(), "w".to_string()),
        ]);

        let mut ps = vec![];
        for p in rep_map.keys() {
            let p = escape_string(p);
            ps.push(p);
        }

        let ps = ps.join("|");
        let pattern = Regex::new(&ps).unwrap();

        // 中文和符号
        let punctuation = [
            "!".to_string(),
            "?".to_string(),
            "…".to_string(),
            ",".to_string(),
            ".".to_string(),
            "-".to_string(),
        ];
        let pr = punctuation.join("");
        let mut pt = r"[^\u4e00-\u9fa5".to_string();
        pt.push_str(&pr);
        pt.push_str("]+");

        let pattern2 = Regex::new(&pt).unwrap();

        let pattern3 = Regex::new(r"[?<=[!?…,.-]]\s*").unwrap();
        let pattern4 = Regex::new(r"[a-zA-Z]+").unwrap();

        let text_normalizer = TextNormalizer::new("../data/zh_dict.json");
        let tone_modifier = ToneSandhi::init();
        let jieba_util = Jieba::new();

        let lazy_pinyin = LazyPinyin::init(phrases_dict_path, pinyin_dict_path).unwrap();

        Chinese {
            rep_map,
            pinyin_to_symbol_map,
            v_rep_map,
            pinyin_rep_map,
            single_rep_map,
            pattern,
            pattern2,
            pattern3,
            pattern4,
            punctuation,
            text_normalizer,
            jieba_util,
            tone_modifier,
            lazy_pinyin,
        }
    }

    /// 符号统一替换为英文输入下的符号
    pub fn replace_symbol(&self, sentence: String) -> String {
        let replacement = |caps: &Captures| -> String {
            let v = &caps[0];
            let s = self.rep_map.get(v).unwrap().clone();
            s
        };
        let replaced_text = self
            .pattern
            .replace_all(&sentence, &replacement)
            .to_string();

        replaced_text
    }

    /// 标点符号替换
    pub fn replace_punctuation(&self, sentence: String) -> String {
        let text = sentence.replace("嗯", "恩").replace("呣", "母");

        let replaced_text = self.replace_symbol(text);
        // 移除非中文的
        let replaced_text = self.pattern2.replace_all(&replaced_text, "").to_string();

        replaced_text
    }

    pub fn g2p(&self, text: &str) -> (Vec<String>, Vec<usize>) {
        let mut sentences = vec![];

        let replacement = |caps: &Captures| -> String {
            let v = &caps[0];
            v.to_string().add("\n")
        };
        let text2 = self.pattern3.replace_all(text, replacement);

        for i in text2.split("\n") {
            if i.trim() != "" {
                sentences.push(i.to_string());
            }
        }

        let (phones, word2ph) = self._g2p(&sentences);

        (phones, word2ph)
    }

    //
    fn _get_initials_finals(&self, word: &str) -> (Vec<String>, Vec<String>) {
        let mut initials: Vec<String> = vec![];
        let mut finals: Vec<String> = vec![];
        let orig_initials = self.lazy_pinyin.lazy_pinyin(word, Style::Initials, true);
        let orig_finals = self
            .lazy_pinyin
            .lazy_pinyin(word, Style::InitialsTone3, true);
        for (mut c, mut v) in zip(orig_initials, orig_finals) {
            initials.append(&mut c);
            finals.append(&mut v)
        }
        (initials, finals)
    }

    pub fn _g2p(&self, segments: &[String]) -> (Vec<String>, Vec<usize>) {
        let mut phones_list: Vec<String> = vec![];
        let mut word2ph: Vec<usize> = vec![];

        (0..segments.len()).for_each(|i| {
            let seg = &segments[i];
            // Replace all English words in the sentence
            let rp_seg = self.pattern4.replace_all(seg, "").to_string();

            let seg_cut: Vec<Tag> = self.jieba_util.tag(&rp_seg, false);

            let mut initials: Vec<String> = vec![];
            let mut finals: Vec<String> = vec![];
            //
            let seg_cut = self.tone_modifier.pre_merge_for_modify(&seg_cut);

            for (word, pos) in &seg_cut {
                if pos == "eng" {
                    continue;
                }
                let (mut sub_initials, sub_finals) = self._get_initials_finals(word);
                let mut sub_finals =
                    self.tone_modifier
                        .modified_tone(word, pos, sub_finals, &self.jieba_util);
                initials.append(&mut sub_initials);
                finals.append(&mut sub_finals);
            }
            for (c, v) in zip(initials, finals) {
                // let raw_pinyin = c.clone() + &v;
                let mut phone: Vec<String> = vec![];
                if c == v {
                    // assert c in punctuation
                    if !self.punctuation.contains(&c) {
                        info!("assert c in punctuation is false");
                    }
                    phone = Vec::from([c.clone()]);
                    word2ph.push(1);
                } else {
                    let v_len = v.len();
                    let v_without_tone = &v[..v_len - 1];
                    let tone = &v[v_len - 1..];
                    let mut pinyin = c.clone() + v_without_tone;
                    // assert tone in "12345";
                    if !"12345".contains(tone) {
                        info!("assert tone in 12345 is false");
                    }
                    if !c.is_empty() {
                        if self.v_rep_map.contains_key(v_without_tone) {
                            pinyin = c.clone() + self.v_rep_map.get(v_without_tone).unwrap();
                        }
                    } else if self.pinyin_rep_map.contains_key(&pinyin) {
                        pinyin = self.pinyin_rep_map.get(&pinyin).unwrap().to_string();
                    } else if pinyin.chars().count() > 0 {
                        let pinyin_0 = pinyin.chars().nth(0).unwrap().to_string();
                        if self.single_rep_map.contains_key(&pinyin_0) {
                            pinyin = self.single_rep_map.get(&pinyin_0).unwrap().to_string()
                                + &pinyin[1..];
                        }
                    }

                    let new_cv_opt = self.pinyin_to_symbol_map.get(&pinyin);
                    if new_cv_opt.is_some() {
                        let new_cv: Vec<&str> = new_cv_opt.unwrap().split(" ").collect();
                        let new_c = new_cv[0];
                        let new_v = new_cv[1];
                        let new_v = new_v.to_string() + tone;
                        phone = vec![new_c.to_string(), new_v];
                        word2ph.push(phone.len());
                    } else {
                        info!("assert {} in pinyin_to_symbol_map.keys() error", pinyin)
                    }
                }

                phones_list.append(&mut phone);
            }
        });

        (phones_list, word2ph)
    }

    /// test 1次
    pub fn text_normalize(&self, text: String) -> String {
        // 只是替换符号
        let replaced_text = self.replace_symbol(text);
        let sentences = self.text_normalizer.normalize(&replaced_text);
        let mut dest_text = "".to_string();
        for sentence in sentences {
            let dt = self.replace_punctuation(sentence);
            dest_text.push_str(&dt);
        }
        dest_text
    }
}

#[test]
fn chinese_test0() {
    let num = Chinese::init(
        "../data/rep_map.json",
        "../data/PHRASES_DICT.json",
        "../data/PINYIN_DICT.json",
    );

    // let text = "上山不一样，下山也不一样，都是受伤了的".to_string();
    let text = "每个人的理想不一样，扎出来的风筝也不一样。所有的风筝中，要数小音乐家根子的最棒了，那是一架竖琴。让她到天上去好好想想吧！哈，风筝的后脑勺上还拖着一条马尾巴似的长辫子！在地面上，我们一边放线一边跑着，手里的线越放越长，风筝也带着我们的理想越飞越远，越飞越高如果把眼前的一池荷花看作一大幅活的画，那画家的本领可真了不起。".to_string();

    let text = num.text_normalize(text);
    let text = num.text_normalizer.normalize(&text);

    for t in text {
        let (phones_list, word2ph) = num.g2p(&t);
        println!("{:?}", phones_list);
        println!("{:?}", word2ph);
    }
}
