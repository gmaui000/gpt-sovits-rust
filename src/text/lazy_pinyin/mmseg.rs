use std::collections::{HashMap, HashSet};
use substring::Substring;

pub struct MMSeg {
    _no_non_phrases: bool,
    _prefix_set: HashSet<String>,
}

impl MMSeg {
    pub fn init(_no_non_phrases: bool, phrases_dict: &HashMap<String, Vec<Vec<String>>>) -> Self {
        let mut _prefix_set: HashSet<String> = HashSet::new();
        for word in phrases_dict.keys() {
            for index in 0..word.chars().count() {
                _prefix_set.insert(word.substring(0, index + 1).to_string());
            }
        }

        MMSeg {
            _no_non_phrases,
            _prefix_set,
        }
    }

    pub fn seg(&self, text: &str, phrases_dict: &HashMap<String, Vec<Vec<String>>>) -> Vec<String> {
        let mut seg_words = vec![];
        let mut remain = text.to_string();
        while !remain.is_empty() {
            let mut matched = "".to_string();
            let seg_words_len = seg_words.len();
            for index in 0..remain.chars().count() {
                let word = remain.substring(0, index + 1);
                if self._prefix_set.contains(word) {
                    matched = word.to_string();
                } else {
                    if !matched.is_empty()
                        && ((!self._no_non_phrases) || phrases_dict.contains_key(&matched))
                    {
                        seg_words.push(matched.clone());
                        remain = remain.substring(index, remain.chars().count()).to_string();
                    } else if self._no_non_phrases {
                        seg_words.push(word.chars().nth(0).unwrap().to_string());
                        remain = remain
                            .substring(index + 2 - word.chars().count(), remain.chars().count())
                            .to_string();
                    } else {
                        seg_words.push(word.to_string());
                        remain = remain
                            .substring(index + 1, remain.chars().count())
                            .to_string();
                    }
                    break;
                }
            }
            // 整个文本就是一个词语，或者不包含任何词语
            if seg_words_len == seg_words.len() {
                if self._no_non_phrases && !phrases_dict.contains_key(&remain) {
                    for x in remain.chars() {
                        seg_words.push(x.to_string());
                    }
                } else {
                    seg_words.push(remain.clone());
                }
                break;
            }
        }

        seg_words
    }
}

#[test]
fn test2() {}
