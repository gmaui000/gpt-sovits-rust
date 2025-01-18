use std::collections::{HashMap, HashSet};
use substring::Substring;

pub struct MMSeg {
    no_non_phrases: bool,
    prefix_set: HashSet<String>,
}

impl MMSeg {
    pub fn new(no_non_phrases: bool, phrases_dict: &HashMap<String, Vec<Vec<String>>>) -> Self {
        let prefix_set: HashSet<String> = phrases_dict
            .keys()
            .flat_map(|word| {
                (1..=word.chars().count()).map(move |i| word.substring(0, i).to_string())
            })
            .collect();

        Self {
            no_non_phrases,
            prefix_set,
        }
    }

    pub fn seg(&self, text: &str, phrases_dict: &HashMap<String, Vec<Vec<String>>>) -> Vec<String> {
        let mut seg_words = Vec::new();
        let mut remain = text;

        while !remain.is_empty() {
            let mut matched = "".to_string();
            let seg_words_len = seg_words.len();
            for index in 0..remain.chars().count() {
                let word = remain.substring(0, index + 1);
                if self.prefix_set.contains(word) {
                    matched = word.to_string();
                } else {
                    if !matched.is_empty()
                        && (!self.no_non_phrases || phrases_dict.contains_key(&matched))
                    {
                        seg_words.push(matched);
                        remain = remain.substring(index, remain.len());
                    } else if self.no_non_phrases {
                        seg_words.push(remain.chars().next().unwrap().to_string());
                        remain = remain.substring(1, remain.len());
                    } else {
                        seg_words.push(word.to_string());
                        remain = remain.substring(index + 1, remain.len());
                    }
                    break;
                }
            }

            if seg_words_len == seg_words.len() {
                seg_words.extend(
                    if self.no_non_phrases && !phrases_dict.contains_key(remain) {
                        remain.chars().map(|c| c.to_string()).collect::<Vec<_>>()
                    } else {
                        vec![remain.to_string()]
                    },
                );
                break;
            }
        }

        seg_words
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_normal_segmentation() {
        let mut phrases_dict: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        // phrases_dict.insert(
        //     "中华人民共和国".to_string(),
        //     vec![vec!["中华人民共和国".to_string()]],
        // );
        phrases_dict.insert("中国".to_string(), vec![vec!["中国".to_string()]]);
        phrases_dict.insert("人民".to_string(), vec![vec!["人民".to_string()]]);
        phrases_dict.insert("共和国".to_string(), vec![vec!["共和国".to_string()]]);

        let mmseg = MMSeg::new(false, &phrases_dict);
        let result = mmseg.seg("中华人民共和国", &phrases_dict);
        assert_eq!(result, vec!["中", "华", "人民", "共和国"]);
    }

    #[test]
    fn test_overlapping_phrases_false() {
        let mut phrases_dict: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        phrases_dict.insert("中华人民".to_string(), vec![vec!["中华人民".to_string()]]);
        phrases_dict.insert("共和国".to_string(), vec![vec!["共和国".to_string()]]);

        let mmseg = MMSeg::new(false, &phrases_dict);
        let result = mmseg.seg("中华人生共和国", &phrases_dict);
        assert_eq!(result, vec!["中华人", "生", "共和国"]);
    }

    #[test]
    fn test_overlapping_phrases_true() {
        let mut phrases_dict: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        phrases_dict.insert("中华人民".to_string(), vec![vec!["中华人民".to_string()]]);
        phrases_dict.insert("共和国".to_string(), vec![vec!["共和国".to_string()]]);

        let mmseg = MMSeg::new(true, &phrases_dict);
        let result = mmseg.seg("中华人生共和国", &phrases_dict);
        assert_eq!(result, vec!["中", "华", "人", "生", "共和国"]);
    }

    #[test]
    fn test_empty_text() {
        let phrases_dict: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        let mmseg = MMSeg::new(false, &phrases_dict);
        let result = mmseg.seg("", &phrases_dict);
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_word() {
        let mut phrases_dict: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        phrases_dict.insert("测试".to_string(), vec![vec!["测试".to_string()]]);

        let mmseg = MMSeg::new(false, &phrases_dict);
        let result = mmseg.seg("测试", &phrases_dict);
        assert_eq!(result, vec!["测试".to_string()]);
    }

    #[test]
    fn test_mixed_text() {
        let mut phrases_dict: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        phrases_dict.insert("测试".to_string(), vec![vec!["测试".to_string()]]);
        phrases_dict.insert("开发".to_string(), vec![vec!["开发".to_string()]]);

        let mmseg = MMSeg::new(false, &phrases_dict);
        let result = mmseg.seg("测试开发中", &phrases_dict);
        assert_eq!(
            result,
            vec!["测试".to_string(), "开发".to_string(), "中".to_string()]
        );
    }

    #[test]
    fn test_no_phrase_in_dict() {
        let phrases_dict: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        let mmseg = MMSeg::new(false, &phrases_dict);
        let result = mmseg.seg("测试开发中", &phrases_dict);
        assert_eq!(result, vec!["测", "试", "开", "发", "中"]);
    }

    #[test]
    fn test_repeated_phrases() {
        let mut phrases_dict: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        phrases_dict.insert("好好".to_string(), vec![vec!["好好".to_string()]]);
        phrases_dict.insert("学习".to_string(), vec![vec!["学习".to_string()]]);

        let mmseg = MMSeg::new(false, &phrases_dict);
        let result = mmseg.seg("好好学习好好学习", &phrases_dict);
        assert_eq!(
            result,
            vec![
                "好好".to_string(),
                "学习".to_string(),
                "好好".to_string(),
                "学习".to_string()
            ]
        );
    }

    #[test]
    fn test_special_characters() {
        let mut phrases_dict: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        phrases_dict.insert(
            "hello world".to_string(),
            vec![vec!["hello".to_string(), "world".to_string()]],
        );
        phrases_dict.insert("hello".to_string(), vec![vec!["hello".to_string()]]);
        phrases_dict.insert("world".to_string(), vec![vec!["world".to_string()]]);

        let mmseg = MMSeg::new(false, &phrases_dict);
        let result = mmseg.seg("hello world", &phrases_dict);
        assert_eq!(result, vec!["hello world"]);
    }

    #[test]
    fn test_numbers_and_symbols() {
        let mut phrases_dict: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        phrases_dict.insert("2025年".to_string(), vec![vec!["2025年".to_string()]]);
        phrases_dict.insert("2025".to_string(), vec![vec!["2025".to_string()]]);
        phrases_dict.insert("年".to_string(), vec![vec!["年".to_string()]]);

        let mmseg = MMSeg::new(false, &phrases_dict);
        let result = mmseg.seg("2025年", &phrases_dict);
        assert_eq!(result, vec!["2025年".to_string()]);
    }

    #[test]
    fn test_text_with_spaces() {
        let mut phrases_dict: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        phrases_dict.insert(
            "你好 世界".to_string(),
            vec![vec!["你好".to_string(), "世界".to_string()]],
        );
        phrases_dict.insert("你好".to_string(), vec![vec!["你好".to_string()]]);
        phrases_dict.insert("世界".to_string(), vec![vec!["世界".to_string()]]);

        let mmseg = MMSeg::new(false, &phrases_dict);
        let result = mmseg.seg("你好 世界", &phrases_dict);
        assert_eq!(result, vec!["你好 世界".to_string()]);
    }
}
