use jieba_rs::{Jieba, Tag};
use pinyin::ToPinyin;
use substring::Substring;

use super::lazy_pinyin::style::get_finals;

pub struct ToneSandhi {
    pub must_neural_tone_words: Vec<String>,
    pub must_not_neural_tone_words: Vec<String>,
    pub punc: String,
}

impl ToneSandhi {
    pub fn init() -> Self {
        let must_neural_tone_words = {
            [
                "麻烦", "麻利", "鸳鸯", "高粱", "骨头", "骆驼", "马虎", "首饰", "馒头", "馄饨",
                "风筝", "难为", "队伍", "阔气", "闺女", "门道", "锄头", "铺盖", "铃铛", "铁匠",
                "钥匙", "里脊", "里头", "部分", "那么", "道士", "造化", "迷糊", "连累", "这么",
                "这个", "运气", "过去", "软和", "转悠", "踏实", "跳蚤", "跟头", "趔趄", "财主",
                "豆腐", "讲究", "记性", "记号", "认识", "规矩", "见识", "裁缝", "补丁", "衣裳",
                "衣服", "衙门", "街坊", "行李", "行当", "蛤蟆", "蘑菇", "薄荷", "葫芦", "葡萄",
                "萝卜", "荸荠", "苗条", "苗头", "苍蝇", "芝麻", "舒服", "舒坦", "舌头", "自在",
                "膏药", "脾气", "脑袋", "脊梁", "能耐", "胳膊", "胭脂", "胡萝", "胡琴", "胡同",
                "聪明", "耽误", "耽搁", "耷拉", "耳朵", "老爷", "老实", "老婆", "老头", "老太",
                "翻腾", "罗嗦", "罐头", "编辑", "结实", "红火", "累赘", "糨糊", "糊涂", "精神",
                "粮食", "簸箕", "篱笆", "算计", "算盘", "答应", "笤帚", "笑语", "笑话", "窟窿",
                "窝囊", "窗户", "稳当", "稀罕", "称呼", "秧歌", "秀气", "秀才", "福气", "祖宗",
                "砚台", "码头", "石榴", "石头", "石匠", "知识", "眼睛", "眯缝", "眨巴", "眉毛",
                "相声", "盘算", "白净", "痢疾", "痛快", "疟疾", "疙瘩", "疏忽", "畜生", "生意",
                "甘蔗", "琵琶", "琢磨", "琉璃", "玻璃", "玫瑰", "玄乎", "狐狸", "状元", "特务",
                "牲口", "牙碜", "牌楼", "爽快", "爱人", "热闹", "烧饼", "烟筒", "烂糊", "点心",
                "炊帚", "灯笼", "火候", "漂亮", "滑溜", "溜达", "温和", "清楚", "消息", "浪头",
                "活泼", "比方", "正经", "欺负", "模糊", "槟榔", "棺材", "棒槌", "棉花", "核桃",
                "栅栏", "柴火", "架势", "枕头", "枇杷", "机灵", "本事", "木头", "木匠", "朋友",
                "月饼", "月亮", "暖和", "明白", "时候", "新鲜", "故事", "收拾", "收成", "提防",
                "挖苦", "挑剔", "指甲", "指头", "拾掇", "拳头", "拨弄", "招牌", "招呼", "抬举",
                "护士", "折腾", "扫帚", "打量", "打算", "打点", "打扮", "打听", "打发", "扎实",
                "扁担", "戒指", "懒得", "意识", "意思", "情形", "悟性", "怪物", "思量", "怎么",
                "念头", "念叨", "快活", "忙活", "志气", "心思", "得罪", "张罗", "弟兄", "开通",
                "应酬", "庄稼", "干事", "帮手", "帐篷", "希罕", "师父", "师傅", "巴结", "巴掌",
                "差事", "工夫", "岁数", "屁股", "尾巴", "少爷", "小气", "小伙", "将就", "对头",
                "对付", "寡妇", "家伙", "客气", "实在", "官司", "学问", "学生", "字号", "嫁妆",
                "媳妇", "媒人", "婆家", "娘家", "委屈", "姑娘", "姐夫", "妯娌", "妥当", "妖精",
                "奴才", "女婿", "头发", "太阳", "大爷", "大方", "大意", "大夫", "多少", "多么",
                "外甥", "壮实", "地道", "地方", "在乎", "困难", "嘴巴", "嘱咐", "嘟囔", "嘀咕",
                "喜欢", "喇嘛", "喇叭", "商量", "唾沫", "哑巴", "哈欠", "哆嗦", "咳嗽", "和尚",
                "告诉", "告示", "含糊", "吓唬", "后头", "名字", "名堂", "合同", "吆喝", "叫唤",
                "口袋", "厚道", "厉害", "千斤", "包袱", "包涵", "匀称", "勤快", "动静", "动弹",
                "功夫", "力气", "前头", "刺猬", "刺激", "别扭", "利落", "利索", "利害", "分析",
                "出息", "凑合", "凉快", "冷战", "冤枉", "冒失", "养活", "关系", "先生", "兄弟",
                "便宜", "使唤", "佩服", "作坊", "体面", "位置", "似的", "伙计", "休息", "什么",
                "人家", "亲戚", "亲家", "交情", "云彩", "事情", "买卖", "主意", "丫头", "丧气",
                "两口", "东西", "东家", "世故", "不由", "不在", "下水", "下巴", "上头", "上司",
                "丈夫", "丈人", "一辈", "那个", "菩萨", "父亲", "母亲", "咕噜", "邋遢", "费用",
                "冤家", "甜头", "介绍", "荒唐", "大人", "泥鳅", "幸福", "熟悉", "计划", "扑腾",
                "蜡烛", "姥爷", "照顾", "喉咙", "吉他", "弄堂", "蚂蚱", "凤凰", "拖沓", "寒碜",
                "糟蹋", "倒腾", "报复", "逻辑", "盘缠", "喽啰", "牢骚", "咖喱", "扫把", "惦记",
            ]
        };
        let must_not_neural_tone_words = {
            [
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
        };

        let must_neural_tone_words: Vec<String> = must_neural_tone_words
            .iter()
            .map(|s| s.to_string())
            .collect();
        let must_not_neural_tone_words: Vec<String> = must_not_neural_tone_words
            .iter()
            .map(|s| s.to_string())
            .collect();

        let punc = "：，；。？！“”‘’':,;.?!".to_string();

        ToneSandhi {
            must_neural_tone_words,
            must_not_neural_tone_words,
            punc,
        }
    }

    fn _neural_sandhi(
        &self,
        word: &String,
        pos: &str,
        finals: Vec<String>,
        jieba_util: &Jieba,
    ) -> Vec<String> {
        let mut finals = finals;
        (0..word.chars().count()).for_each(|j| {
            if j >= 1 {
                let have_item2 = pos.chars().nth(0).is_some();
                if have_item2 {
                    let item = word.chars().nth(j).unwrap();
                    let pre_item = word.chars().nth(j - 1).unwrap();
                    let b1 = item == pre_item;
                    let p0 = pos.chars().nth(0).unwrap();
                    let b2 = p0 == 'n' || p0 == 'v' || p0 == 'a';
                    let b3 = !self.must_not_neural_tone_words.contains(word);
                    if b1 && b2 && b3 {
                        let f_len = finals[j].chars().count();
                        let mut f_pre = finals[j].substring(0, f_len - 1).to_string();
                        if f_pre.is_empty() {
                            f_pre = finals[j].clone();
                        }
                        finals[j] = f_pre + "5"
                    }
                }
            }
        });

        let ge_idx: i32 = {
            let r = word.chars().position(|c| c == '个');
            if r.is_none() {
                -1
            } else {
                Option::unwrap(r) as i32
            }
        };

        let word_len = word.chars().count();
        let finals_len = finals.len();
        if word_len == 0 || finals_len == 0 {
            return finals;
        }

        let word_last_item = word.chars().nth(word_len - 1).unwrap();

        if word.chars().count() >= 1
            && "吧呢哈啊呐噻嘛吖嗨呐哦哒额滴哩哟喽啰耶喔诶".contains(word_last_item)
        {
            let f_len = finals[finals_len - 1].chars().count();
            let f_pre = finals[finals_len - 1].substring(0, f_len - 1).to_string();
            finals[finals_len - 1] = f_pre + "5"
        } else if word.chars().count() >= 1 && "的地得".contains(word_last_item) {
            let f_len = finals[finals_len - 1].chars().count();
            let mut f_pre = finals[finals_len - 1].substring(0, f_len - 1).to_string();
            if f_pre.is_empty() {
                f_pre = finals[finals_len - 1].clone();
            }
            finals[finals_len - 1] = f_pre + "5"
        } else if word.chars().count() > 1
            && "来去".contains(word_last_item)
            && (word_len > 1
                && "上下进出回过起开".contains(word.chars().nth(word_len - 2).unwrap()))
        {
            let f_len = finals[finals_len - 1].chars().count();
            let f_pre = finals[finals_len - 1].substring(0, f_len - 1).to_string();
            finals[finals_len - 1] = f_pre + "5"
        } else if (ge_idx >= 1
            && (word
                .chars()
                .nth((ge_idx - 1) as usize)
                .unwrap()
                .is_numeric()
                || "几有两半多各整每做是"
                    .contains(word.chars().nth((ge_idx - 1) as usize).unwrap())))
            || word == "个"
        {
            let ge_idx = ge_idx as usize;
            let f_len = finals[ge_idx].chars().count();
            let f_pre = finals[ge_idx].substring(0, f_len - 1).to_string();
            finals[ge_idx] = f_pre + "5"
        } else if (self.must_neural_tone_words.contains(word)
            || (word_len > 1
                && self
                    .must_neural_tone_words
                    .contains(&word.substring(word_len - 2, word_len).to_string())))
            || (word.chars().count() == 1
                && "了着过".contains(word)
                && ["ul", "uz", "ug"].contains(&pos))
            || (word.chars().count() > 1
                && "们子".contains(word_last_item)
                && ["r", "n"].contains(&pos)
                && !self.must_not_neural_tone_words.contains(word))
            || (word.chars().count() > 1
                && "上下里".contains(word_last_item)
                && ["s", "l", "f"].contains(&pos))
        {
            let f_len = finals[finals_len - 1].chars().count();
            let f_pre = finals[finals_len - 1].substring(0, f_len - 1).to_string();
            finals[finals_len - 1] = f_pre + "5"
        }

        let word_list = ToneSandhi::_split_word(word, jieba_util);
        let w0_len = word_list[0].chars().count();

        let mut finals_list = vec![finals[..w0_len].to_vec(), finals[w0_len..].to_vec()];
        for (i, word) in word_list.iter().enumerate() {
            if self.must_neural_tone_words.contains(word)
                || (word_len > 1
                    && self
                        .must_neural_tone_words
                        .contains(&word.substring(word_len - 2, word_len).to_string()))
            {
                let finals_list_i_len = finals_list[i].len();
                let f = &finals_list[i][finals_list_i_len - 1];
                let finals_list_ii_len = f.chars().count();
                let s = f.substring(0, finals_list_ii_len - 1).to_string() + "5";
                finals_list[i][finals_list_i_len - 1] = s;
            }
        }
        let mut finals: Vec<String> = vec![];
        for f_list in &finals_list {
            for fi in f_list {
                finals.push(fi.clone());
            }
        }
        finals
    }

    fn _bu_sandhi(word: &str, finals: Vec<String>) -> Vec<String> {
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

    fn _yi_sandhi(&self, word: &str, finals: Vec<String>) -> Vec<String> {
        let mut finals = finals;
        let b1 = word.contains("一");
        let b2 = {
            let mut b = true;
            for wi in word.chars() {
                if wi != '一' && !wi.is_numeric() {
                    b = false;
                    break;
                }
            }
            b
        };
        let w_len = word.chars().count();
        if w_len == 0 {
            return finals;
        }
        let w_0 = word.chars().nth(0).unwrap();
        let w_last = word.chars().nth(w_len - 1).unwrap();
        if b1 && b2 {
            return finals;
        } else if word.chars().count() == 3 && word.chars().nth(1).unwrap() == '一' && w_0 == w_last
        {
            if finals.len() > 2 {
                let f1_len = finals[1].chars().count();
                finals[1] = finals[1].substring(0, f1_len - 1).to_string() + "5";
            }
        } else if word.starts_with("第一") {
            if finals.len() > 2 {
                let f1_len = finals[1].chars().count();
                finals[1] = finals[1].substring(0, f1_len - 1).to_string() + "1";
            }
        } else {
            for (i, char) in word.chars().enumerate() {
                if char == '一' && i + 1 < word.chars().count() && finals.len() > i + 1 {
                    let fi_len = finals[i].chars().count();
                    let fi1_len = finals[i + 1].chars().count();
                    if finals[i + 1].substring(fi1_len - 1, fi1_len) == "4" {
                        finals[i] = finals[i].substring(0, fi_len - 1).to_string() + "2";
                    } else {
                        let b0 = word.chars().nth(i + 1).is_some();
                        if b0 && !self.punc.contains(word.chars().nth(i + 1).unwrap()) {
                            finals[i] = finals[i].substring(0, fi_len - 1).to_string() + "4";
                        }
                    }
                }
            }
        }

        finals
    }

    fn _split_word(word: &String, jieba_util: &Jieba) -> Vec<String> {
        let mut word_list = jieba_util.cut_for_search(word, true);
        word_list.sort_by(|&a, &b| a.chars().count().partial_cmp(&b.len()).unwrap());
        let first_subword = word_list[0];
        let first_begin = {
            // 中文3个位置
            let res = word.find(first_subword);
            if res.is_none() {
                false
            } else {
                Option::unwrap(res) == 0
            }
        };
        let word_len = word.chars().count();
        let new_word_list = {
            if first_begin {
                let second_subword = {
                    if first_subword.chars().count() != word.chars().count() {
                        word.substring(first_subword.chars().count(), word_len)
                    } else {
                        ""
                    }
                };
                let new_word_list = vec![first_subword.to_string(), second_subword.to_string()];
                new_word_list
            } else {
                let l = word.chars().count();
                let second_subword = {
                    if first_subword.chars().count() != word.chars().count() {
                        word.substring(0, l - first_subword.chars().count())
                    } else {
                        ""
                    }
                };
                let new_word_list = vec![second_subword.to_string(), first_subword.to_string()];
                new_word_list
            }
        };

        new_word_list
    }

    fn _three_sandhi(&self, word: &String, finals: Vec<String>, jieba_util: &Jieba) -> Vec<String> {
        if finals.is_empty() {
            return finals;
        }
        let mut finals = finals;
        let f0_len = finals[0].chars().count();
        if word.chars().count() == 2 && ToneSandhi::_all_tone_three(&finals) {
            finals[0] = finals[0].substring(0, f0_len - 1).to_string() + "2";
        } else if word.chars().count() == 3 {
            let word_list = ToneSandhi::_split_word(word, jieba_util);
            if ToneSandhi::_all_tone_three(&finals) {
                if finals.len() >= 2 {
                    let f1_len = finals[1].chars().count();
                    if word_list[0].chars().count() == 2 {
                        finals[0] = finals[0].substring(0, f0_len - 1).to_string() + "2";
                        finals[1] = finals[1].substring(0, f1_len - 1).to_string() + "2";
                    } else if word_list[0].chars().count() == 1 {
                        finals[1] = finals[1].substring(0, f1_len - 1).to_string() + "2";
                    }
                }
            } else {
                let mut finals_list = vec![
                    finals[..word_list[0].chars().count()].to_vec(),
                    finals[word_list[0].chars().count()..].to_vec(),
                ];
                if finals_list.len() == 2 {
                    for i in 0..finals_list.len() {
                        let sub = &finals_list[i];
                        if ToneSandhi::_all_tone_three(sub) && sub.len() == 2 {
                            let fi0_len = finals_list[i][0].chars().count();
                            finals_list[i][0] =
                                finals_list[i][0].substring(0, fi0_len - 1).to_string() + "2";
                        } else if i == 1 && !ToneSandhi::_all_tone_three(sub) {
                            let f0_len = finals_list[0].len();
                            let f0_last_len = finals_list[0][f0_len - 1].chars().count();
                            let fi0_len = finals_list[i][0].chars().count();
                            if finals_list[i][0].substring(fi0_len - 1, fi0_len) == "3"
                                && finals_list[0][f0_len - 1]
                                    .substring(f0_last_len - 1, f0_last_len)
                                    == "3"
                            {
                                finals_list[0][f0_len - 1] = finals_list[0][f0_len - 1]
                                    .substring(0, f0_last_len - 1)
                                    .to_string()
                                    + "2";
                            }
                        }

                        finals = vec![];
                        for f_list in &finals_list {
                            for fi in f_list {
                                finals.push(fi.clone());
                            }
                        }
                    }
                }
            }
        } else if word.chars().count() == 4 {
            let finals_list = vec![finals[..2].to_vec(), finals[2..].to_vec()];
            finals = vec![];
            for mut sub in finals_list {
                if ToneSandhi::_all_tone_three(&sub) {
                    let s0_len = sub[0].chars().count();
                    sub[0] = sub[0].substring(0, s0_len - 1).to_string() + "2";
                }
                finals.append(&mut sub);
            }
        }

        finals
    }

    pub fn _merge_bu(seg_cut: &Vec<Tag>) -> Vec<(String, String)> {
        let mut new_seg: Vec<(String, String)> = vec![];
        let mut last_word = "".to_string();
        for seg in seg_cut {
            let (mut word, pos) = (seg.word.to_string(), seg.tag.to_string());
            if last_word == "不" {
                word = last_word + &word;
            }
            if word != "不" {
                new_seg.push((word.clone(), pos));
            }
            last_word = word.clone();
        }
        if last_word == "不" {
            new_seg.push((last_word, "d".to_string()));
        }

        new_seg
    }

    pub fn _merge_yi(seg_cut: &[(String, String)]) -> Vec<(String, String)> {
        let mut new_seg: Vec<(String, String)> = vec![];
        for i in 0..seg_cut.len() {
            let (word, pos) = &seg_cut[i];
            if i >= 1
                && word == "一"
                && i + 1 < seg_cut.len()
                && seg_cut[i - 1].0 == seg_cut[i + 1].0
                && seg_cut[i - 1].1 == "v"
                && seg_cut[i + 1].1 == "v"
                && i - 1 < new_seg.len()
            {
                let mut a = new_seg[i - 1].0.clone();
                let b = &new_seg[i - 1].0;
                a.push('一');
                a.push_str(b);

                new_seg[i - 1].0 = a;
                continue;
            }
            if i >= 2 && seg_cut[i - 1].0 == "一" && &seg_cut[i - 2].0 == word && pos == "v" {
                continue;
            } else {
                new_seg.push((word.clone(), pos.clone()));
            }
        }
        let mut new_seg2: Vec<(String, String)> = vec![];
        (0..new_seg.len()).for_each(|i| {
            let l2 = new_seg2.len();
            let (word, pos) = &new_seg[i];
            if l2 > 0 && new_seg2[l2 - 1].0 == "一" {
                new_seg2[l2 - 1].0 = new_seg2[l2 - 1].0.clone() + word;
            } else {
                new_seg2.push((word.clone(), pos.clone()));
            }
        });
        new_seg2
    }

    fn _all_tone_three(finals: &Vec<String>) -> bool {
        let mut res = true;
        for x in finals {
            let len = x.chars().count();
            let final_str = x.substring(len - 1, len);
            if final_str != "3" {
                res = false;
                break;
            }
        }
        res
    }

    // 获取拼音
    fn get_pinyin(word: &str, _with_five: bool) -> Vec<String> {
        // let word_pinyin = pinyin::lazy_pinyin(
        //     word,
        //     &pinyin::Args {
        //         style: pinyin::Style::Initials,
        //         heteronym: false,
        //     },
        // )
        // .into_iter()
        // .map(|s| {
        //     let (non_num, num): (String, String) = s.chars().partition(|c| !c.is_ascii_digit());
        //     if _with_five && num.is_empty() {
        //         format!("{}{}", non_num, '5')
        //     } else {
        //         format!("{}{}", non_num, num)
        //     }
        // })
        // .collect();
        let mut word_pinyin = vec![];
        for (i, p) in word.to_pinyin().enumerate() {
            if p.is_some() {
                let p = p.unwrap();
                let pn = p.with_tone_num();
                let py2 = get_finals(pn, true);
                let (non_num, num): (String, String) =
                    py2.chars().partition(|c| !c.is_ascii_digit());
                let py2 = format!(
                    "{}{}",
                    non_num,
                    if num.trim().is_empty() {
                        if _with_five {
                            "5"
                        } else {
                            ""
                        }
                    } else {
                        &num
                    }
                );
                // let mut py2 = p.finals_with_tone_num_end().to_string();
                // if _with_five {
                //     let py2_len = py2.chars().count();
                //     let py2_tone = {
                //         let e = py2.chars().nth(py2_len - 1);
                //         if let Some(e) = e {
                //             match !e.is_numeric() {
                //                 true => "5",
                //                 false => "",
                //             }
                //         } else {
                //             ""
                //         }
                //     };
                //     py2 = format!("{}{}", py2, py2_tone);
                // }
                word_pinyin.push(py2);
            } else {
                let wc = word.chars().nth(i).unwrap();
                word_pinyin.push(wc.to_string());
            }
        }
        word_pinyin
    }

    fn _is_reduplication(word: &String) -> bool {
        if word.chars().count() < 2 {
            return false;
        }
        let a0 = word.substring(0, 1);
        let a1 = word.substring(1, 2);
        return word.chars().count() == 2 && a0 == a1;
    }

    fn _merge_continuous_three_tones(seg_cut: &[(String, String)]) -> Vec<(String, String)> {
        let mut new_seg: Vec<(String, String)> = vec![];
        let mut sub_finals_list: Vec<Vec<String>> = vec![];
        (0..seg_cut.len()).for_each(|i| {
            let (word, _) = &seg_cut[i];
            let word_pinyin = ToneSandhi::get_pinyin(word, true);
            if !word_pinyin.is_empty() {
                sub_finals_list.push(word_pinyin);
            } else {
                sub_finals_list.push(vec![word.clone()]);
            }
        });

        let mut merge_last = vec![false; seg_cut.len()];
        for i in 0..seg_cut.len() {
            let (word, pos) = &seg_cut[i];
            if i >= 1 {
                let b1 = ToneSandhi::_all_tone_three(&sub_finals_list[i - 1]);
                let b2 = ToneSandhi::_all_tone_three(&sub_finals_list[i]);
                let b3 = merge_last[i - 1];
                if b1 && b2 && b3 {
                    if !ToneSandhi::_is_reduplication(&seg_cut[i - 1].0)
                        && seg_cut[i - 1].0.chars().count() + seg_cut[i].0.chars().count() <= 3
                    {
                        let l = new_seg.len();
                        new_seg[l - 1].0 = new_seg[l - 1].0.clone() + &seg_cut[i].0;
                        merge_last[i] = true;
                    } else {
                        new_seg.push((word.clone(), pos.clone()));
                    }
                } else {
                    new_seg.push((word.clone(), pos.clone()));
                }
            } else {
                new_seg.push((word.clone(), pos.clone()));
            }
        }
        new_seg
    }

    fn _merge_continuous_three_tones_2(seg_cut: &[(String, String)]) -> Vec<(String, String)> {
        let mut new_seg: Vec<(String, String)> = vec![];
        let mut sub_finals_list: Vec<Vec<String>> = vec![];
        (0..seg_cut.len()).for_each(|i| {
            let (word, _) = &seg_cut[i];
            let word_pinyin = ToneSandhi::get_pinyin(word, true);
            if !word_pinyin.is_empty() {
                sub_finals_list.push(word_pinyin);
            } else {
                sub_finals_list.push(vec![word.clone()]);
            }
        });

        let mut merge_last = vec![false; seg_cut.len()];
        for i in 0..seg_cut.len() {
            let (word, pos) = &seg_cut[i];
            if i >= 1 {
                let l1 = sub_finals_list[i - 1].len();
                let l11 = sub_finals_list[i - 1][l1 - 1].chars().count();
                let b1 = sub_finals_list[i - 1][l1 - 1].substring(l11 - 1, l11) == "3";

                let l2 = sub_finals_list[i][0].chars().count();
                let b2 = sub_finals_list[i][0].substring(l2 - 1, l2) == "3";
                let b3 = merge_last[i - 1];
                if b1 && b2 && b3 {
                    if !ToneSandhi::_is_reduplication(&seg_cut[i - 1].0)
                        && seg_cut[i - 1].0.chars().count() + seg_cut[i].0.chars().count() <= 3
                    {
                        let l = new_seg.len();
                        new_seg[l - 1].0 = new_seg[l - 1].0.clone() + &seg_cut[i].0;
                        merge_last[i] = true;
                    } else {
                        new_seg.push((word.clone(), pos.clone()));
                    }
                } else {
                    new_seg.push((word.clone(), pos.clone()));
                }
            } else {
                new_seg.push((word.clone(), pos.clone()));
            }
        }
        new_seg
    }

    fn _merge_er(seg_cut: &[(String, String)]) -> Vec<(String, String)> {
        let mut new_seg: Vec<(String, String)> = vec![];
        for i in 0..seg_cut.len() {
            let (word, pos) = &seg_cut[i];
            if i >= 1 {
                if word == "儿" && seg_cut[i - 1].0 != "#" {
                    let l1 = new_seg.len();
                    new_seg[l1 - 1].0 = new_seg[l1 - 1].0.clone() + &seg_cut[i].0;
                } else {
                    new_seg.push((word.clone(), pos.clone()));
                }
            } else {
                new_seg.push((word.clone(), pos.clone()));
            }
        }
        new_seg
    }

    fn _merge_reduplication(seg_cut: &[(String, String)]) -> Vec<(String, String)> {
        let mut new_seg: Vec<(String, String)> = vec![];
        (0..seg_cut.len()).for_each(|i| {
            let (word, pos) = &seg_cut[i];
            let l1 = new_seg.len();
            if l1 > 0 && &new_seg[l1 - 1].0 == word {
                new_seg[l1 - 1].0 = new_seg[l1 - 1].0.clone() + &seg_cut[i].0;
            } else {
                new_seg.push((word.clone(), pos.clone()));
            }
        });
        new_seg
    }

    pub fn pre_merge_for_modify(&self, seg_cut: &Vec<Tag>) -> Vec<(String, String)> {
        let seg_cut = ToneSandhi::_merge_bu(seg_cut);
        let seg_cut = ToneSandhi::_merge_yi(&seg_cut);
        let seg_cut = ToneSandhi::_merge_reduplication(&seg_cut);
        let seg_cut = ToneSandhi::_merge_continuous_three_tones(&seg_cut);
        let seg_cut = ToneSandhi::_merge_continuous_three_tones_2(&seg_cut);

        ToneSandhi::_merge_er(&seg_cut)
    }

    pub fn modified_tone(
        &self,
        word: &String,
        pos: &str,
        finals: Vec<String>,
        jieba_util: &Jieba,
    ) -> Vec<String> {
        let finals = ToneSandhi::_bu_sandhi(word, finals);
        let finals = self._yi_sandhi(word, finals);
        let finals = self._neural_sandhi(word, pos, finals, jieba_util);

        self._three_sandhi(word, finals, jieba_util)
    }
}

#[test]
fn test0() {
    let _t = ToneSandhi::init();
    let _word = &"男子".to_string();
}
