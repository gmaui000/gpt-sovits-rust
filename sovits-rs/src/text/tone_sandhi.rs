use jieba_rs::{Jieba, Tag};
use pinyin::ToPinyin;
use substring::Substring;

use super::lazy_pinyin::style::get_finals;
use lazy_static::lazy_static;
use std::collections::HashSet;

lazy_static! {
    static ref MUST_NEURAL_TONE_WORDS: HashSet<String> = [
        "麻烦", "麻利", "鸳鸯", "高粱", "骨头", "骆驼", "马虎", "首饰", "馒头", "馄饨", "风筝",
        "难为", "队伍", "阔气", "闺女", "门道", "锄头", "铺盖", "铃铛", "铁匠", "钥匙", "里脊",
        "里头", "部分", "那么", "道士", "造化", "迷糊", "连累", "这么", "这个", "运气", "过去",
        "软和", "转悠", "踏实", "跳蚤", "跟头", "趔趄", "财主", "豆腐", "讲究", "记性", "记号",
        "认识", "规矩", "见识", "裁缝", "补丁", "衣裳", "衣服", "衙门", "街坊", "行李", "行当",
        "蛤蟆", "蘑菇", "薄荷", "葫芦", "葡萄", "萝卜", "荸荠", "苗条", "苗头", "苍蝇", "芝麻",
        "舒服", "舒坦", "舌头", "自在", "膏药", "脾气", "脑袋", "脊梁", "能耐", "胳膊", "胭脂",
        "胡萝", "胡琴", "胡同", "聪明", "耽误", "耽搁", "耷拉", "耳朵", "老爷", "老实", "老婆",
        "老头", "老太", "翻腾", "罗嗦", "罐头", "编辑", "结实", "红火", "累赘", "糨糊", "糊涂",
        "精神", "粮食", "簸箕", "篱笆", "算计", "算盘", "答应", "笤帚", "笑语", "笑话", "窟窿",
        "窝囊", "窗户", "稳当", "稀罕", "称呼", "秧歌", "秀气", "秀才", "福气", "祖宗", "砚台",
        "码头", "石榴", "石头", "石匠", "知识", "眼睛", "眯缝", "眨巴", "眉毛", "相声", "盘算",
        "白净", "痢疾", "痛快", "疟疾", "疙瘩", "疏忽", "畜生", "生意", "甘蔗", "琵琶", "琢磨",
        "琉璃", "玻璃", "玫瑰", "玄乎", "狐狸", "状元", "特务", "牲口", "牙碜", "牌楼", "爽快",
        "爱人", "热闹", "烧饼", "烟筒", "烂糊", "点心", "炊帚", "灯笼", "火候", "漂亮", "滑溜",
        "溜达", "温和", "清楚", "消息", "浪头", "活泼", "比方", "正经", "欺负", "模糊", "槟榔",
        "棺材", "棒槌", "棉花", "核桃", "栅栏", "柴火", "架势", "枕头", "枇杷", "机灵", "本事",
        "木头", "木匠", "朋友", "月饼", "月亮", "暖和", "明白", "时候", "新鲜", "故事", "收拾",
        "收成", "提防", "挖苦", "挑剔", "指甲", "指头", "拾掇", "拳头", "拨弄", "招牌", "招呼",
        "抬举", "护士", "折腾", "扫帚", "打量", "打算", "打点", "打扮", "打听", "打发", "扎实",
        "扁担", "戒指", "懒得", "意识", "意思", "情形", "悟性", "怪物", "思量", "怎么", "念头",
        "念叨", "快活", "忙活", "志气", "心思", "得罪", "张罗", "弟兄", "开通", "应酬", "庄稼",
        "干事", "帮手", "帐篷", "希罕", "师父", "师傅", "巴结", "巴掌", "差事", "工夫", "岁数",
        "屁股", "尾巴", "少爷", "小气", "小伙", "将就", "对头", "对付", "寡妇", "家伙", "客气",
        "实在", "官司", "学问", "学生", "字号", "嫁妆", "媳妇", "媒人", "婆家", "娘家", "委屈",
        "姑娘", "姐夫", "妯娌", "妥当", "妖精", "奴才", "女婿", "头发", "太阳", "大爷", "大方",
        "大意", "大夫", "多少", "多么", "外甥", "壮实", "地道", "地方", "在乎", "困难", "嘴巴",
        "嘱咐", "嘟囔", "嘀咕", "喜欢", "喇嘛", "喇叭", "商量", "唾沫", "哑巴", "哈欠", "哆嗦",
        "咳嗽", "和尚", "告诉", "告示", "含糊", "吓唬", "后头", "名字", "名堂", "合同", "吆喝",
        "叫唤", "口袋", "厚道", "厉害", "千斤", "包袱", "包涵", "匀称", "勤快", "动静", "动弹",
        "功夫", "力气", "前头", "刺猬", "刺激", "别扭", "利落", "利索", "利害", "分析", "出息",
        "凑合", "凉快", "冷战", "冤枉", "冒失", "养活", "关系", "先生", "兄弟", "便宜", "使唤",
        "佩服", "作坊", "体面", "位置", "似的", "伙计", "休息", "什么", "人家", "亲戚", "亲家",
        "交情", "云彩", "事情", "买卖", "主意", "丫头", "丧气", "两口", "东西", "东家", "世故",
        "不由", "不在", "下水", "下巴", "上头", "上司", "丈夫", "丈人", "一辈", "那个", "菩萨",
        "父亲", "母亲", "咕噜", "邋遢", "费用", "冤家", "甜头", "介绍", "荒唐", "大人", "泥鳅",
        "幸福", "熟悉", "计划", "扑腾", "蜡烛", "姥爷", "照顾", "喉咙", "吉他", "弄堂", "蚂蚱",
        "凤凰", "拖沓", "寒碜", "糟蹋", "倒腾", "报复", "逻辑", "盘缠", "喽啰", "牢骚", "咖喱",
        "扫把", "惦记",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    static ref MUST_NOT_NEURAL_TONE_WORDS: HashSet<String> = [
        "男子",
        "女子",
        "分子",
        "原子",
        "量子",
        "莲子",
        "石子",
        "瓜子",
        "电子",
        "人人",
        "虎虎",
        "幺幺",
        "干嘛",
        "学子",
        "哈哈",
        "数数",
        "袅袅",
        "局地",
        "以下",
        "娃哈哈",
        "花花草草",
        "留得",
        "耕地",
        "想想",
        "熙熙",
        "攘攘",
        "卵子",
        "死死",
        "冉冉",
        "恳恳",
        "佼佼",
        "吵吵",
        "打打",
        "考考",
        "整整",
        "莘莘",
        "落地",
        "算子",
        "家家户户",
        "青青",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    static ref PUNCTUATION: String = String::from("：，；。？！“”‘’':,;.?!");
}

pub struct ToneSandhi;

impl ToneSandhi {
    pub fn new() -> Self {
        Self
    }

    pub fn pre_merge_for_modify(&self, seg_cut: &[Tag]) -> Vec<(String, String)> {
        [
            ToneSandhi::merge_yi,
            ToneSandhi::merge_reduplication,
            ToneSandhi::merge_continuous_three_tones,
            ToneSandhi::merge_continuous_three_tones_2,
            ToneSandhi::merge_er,
        ]
        .into_iter()
        .fold(ToneSandhi::merge_bu(seg_cut), |acc, merge_fn| {
            merge_fn(&acc)
        })
    }

    pub fn modified_tone(
        &self,
        word: &str,
        pos: &str,
        finals: Vec<String>,
        jieba_util: &Jieba,
    ) -> Vec<String> {
        let finals = ToneSandhi::bu_sandhi(word, finals);
        let finals = self.yi_sandhi(word, finals);
        let finals = self.neural_sandhi(word, pos, finals, jieba_util);

        self.three_sandhi(word, finals, jieba_util)
    }

    fn neural_sandhi(
        &self,
        word: &str,
        pos: &str,
        finals: Vec<String>,
        jieba_util: &Jieba,
    ) -> Vec<String> {
        let word_len = word.chars().count();
        let finals_len = finals.len();
        if word_len == 0 || finals_len == 0 {
            return finals;
        }

        let mut finals = finals;
        // 调整重复音调的逻辑
        (1..word_len).for_each(|j| {
            if let (Some(item), Some(pre_item)) = (word.chars().nth(j), word.chars().nth(j - 1)) {
                if item == pre_item
                    && matches!(pos.chars().next(), Some('n' | 'v' | 'a'))
                    && !MUST_NOT_NEURAL_TONE_WORDS.contains(word)
                {
                    let f_len = finals[j].chars().count();
                    let mut f_pre = finals[j].chars().take(f_len - 1).collect::<String>();
                    if f_pre.is_empty() {
                        f_pre = finals[j].clone();
                    }
                    finals[j] = f_pre + "5";
                }
            }
        });

        // 检查特殊字符并调整音调
        if let Some(last_char) = word.chars().last() {
            if (word_len >= 1 && "吧呢哈啊呐噻嘛吖嗨呐哦哒额滴哩哟喽啰耶喔诶".contains(last_char))
                || (word_len == 1
                    && "了着过".contains(last_char)
                    && ["ul", "uz", "ug"].contains(&pos))
                || (word_len > 1
                    && ("们子".contains(last_char)
                        && ["r", "n"].contains(&pos)
                        && !MUST_NOT_NEURAL_TONE_WORDS.contains(word)
                        || "来去".contains(last_char)
                            && "上下进出回过起开"
                                .contains(word.chars().nth(word_len - 2).unwrap())
                        || "上下里".contains(last_char) && ["s", "l", "f"].contains(&pos)))
            {
                let f_len = finals[finals_len - 1].chars().count();
                let f_pre = finals[finals_len - 1]
                    .chars()
                    .take(f_len - 1)
                    .collect::<String>();
                finals[finals_len - 1] = f_pre + "5";
            } else if word_len >= 1 && "的地得".contains(last_char) {
                let f_len = finals[finals_len - 1].chars().count();
                let mut f_pre = finals[finals_len - 1]
                    .chars()
                    .take(f_len - 1)
                    .collect::<String>();
                if f_pre.is_empty() {
                    f_pre = finals[finals_len - 1].clone();
                }
                finals[finals_len - 1] = f_pre + "5";
            }
        }

        // 处理 "个" 字逻辑
        if let Some(ge_idx) = word.chars().position(|c| c == '个') {
            if ge_idx >= 1 {
                if let Some(prev_char) = word.chars().nth(ge_idx - 1) {
                    if prev_char.is_numeric()
                        || "几有两半多各整每做是".contains(prev_char)
                        || word == "个"
                    {
                        let f_pre = finals[ge_idx]
                            .chars()
                            .take(finals[ge_idx].chars().count() - 1)
                            .collect::<String>();
                        finals[ge_idx] = f_pre + "5";
                    }
                }
            }
        } else if MUST_NEURAL_TONE_WORDS.contains(word)
            || (word_len > 1
                && MUST_NEURAL_TONE_WORDS
                    .contains(&word.substring(word_len - 2, word_len).to_string()))
        {
            let f_pre = finals[finals_len - 1]
                .chars()
                .take(finals[finals_len - 1].chars().count() - 1)
                .collect::<String>();
            finals[finals_len - 1] = f_pre + "5";
        }
        let word_list = ToneSandhi::split_word(word, jieba_util);
        let w0_len = word_list[0].chars().count();

        let mut finals_list = [finals[..w0_len].to_vec(), finals[w0_len..].to_vec()];
        for (i, word) in word_list.iter().enumerate() {
            if MUST_NEURAL_TONE_WORDS.contains(word)
                || (word_len > 1
                    && MUST_NEURAL_TONE_WORDS
                        .contains(&word.substring(word_len - 2, word_len).to_string()))
            {
                let finals_list_i_len = finals_list[i].len();
                let f = &finals_list[i][finals_list_i_len - 1];
                let finals_list_ii_len = f.chars().count();
                let s = f.substring(0, finals_list_ii_len - 1).to_string() + "5";
                finals_list[i][finals_list_i_len - 1] = s;
            }
        }
        finals_list.iter().flatten().cloned().collect()
    }

    fn bu_sandhi(word: &str, finals: Vec<String>) -> Vec<String> {
        let mut finals = finals;

        let b0 = word.chars().count() == 3;
        let b1 = word.chars().nth(1).is_some();
        let b2 = {
            if b1 {
                word.chars().nth(1).unwrap() == '不'
            } else {
                false
            }
        };

        if b0 && b2 {
            if finals.len() > 1 {
                let f1_len = finals[1].chars().count();
                finals[1] = finals[1].substring(0, f1_len - 1).to_string() + "5";
            }
        } else {
            for (i, char) in word.chars().enumerate() {
                if finals.len() > i + 1 {
                    let fi_len = finals[i].chars().count();
                    let fi1_len = finals[i + 1].chars().count();
                    let b0 = finals[i + 1].chars().nth(fi1_len - 1).is_some();
                    let b1 = {
                        if b0 {
                            finals[i + 1].chars().nth(fi1_len - 1).unwrap() == '4'
                        } else {
                            false
                        }
                    };
                    if char == '不' && i + 1 < word.chars().count() && b1 {
                        finals[i] = finals[i].substring(0, fi_len - 1).to_string() + "2";
                    }
                }
            }
        }

        finals
    }

    fn yi_sandhi(&self, word: &str, mut finals: Vec<String>) -> Vec<String> {
        if word.is_empty() {
            return finals;
        }

        let w_len = word.chars().count();
        let w_0 = word.chars().nth(0).unwrap();
        let w_last = word.chars().nth(w_len - 1).unwrap();

        let b1 = word.contains("一");
        let b2 = word.chars().all(|wi| wi == '一' || wi.is_numeric());

        if b1 && b2 {
            return finals;
        }

        if w_len == 3 && word.chars().nth(1).unwrap() == '一' && w_0 == w_last {
            if finals.len() > 2 {
                finals[1] = finals[1][..finals[1].chars().count() - 1].to_string() + "5";
            }
        } else if word.starts_with("第一") {
            if finals.len() > 2 {
                finals[1] = finals[1][..finals[1].chars().count() - 1].to_string() + "1";
            }
        } else {
            for (i, char) in word.chars().enumerate() {
                if char == '一' && i + 1 < w_len && finals.len() > i + 1 {
                    let fi_len = finals[i].chars().count();
                    let fi1_len = finals[i + 1].chars().count();

                    // 修正1：将切片与字符串进行比较
                    if &finals[i + 1][fi1_len - 1..] == "4" {
                        finals[i] = finals[i][..fi_len - 1].to_string() + "2";
                    } else if let Some(next_char) = word.chars().nth(i + 1) {
                        // 修正2：直接传递 `next_char`，不再传递引用
                        if !PUNCTUATION.contains(next_char) {
                            finals[i] = finals[i][..fi_len - 1].to_string() + "4";
                        }
                    }
                }
            }
        }

        finals
    }

    fn three_sandhi(&self, word: &str, mut finals: Vec<String>, jieba_util: &Jieba) -> Vec<String> {
        if finals.is_empty() {
            return finals;
        }

        let update_tone = |tone: &mut String, new_tone: char| {
            let len = tone.chars().count();
            *tone = tone.chars().take(len - 1).collect::<String>() + &new_tone.to_string();
        };

        match word.chars().count() {
            2 if ToneSandhi::all_tone_three(&finals) => {
                update_tone(&mut finals[0], '2');
            }
            3 => {
                let word_list = ToneSandhi::split_word(word, jieba_util);
                if ToneSandhi::all_tone_three(&finals) && finals.len() >= 2 {
                    match word_list[0].chars().count() {
                        2 => {
                            update_tone(&mut finals[0], '2');
                            update_tone(&mut finals[1], '2');
                        }
                        1 => update_tone(&mut finals[1], '2'),
                        _ => {}
                    }
                } else {
                    let mut finals_list = vec![
                        finals[..word_list[0].chars().count()].to_vec(),
                        finals[word_list[0].chars().count()..].to_vec(),
                    ];

                    if finals_list.len() == 2 {
                        for i in 0..finals_list.len() {
                            let sub = &mut finals_list[i]; // 当前子列表
                            if ToneSandhi::all_tone_three(sub) && sub.len() == 2 {
                                // 如果当前子列表全为第三声，且长度为2，则将第一个改为第二声
                                let s0_len = sub[0].chars().count();
                                sub[0] = sub[0].chars().take(s0_len - 1).collect::<String>() + "2";
                            } else if i == 1 && !ToneSandhi::all_tone_three(sub) {
                                // 如果是第二个子列表，检查前一个子列表的最后一个音节
                                let (prev_part, current_part) = finals_list.split_at_mut(i);
                                if let Some(prev_last) =
                                    prev_part.last_mut().and_then(|prev| prev.last_mut())
                                {
                                    // 获取前一项的最后一个音节
                                    if prev_last.ends_with('3') && current_part[0][0].ends_with('3')
                                    {
                                        let prev_len = prev_last.chars().count();
                                        *prev_last = prev_last
                                            .chars()
                                            .take(prev_len - 1)
                                            .collect::<String>()
                                            + "2";
                                    }
                                }
                            }
                        }

                        finals = finals_list.into_iter().flatten().collect();
                    }
                }
            }
            4 => {
                let finals_list = vec![finals[..2].to_vec(), finals[2..].to_vec()];
                finals = finals_list
                    .into_iter()
                    .flat_map(|mut sub| {
                        if ToneSandhi::all_tone_three(&sub) {
                            update_tone(&mut sub[0], '2');
                        }
                        sub
                    })
                    .collect();
            }
            _ => {}
        }

        finals
    }

    fn split_word(word: &str, jieba_util: &Jieba) -> Vec<String> {
        let mut word_list = jieba_util.cut_for_search(word, true);
        word_list.sort_by(|&a, &b| a.chars().count().partial_cmp(&b.len()).unwrap());
        word_list.first().map_or(
            vec![], // 处理空列表
            |first_word| {
                let first_word_len = first_word.chars().count();
                if word.starts_with(first_word) {
                    let second_word = word.get(first_word_len..).unwrap_or("");
                    vec![first_word.to_string(), second_word.to_string()]
                } else {
                    let second_word: String = word
                        .chars()
                        .take(word.chars().count() - first_word_len)
                        .collect();
                    vec![second_word, first_word.to_string()]
                }
            },
        )
    }

    fn merge_bu(seg_cut: &[Tag]) -> Vec<(String, String)> {
        let mut result: Vec<(String, String)> = Vec::new();
        let mut last_word = String::new();

        for seg in seg_cut {
            let (word, pos) = (&seg.word, &seg.tag);

            // Combine "不" with the current word if applicable
            let merged_word = if last_word == "不" {
                format!("{}{}", last_word, word)
            } else {
                word.to_string()
            };

            // Add the word to the result if it's not "不"
            if merged_word != "不" {
                result.push((merged_word.clone(), pos.to_string()));
            }

            last_word = merged_word;
        }

        // Handle the case where the last word is "不"
        if last_word == "不" {
            result.push((last_word, "d".to_string()));
        }

        result
    }

    fn merge_yi(seg_cut: &[(String, String)]) -> Vec<(String, String)> {
        let mut result: Vec<(String, String)> = Vec::new();

        for (i, (word, pos)) in seg_cut.iter().enumerate() {
            // Case 1: Merge "一" with surrounding words under specific conditions
            if *word == "一"
                && i > 0
                && i + 1 < seg_cut.len()
                && seg_cut[i - 1].0 == seg_cut[i + 1].0
                && seg_cut[i - 1].1 == "v"
                && seg_cut[i + 1].1 == "v"
            {
                if let Some(last) = result.last_mut() {
                    last.0.push('一');
                    last.0.push_str(&seg_cut[i + 1].0);
                }
                continue;
            }

            // Case 2: Skip redundant "一" between identical verbs
            if i >= 2 && seg_cut[i - 1].0 == "一" && seg_cut[i - 2].0 == *word && pos == "v" {
                continue;
            }

            // Default case: Add current word and POS to the result
            result.push((word.clone(), pos.clone()));
        }

        // Merge remaining "一" with the next word if possible
        result.into_iter().fold(Vec::new(), |mut acc, (word, pos)| {
            if let Some(last) = acc.last_mut() {
                if last.0 == "一" {
                    last.0.push_str(&word);
                    return acc;
                }
            }
            acc.push((word, pos));
            acc
        })
    }

    fn all_tone_three(finals: &[String]) -> bool {
        finals.iter().all(|x| x.ends_with('3'))
    }

    fn get_pinyin(word: &str, with_five: bool) -> Vec<String> {
        word.to_pinyin()
            .enumerate()
            .map(|(i, p)| {
                if let Some(pinyin) = p {
                    let pn = pinyin.with_tone_num();
                    let py2 = get_finals(pn, true);
                    let (non_num, num): (String, String) =
                        py2.chars().partition(|c| !c.is_ascii_digit());
                    format!(
                        "{}{}",
                        non_num,
                        if num.trim().is_empty() && with_five {
                            "5"
                        } else {
                            &num
                        }
                    )
                } else {
                    word.chars().nth(i).unwrap().to_string()
                }
            })
            .collect()
    }

    fn is_reduplication(word: &str) -> bool {
        let chars: Vec<char> = word.chars().collect();
        chars.len() == 2 && chars[0] == chars[1]
    }

    fn merge_continuous_three_tones(seg_cut: &[(String, String)]) -> Vec<(String, String)> {
        let sub_finals_list: Vec<Vec<String>> = seg_cut
            .iter()
            .map(|(word, _)| {
                let pinyin = ToneSandhi::get_pinyin(word, true);
                if pinyin.is_empty() {
                    vec![word.clone()]
                } else {
                    pinyin
                }
            })
            .collect();

        let mut new_seg: Vec<(String, String)> = Vec::with_capacity(seg_cut.len());
        let mut merge_last = vec![false; seg_cut.len()];

        for (i, (word, pos)) in seg_cut.iter().enumerate() {
            if i > 0 {
                let b1 = ToneSandhi::all_tone_three(&sub_finals_list[i - 1]);
                let b2 = ToneSandhi::all_tone_three(&sub_finals_list[i]);
                let b3 = merge_last[i - 1];

                if b1 && b2 && b3 {
                    let prev_word = &seg_cut[i - 1].0;
                    if !ToneSandhi::is_reduplication(prev_word)
                        && prev_word.chars().count() + word.chars().count() <= 3
                    {
                        if let Some(last) = new_seg.last_mut() {
                            last.0.push_str(word);
                        }
                        merge_last[i] = true;
                        continue;
                    }
                }
            }
            new_seg.push((word.clone(), pos.clone()));
        }

        new_seg
    }

    fn merge_continuous_three_tones_2(seg_cut: &[(String, String)]) -> Vec<(String, String)> {
        let mut new_seg: Vec<(String, String)> = Vec::new();
        let sub_finals_list: Vec<Vec<String>> = seg_cut
            .iter()
            .map(|(word, _)| {
                let pinyin = ToneSandhi::get_pinyin(word, true);
                if pinyin.is_empty() {
                    vec![word.clone()]
                } else {
                    pinyin
                }
            })
            .collect();

        let mut merge_last = vec![false; seg_cut.len()];

        for (i, (word, pos)) in seg_cut.iter().enumerate() {
            if i > 0 {
                let prev_finals = &sub_finals_list[i - 1];
                let curr_finals = &sub_finals_list[i];

                let prev_tone_is_three = prev_finals
                    .last()
                    .and_then(|f| f.chars().last())
                    .map_or(false, |c| c == '3');
                let curr_tone_is_three = curr_finals
                    .first()
                    .and_then(|f| f.chars().last())
                    .map_or(false, |c| c == '3');

                if prev_tone_is_three
                    && curr_tone_is_three
                    && merge_last[i - 1]
                    && !ToneSandhi::is_reduplication(&seg_cut[i - 1].0)
                    && seg_cut[i - 1].0.chars().count() + word.chars().count() <= 3
                {
                    if let Some((last_word, _)) = new_seg.last_mut() {
                        last_word.push_str(word);
                    }
                    merge_last[i] = true;
                    continue;
                }
            }
            new_seg.push((word.clone(), pos.clone()));
        }

        new_seg
    }

    fn merge_er(seg_cut: &[(String, String)]) -> Vec<(String, String)> {
        let mut new_seg: Vec<(String, String)> = Vec::new();

        for (i, (word, pos)) in seg_cut.iter().enumerate() {
            if i > 0 && word == "儿" && seg_cut[i - 1].0 != "#" {
                if let Some((last_word, _)) = new_seg.last_mut() {
                    last_word.push_str(word);
                }
            } else {
                new_seg.push((word.clone(), pos.clone()));
            }
        }

        new_seg
    }

    fn merge_reduplication(seg_cut: &[(String, String)]) -> Vec<(String, String)> {
        let mut new_seg: Vec<(String, String)> = Vec::new();

        for (word, pos) in seg_cut {
            if let Some((last_word, _)) = new_seg.last_mut() {
                if last_word == word {
                    last_word.push_str(word);
                    continue;
                }
            }
            new_seg.push((word.clone(), pos.clone()));
        }

        new_seg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jieba_rs::{Jieba, Tag};

    #[test]
    fn test_neural_sandhi() {
        let tone_sandhi = ToneSandhi::new();
        let jieba = Jieba::new();

        // 测试单个字符的情况
        let word = "了".to_string();
        let pos = "ul";
        let finals = vec!["le4".to_string()];
        let result = tone_sandhi.neural_sandhi(&word, pos, finals.clone(), &jieba);
        assert_eq!(result, vec!["le5".to_string()]);

        // 测试两个相同字符的情况
        let word = "来来".to_string();
        let pos = "v";
        let finals = vec!["lai2".to_string(), "lai2".to_string()];
        let result = tone_sandhi.neural_sandhi(&word, pos, finals.clone(), &jieba);
        assert_eq!(result, vec!["lai2".to_string(), "lai5".to_string()]);

        // 测试包含 '个' 的情况
        let word = "几个".to_string();
        let pos = "m";
        let finals = vec!["ji3".to_string(), "ge4".to_string()];
        let result = tone_sandhi.neural_sandhi(&word, pos, finals.clone(), &jieba);
        assert_eq!(result, vec!["ji3".to_string(), "ge5".to_string()]);

        // 测试 must_neural_tone_words 中的词
        let word = "麻烦".to_string();
        let pos = "n";
        let finals = vec!["ma2".to_string(), "fan2".to_string()];
        let result = tone_sandhi.neural_sandhi(&word, pos, finals.clone(), &jieba);
        assert_eq!(result, vec!["ma2".to_string(), "fan5".to_string()]);

        // 测试 must_not_neural_tone_words 中的词
        let word = "男子".to_string();
        let pos = "n";
        let finals = vec!["nan2".to_string(), "zi3".to_string()];
        let result = tone_sandhi.neural_sandhi(&word, pos, finals.clone(), &jieba);
        assert_eq!(result, vec!["nan2".to_string(), "zi3".to_string()]);

        // 测试以 '的' 结尾的情况
        let word = "漂亮的".to_string();
        let pos = "a";
        let finals = vec!["piao4".to_string(), "liang4".to_string(), "de5".to_string()];
        let result = tone_sandhi.neural_sandhi(&word, pos, finals.clone(), &jieba);
        assert_eq!(
            result,
            vec!["piao4".to_string(), "liang5".to_string(), "de5".to_string()]
        );

        // 测试以 '吧' 结尾的情况
        let word = "吃吧".to_string();
        let pos = "v";
        let finals = vec!["chi1".to_string(), "ba5".to_string()];
        let result = tone_sandhi.neural_sandhi(&word, pos, finals.clone(), &jieba);
        assert_eq!(result, vec!["chi1".to_string(), "ba5".to_string()]);

        // 测试以 '来' 结尾且满足条件的情况
        let word = "进来".to_string();
        let pos = "v";
        let finals = vec!["jin4".to_string(), "lai2".to_string()];
        let result = tone_sandhi.neural_sandhi(&word, pos, finals.clone(), &jieba);
        assert_eq!(result, vec!["jin4".to_string(), "lai5".to_string()]);

        // 测试包含 '们' 且满足条件的情况
        let word = "人们".to_string();
        let pos = "n";
        let finals = vec!["ren2".to_string(), "men5".to_string()];
        let result = tone_sandhi.neural_sandhi(&word, pos, finals.clone(), &jieba);
        assert_eq!(result, vec!["ren2".to_string(), "men5".to_string()]);

        // 测试包含 '上下里' 且满足条件的情况
        let word = "家里".to_string();
        let pos = "s";
        let finals = vec!["jia1".to_string(), "li5".to_string()];
        let result = tone_sandhi.neural_sandhi(&word, pos, finals.clone(), &jieba);
        assert_eq!(result, vec!["jia1".to_string(), "li5".to_string()]);

        // 测试包含 '上下里' 且满足条件的情况
        let word = "地面上".to_string();
        let pos = "s";
        let finals = vec!["di4".to_string(), "mian4".to_string(), "shang4".to_string()];
        let result = tone_sandhi.neural_sandhi(&word, pos, finals.clone(), &jieba);
        assert_eq!(
            result,
            vec!["di4".to_string(), "mian4".to_string(), "shang5".to_string()]
        );

        // 测试空字符串
        let word = "".to_string();
        let pos = "";
        let finals = vec![];
        let result = tone_sandhi.neural_sandhi(&word, pos, finals.clone(), &jieba);
        assert!(result.is_empty());
    }

    #[test]
    fn test_bu_sandhi() {
        // 测试正常情况
        let word = "不是".to_string();
        let finals = vec!["bu4".to_string(), "shi4".to_string()];
        let result = ToneSandhi::bu_sandhi(&word, finals.clone());
        assert_eq!(result, vec!["bu2".to_string(), "shi4".to_string()]);

        // 测试 '不' 在中间的情况
        let word = "好不好".to_string();
        let finals = vec!["hao3".to_string(), "bu4".to_string(), "hao3".to_string()];
        let result = ToneSandhi::bu_sandhi(&word, finals.clone());
        assert_eq!(
            result,
            vec!["hao3".to_string(), "bu5".to_string(), "hao3".to_string()]
        );

        // 测试 '不' 在开头且长度为 3 的情况
        let word = "不明白".to_string();
        let finals = vec!["bu4".to_string(), "ming2".to_string(), "bai2".to_string()];
        let result = ToneSandhi::bu_sandhi(&word, finals.clone());
        assert_eq!(
            result,
            vec!["bu4".to_string(), "ming2".to_string(), "bai2".to_string()]
        );

        // 测试 '不' 不在开头且后面不跟第四声的情况
        let word = "行不行".to_string();
        let finals = vec!["xing2".to_string(), "bu4".to_string(), "xing2".to_string()];
        let result = ToneSandhi::bu_sandhi(&word, finals.clone());
        assert_eq!(
            result,
            vec!["xing2".to_string(), "bu5".to_string(), "xing2".to_string()]
        );

        // 测试空字符串
        let word = "".to_string();
        let finals = vec![];
        let result = ToneSandhi::bu_sandhi(&word, finals.clone());
        assert!(result.is_empty());
    }

    #[test]
    fn test_yi_sandhi() {
        let tone_sandhi = ToneSandhi::new();

        // 测试正常情况
        let word = "一个".to_string();
        let finals = vec!["yi2".to_string(), "ge4".to_string()];
        let result = tone_sandhi.yi_sandhi(&word, finals.clone());
        assert_eq!(result, vec!["yi2".to_string(), "ge4".to_string()]);

        // 测试 '第一' 的情况
        let word = "第一".to_string();
        let finals = vec!["di4".to_string(), "yi1".to_string()];
        let result = tone_sandhi.yi_sandhi(&word, finals.clone());
        assert_eq!(result, vec!["di4".to_string(), "yi1".to_string()]);

        // 测试多个 '一' 的情况
        let word = "一二三".to_string();
        let finals = vec!["yi1".to_string(), "er4".to_string(), "san1".to_string()];
        let result = tone_sandhi.yi_sandhi(&word, finals.clone());
        assert_eq!(
            result,
            vec!["yi2".to_string(), "er4".to_string(), "san1".to_string()]
        );

        // 测试 '一' 在中间且前后字符相同的情况
        let word = "看一看".to_string();
        let finals = vec!["kan4".to_string(), "yi2".to_string(), "kan4".to_string()];
        let result = tone_sandhi.yi_sandhi(&word, finals.clone());
        assert_eq!(
            result,
            vec!["kan4".to_string(), "yi5".to_string(), "kan4".to_string()]
        );

        // 测试 '一' 在开头且前后字符不同的情况
        let word = "一次".to_string();
        let finals = vec!["yi2".to_string(), "ci4".to_string()];
        let result = tone_sandhi.yi_sandhi(&word, finals.clone());
        assert_eq!(result, vec!["yi2".to_string(), "ci4".to_string()]);

        // 测试空字符串
        let word = "".to_string();
        let finals = vec![];
        let result = tone_sandhi.yi_sandhi(&word, finals.clone());
        assert!(result.is_empty());
    }

    #[test]
    fn test_split_word() {
        let jieba = Jieba::new();

        // 测试正常情况
        let word = "中国人".to_string();
        let result = ToneSandhi::split_word(&word, &jieba);
        assert_eq!(result, vec!["中国".to_string(), "人".to_string()]);

        // 测试单字
        let word = "我".to_string();
        let result = ToneSandhi::split_word(&word, &jieba);
        assert_eq!(result, vec!["我".to_string(), "".to_string()]);

        // 测试两个字的词
        let word = "你好".to_string();
        let result = ToneSandhi::split_word(&word, &jieba);
        assert_eq!(result, vec!["你好".to_string(), "".to_string()]);

        // 测试相同长度的拆分
        let word = "好好".to_string();
        let result = ToneSandhi::split_word(&word, &jieba);
        assert_eq!(result, vec!["好好".to_string(), "".to_string()]);

        // 测试包含特殊字符
        let word = "测试-中".to_string();
        let result = ToneSandhi::split_word(&word, &jieba);
        assert_eq!(result, vec!["测试-".to_string(), "-".to_string()]);

        // 测试空字符串
        let word = "".to_string();
        let result = ToneSandhi::split_word(&word, &jieba);
        assert!(result.is_empty());
    }

    #[test]
    fn test_three_sandhi() {
        let tone_sandhi = ToneSandhi::new();
        let jieba = Jieba::new();

        // 测试两个字且都是三声的情况
        let word = "水果".to_string();
        let finals = vec!["shui3".to_string(), "guo3".to_string()];
        let result = tone_sandhi.three_sandhi(&word, finals.clone(), &jieba);
        assert_eq!(result, vec!["shui2".to_string(), "guo3".to_string()]);

        // 测试三个字且都是三声的情况
        let word = "管理者".to_string();
        let finals = vec!["guan3".to_string(), "li3".to_string(), "zhe3".to_string()];
        let result = tone_sandhi.three_sandhi(&word, finals.clone(), &jieba);
        assert_eq!(
            result,
            vec!["guan3".to_string(), "li3".to_string(), "zhe3".to_string()]
        );

        // 测试三个字且部分三声的情况
        let word = "管理者们".to_string();
        let finals = vec![
            "guan3".to_string(),
            "li3".to_string(),
            "zhe3".to_string(),
            "men5".to_string(),
        ];
        let result = tone_sandhi.three_sandhi(&word, finals.clone(), &jieba);
        assert_eq!(
            result,
            vec![
                "guan2".to_string(),
                "li3".to_string(),
                "zhe3".to_string(),
                "men5".to_string()
            ]
        );

        // 测试四个字且都是三声的情况
        let word = "管理者们".to_string();
        let finals = vec![
            "guan3".to_string(),
            "li3".to_string(),
            "zhe3".to_string(),
            "men5".to_string(),
        ];
        let result = tone_sandhi.three_sandhi(&word, finals.clone(), &jieba);
        assert_eq!(
            result,
            vec![
                "guan2".to_string(),
                "li3".to_string(),
                "zhe3".to_string(),
                "men5".to_string()
            ]
        );

        // 测试空字符串
        let word = "".to_string();
        let finals = vec![];
        let result = tone_sandhi.three_sandhi(&word, finals.clone(), &jieba);
        assert!(result.is_empty());
    }

    #[test]
    fn test_merge_bu() {
        let tags = vec![
            Tag {
                word: "不",
                tag: "d",
            },
            Tag {
                word: "是",
                tag: "v",
            },
        ];
        let result = ToneSandhi::merge_bu(&tags);
        assert_eq!(result, vec![("不是".to_string(), "v".to_string(),)]);

        let tags = vec![
            Tag {
                word: "不",
                tag: "d",
            },
            Tag {
                word: "不",
                tag: "d",
            },
            Tag {
                word: "行",
                tag: "v",
            },
        ];
        let result = ToneSandhi::merge_bu(&tags);
        assert_eq!(
            result,
            vec![
                ("不不".to_string(), "d".to_string(),),
                ("行".to_string(), "v".to_string(),)
            ]
        );

        let tags = vec![
            Tag {
                word: "行",
                tag: "v",
            },
            Tag {
                word: "不",
                tag: "d",
            },
        ];
        let result = ToneSandhi::merge_bu(&tags);
        assert_eq!(
            result,
            vec![
                ("行".to_string(), "v".to_string(),),
                ("不".to_string(), "d".to_string(),)
            ]
        );
    }

    #[test]
    fn test_merge_yi() {
        let tags = vec![
            ("一".to_string(), "v".to_string()),
            ("看".to_string(), "v".to_string()),
            ("一".to_string(), "v".to_string()),
            ("看".to_string(), "v".to_string()),
        ];
        let result = ToneSandhi::merge_yi(&tags);
        assert_eq!(result, vec![("一看一看".to_string(), "v".to_string(),)]);

        let tags = vec![
            ("看".to_string(), "v".to_string()),
            ("一".to_string(), "v".to_string()),
            ("看".to_string(), "v".to_string()),
        ];
        let result = ToneSandhi::merge_yi(&tags);
        assert_eq!(result, vec![("看一看".to_string(), "v".to_string(),),]);

        let tags = vec![
            ("一".to_string(), "v".to_string()),
            ("看".to_string(), "v".to_string()),
        ];
        let result = ToneSandhi::merge_yi(&tags);
        assert_eq!(result, vec![("一看".to_string(), "v".to_string(),)]);
    }
}
