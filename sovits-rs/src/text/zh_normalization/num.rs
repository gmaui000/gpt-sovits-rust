use fancy_regex::{Captures, Regex};
use lazy_static::lazy_static;
use std::collections::HashMap;

const MAX_NUMERIC_LENGTH: usize = 13; // 超过此长度的数字直接逐位翻译

lazy_static! {
    static ref RE_SCIENTIFIC: Regex = Regex::new(r"(-?)(\d+(\.\d+)?)[eE]([+-]?\d+)").unwrap();
    static ref RE_FRAC: Regex = Regex::new(r"(-?)(\d+)/(\d+)").unwrap();
    static ref RE_PERCENTAGE: Regex = Regex::new(r"(-?)(\d+(\.\d+)?)%").unwrap();
    static ref RE_RANGE: Regex =
        Regex::new(r"((-?)((\d+)(\.\d+)?)|(\.(\d+)))[-~]((-?)((\d+)(\.\d+)?)|(\.(\d+)))").unwrap();
    static ref RE_NUMBER: Regex = Regex::new(r"(-?)((\d+)(\.\d+)?)|(\.(\d+))").unwrap();
    static ref RE_INTEGER: Regex = Regex::new(r"(-)(\d+)").unwrap();
    static ref RE_DEFAULT_NUM: Regex = Regex::new(r"\d{7}\d*").unwrap();
    static ref RE_POSITIVE_QUANTIFIERS: Regex = Regex::new(
        r"(\d+)([多余几\+])?(封|艘|把|目|套|段|人|所|朵|匹|张|座|回|场|尾|条|个|首|阙|阵|网|炮|顶|丘|棵|只|支|袭|辆|挑|担|颗|\
        壳|窠|曲|墙|群|腔|砣|座|客|贯|扎|捆|刀|令|打|手|罗|坡|山|岭|江|溪|钟|队|单|双|对|出|口|头|脚|\
        板|跳|枝|件|贴|针|线|管|名|位|身|堂|课|本|页|家|户|层|丝|毫|厘|分|钱|两|斤|担|铢|石|钧|锱|忽|\
        (千|毫|微)?克|毫|厘|(公)?分|分|寸|尺|丈|里|寻|常|铺|程|(千|分|厘|毫|微)?米|米|撮|勺|合|升|斗|石|\
        盘|碗|碟|叠|桶|笼|盆|盒|杯|钟|斛|锅|簋|篮|盘|桶|罐|瓶|壶|卮|盏|箩|箱|煲|啖|袋|钵|年|月|日|季|刻|\
        时|周|天|秒|分|小时|旬|纪|岁|世|更|夜|春|夏|秋|冬|代|伏|辈|丸|泡|粒|颗|幢|堆|条|根|支|道|面|片|\
        张|颗|块|元|((亿|千万|百万|万|千|百|十)元?)|(亿|千万|百万|万|千|百|十)?吨)"
    ).unwrap();

    static ref DIGITS: HashMap<char, char> = "零一二三四五六七八九"
        .chars()
        .enumerate()
        .map(|(i, c)| (std::char::from_digit(i as u32, 10).unwrap(), c))
        .collect();
    static ref UINT: HashMap<usize, char> =
        [(1, '十'), (2, '百'), (3, '千'), (4, '万'), (8, '亿')]
            .into_iter()
            .collect();
}

#[derive(Default)]
pub struct Num;

impl Num {
    pub fn normalize(&self, num_str: &str) -> String {
        let funcs: Vec<fn(&Self, &str) -> String> = vec![
            Self::replace_scientific,
            Self::replace_frac,
            Self::replace_percentage,
            Self::replace_range,
            Self::replace_number,
            Self::replace_default_num,
            Self::replace_positive_quantifier,
            Self::replace_negative_num,
        ];

        funcs
            .iter()
            .fold(num_str.to_string(), |result, func| func(self, &result))
    }

    pub fn num2str(&self, value: &str, with_limit: bool) -> String {
        if let Some((int, dec)) = value.split_once('.') {
            let mut result = self.verbalize_cardinal(int, false);
            let decimal = dec.trim_end_matches("0");
            if !decimal.is_empty() {
                result = if result.is_empty() {
                    "零".to_string()
                } else {
                    result
                };
                result.push('点');
                result.push_str(&self.verbalize_digits(decimal, false));
            }
            result
        } else {
            self.verbalize_cardinal(value, with_limit)
        }
    }

    pub fn verbalize_cardinal(&self, sentence: &str, with_limit: bool) -> String {
        if sentence.is_empty() {
            sentence.to_string()
        } else if with_limit && (sentence.starts_with('0') || sentence.len() > MAX_NUMERIC_LENGTH) {
            self.verbalize_digits(sentence, true)
        } else {
            let value_string = sentence.trim_start_matches('0');
            if value_string.is_empty() {
                DIGITS[&'0'].to_string()
            } else {
                let mut result_symbols = self.get_value(value_string, true);
                let d1 = DIGITS[&'1'].to_string();
                let u1 = UINT[&1].to_string();
                if result_symbols.len() >= 2 && result_symbols[0] == d1 && result_symbols[1] == u1 {
                    result_symbols = result_symbols[1..].to_vec();
                }

                result_symbols.join("")
            }
        }
    }

    pub fn verbalize_digits(&self, value: &str, alt_one: bool) -> String {
        value
            .chars()
            .map(|c| {
                // Convert `char` to `String` for proper replacement
                DIGITS
                    .get(&c)
                    .map(|&d| d.to_string())
                    .unwrap_or(c.to_string())
            })
            .collect::<String>()
            .replace('一', if alt_one { "幺" } else { "一" })
    }

    fn replace_with_regex<F>(&self, value: &str, regex: &Regex, replacer: F) -> String
    where
        F: Fn(&Captures) -> String,
    {
        regex.replace_all(value, replacer).to_string()
    }

    fn is_all_zero(&self, value: &str) -> bool {
        value.chars().all(|c| c == '0')
    }

    fn replace_number(&self, value: &str) -> String {
        self.replace_with_regex(value, &RE_NUMBER, |caps| {
            let sign = caps.get(1).map_or("", |m| m.as_str());
            let number = caps.get(2).map(|m| m.as_str());
            let pure_decimal = caps.get(5).map(|m| m.as_str());

            if let Some(pure_decimal) = pure_decimal {
                self.num2str(pure_decimal, false)
            } else {
                let number = number.unwrap_or("");
                let sign = {
                    if !sign.is_empty() && !self.is_all_zero(number) {
                        "负"
                    } else {
                        ""
                    }
                };

                let number = self.num2str(number, sign.is_empty());
                format!("{}{}", sign, number)
            }
        })
    }

    fn replace_frac(&self, value: &str) -> String {
        self.replace_with_regex(value, &RE_FRAC, |caps| {
            let sign = caps.get(1).map_or("", |m| m.as_str());
            let nominator = caps.get(2).map_or("", |m| m.as_str());
            let denominator = caps.get(3).map_or("", |m| m.as_str());

            let sign = {
                if !sign.is_empty() && !self.is_all_zero(nominator) {
                    "负"
                } else {
                    ""
                }
            };
            let nominator = self.num2str(nominator, false);
            let denominator = self.num2str(denominator, false);

            format!("{}{}分之{}", sign, denominator, nominator)
        })
    }

    fn replace_percentage(&self, value: &str) -> String {
        self.replace_with_regex(value, &RE_PERCENTAGE, |caps| {
            let sign = caps.get(1).map_or("", |m| m.as_str());
            let percent = caps.get(2).map_or("", |m| m.as_str());

            let percent = self.num2str(percent, false);
            let sign = {
                if !sign.is_empty() {
                    "负"
                } else {
                    ""
                }
            };
            format!("{}百分之{}", sign, percent)
        })
    }

    fn replace_negative_num(&self, value_string: &str) -> String {
        self.replace_with_regex(value_string, &RE_INTEGER, |caps| {
            let sign = caps.get(1).map_or("", |m| m.as_str());
            let number = caps.get(2).map_or("", |m| m.as_str());

            let number = self.num2str(number, false);

            let sign = {
                if !sign.is_empty() {
                    "负"
                } else {
                    ""
                }
            };
            format!("{}{}", sign, number)
        })
    }

    fn replace_positive_quantifier(&self, value: &str) -> String {
        self.replace_with_regex(value, &RE_POSITIVE_QUANTIFIERS, |caps| {
            let number: Option<&str> = caps.get(1).map(|m| m.as_str());
            let match_2: Option<&str> = caps.get(2).map(|m| m.as_str());
            let quantifiers: Option<&str> = caps.get(3).map(|m| m.as_str());

            let match_2 = {
                if match_2.is_some() && match_2.unwrap() == "+" {
                    "多"
                } else {
                    ""
                }
            };
            let number = { number.unwrap_or("") };
            let quantifiers = { quantifiers.unwrap_or("") };
            let number = self.num2str(number, false);

            format!("{}{}{}", number, match_2, quantifiers)
        })
    }

    fn replace_default_num(&self, value: &str) -> String {
        self.replace_with_regex(value, &RE_DEFAULT_NUM, |caps| {
            self.verbalize_digits(caps.get(0).unwrap().as_str(), true)
        })
    }

    fn replace_range(&self, value_string: &str) -> String {
        self.replace_with_regex(value_string, &RE_RANGE, |caps| {
            let first = caps.get(1).map_or("", |m| m.as_str());
            let second = caps.get(8).map_or("", |m| m.as_str());

            let first = self.replace_number(first);
            let second = self.replace_number(second);

            format!("{}到{}", first, second)
        })
    }

    fn replace_scientific(&self, value: &str) -> String {
        self.replace_with_regex(value, &RE_SCIENTIFIC, |caps| {
            let sign = caps.get(1).map_or("", |m| m.as_str());
            let base = caps.get(2).map_or("", |m| m.as_str());
            let exponent = caps.get(4).map_or("", |m| m.as_str());

            let number = if let Ok(exp) = exponent.parse::<i32>() {
                if exp >= 0 {
                    let shift = exp as usize;
                    let parts: Vec<&str> = base.split('.').collect();
                    let integer_part = parts[0];
                    let decimal_part = if parts.len() > 1 { parts[1] } else { "" };
                    let combined = format!("{}{}", integer_part, decimal_part);
                    let padding = "0".repeat(shift.saturating_sub(decimal_part.len()));
                    format!("{}{}", combined, padding)
                } else {
                    let shift = (-exp) as usize;
                    let mut result = "0.".to_string();
                    result.push_str(&"0".repeat(shift - 1));
                    result.push_str(&base.replace(".", ""));
                    result
                }
            } else {
                base.to_string() // 默认返回原始形式
            };

            let normalized = self.num2str(&number, false);
            if !sign.is_empty() {
                format!("负{}", normalized)
            } else {
                normalized
            }
        })
    }

    #[allow(clippy::only_used_in_recursion)]
    fn get_value(&self, value_string: &str, use_zero: bool) -> Vec<String> {
        let stripped = value_string.trim_start_matches('0');
        let stripped_len = stripped.len();
        let value_len = value_string.len();

        match stripped_len {
            0 => Vec::new(),
            1 => {
                let digit = DIGITS[&stripped.chars().next().unwrap()].to_string();
                match (use_zero, stripped_len < value_len) {
                    (true, true) => vec![DIGITS[&'0'].to_string(), digit],
                    _ => vec![digit],
                }
            }
            _ => {
                let keys = [8, 4, 3, 2, 1];
                let largest_unit = keys
                    .iter()
                    .find(|&&power| power < stripped.chars().count())
                    .copied()
                    .unwrap_or(8);

                let split_point = value_string.chars().count() - largest_unit;
                let first_part = &value_string[..split_point];
                let second_part = &value_string[split_point..];

                let mut result = self.get_value(first_part, true);
                if let Some(unit) = UINT.get(&largest_unit) {
                    result.push(unit.to_string());
                }
                result.extend(self.get_value(second_part, true));
                result
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num() {
        let num = Num;

        // 测试 replace_number
        assert_eq!(num.replace_number("2004"), "二千零四");
        assert_eq!(num.replace_number("2014"), "二千零一十四");
        assert_eq!(num.replace_number("-2014"), "负二千零一十四");
        assert_eq!(num.replace_number("0"), "零");
        assert_eq!(num.replace_number("000123"), "零零零幺二三");
        assert_eq!(
            num.replace_number("1234567890"),
            "十二亿三千四百五十六万七千八百九十"
        );
        assert_eq!(num.replace_number("-000"), "零零零");

        // 测试 replace_frac
        assert_eq!(num.replace_frac("-1/3"), "负三分之一");
        assert_eq!(num.replace_frac("3/2"), "二分之三");
        assert_eq!(num.replace_frac("0/1"), "一分之零");
        assert_eq!(num.replace_frac("-0/1"), "一分之零");
        assert_eq!(
            num.replace_frac("123456789/987654321"),
            "九亿八千七百六十五万四千三百二十一分之一亿二千三百四十五万六千七百八十九"
        );

        // 测试 replace_percentage
        assert_eq!(num.replace_percentage("50%"), "百分之五十");
        assert_eq!(num.replace_percentage("0%"), "百分之零");
        assert_eq!(num.replace_percentage("-100%"), "负百分之一百");
        assert_eq!(
            num.replace_percentage("123456%"),
            "百分之十二万三千四百五十六"
        );
        assert_eq!(num.replace_percentage("-0%"), "负百分之零");

        // 测试 replace_range
        assert_eq!(num.replace_range("1.2~3.4"), "一点二到三点四");
        assert_eq!(num.replace_range("0~100"), "零到一百");
        assert_eq!(num.replace_range("-50~50"), "负五十到五十");
        assert_eq!(num.replace_range("123~456"), "一百二十三到四百五十六");
        assert_eq!(num.replace_range("-1.23~4.56"), "负一点二三到四点五六");

        // 测试 verbalize_cardinal
        assert_eq!(num.verbalize_cardinal("0123", true), "零幺二三");
        assert_eq!(num.verbalize_cardinal("0000", true), "零零零零");
        assert_eq!(num.verbalize_cardinal("001001", true), "零零幺零零幺");
        assert_eq!(
            num.verbalize_cardinal("123456789", true),
            "一亿二千三百四十五万六千七百八十九"
        );
        assert_eq!(num.verbalize_cardinal("10001", true), "一万零一");
    }

    #[test]
    fn test_seq() {
        let num = Num;
        assert_eq!(num.normalize("0123"), "零幺二三");
        assert_eq!(
            num.normalize("12345678901234"),
            "幺二三四五六七八九零幺二三四"
        );

        assert_eq!(
            num.normalize("1234567890123"),
            "一万二千三百四十五亿六千七百八十九万零一百二十三"
        );

        assert_eq!(
            num.normalize("-12345678901234"),
            "负十二万三千四百五十六亿七千八百九十万一千二百三十四"
        );
    }

    #[test]
    fn test_num_normalize() {
        let num = Num;

        let cases = vec![
            // 整数
            ("123", "一百二十三"),
            ("0123", "零幺二三"),
            ("0", "零"),
            ("-123", "负一百二十三"),
            ("1234567890", "十二亿三千四百五十六万七千八百九十"),
            ("0000", "零零零零"),
            ("001001", "零零幺零零幺"),
            (
                "1234567890123",
                "一万二千三百四十五亿六千七百八十九万零一百二十三",
            ),
            ("12345678901234", "幺二三四五六七八九零幺二三四"),
            // 小数
            ("123.45", "一百二十三点四五"),
            ("0.001", "零点零零一"),
            ("-0.123", "负零点一二三"),
            ("123.00", "一百二十三"),
            // 分数
            ("1/2", "二分之一"),
            ("-3/4", "负四分之三"),
            ("0/1", "一分之零"),
            ("123/456", "四百五十六分之一百二十三"),
            ("-123/456", "负四百五十六分之一百二十三"),
            // 百分数
            ("50%", "百分之五十"),
            ("0%", "百分之零"),
            ("-100%", "负百分之一百"),
            ("123.45%", "百分之一百二十三点四五"),
            ("-0.1%", "负百分之零点一"),
            // 区间
            ("1~10", "一到十"),
            ("0~100", "零到一百"),
            ("-50~50", "负五十到五十"),
            ("123.45~678.90", "一百二十三点四五到六百七十八点九"),
            ("-1.2~3.4", "负一点二到三点四"),
            // 科学计数法
            ("1e3", "一千"),
            ("1.23e4", "一万二千三百"),
            ("-1.23e-2", "负零点零一二三"),
            ("0e0", "零"),
            // 特殊情况
            ("123~", "一百二十三~"),
            ("~456", "~四百五十六"),
            ("~", "~"),
            ("-", "-"),
            // ("1..2", "一..二"),
            // ("1.23.45", "一点二三点四五"),
        ];

        for (input, expected) in cases {
            let result = num.normalize(input);
            assert_eq!(
                result, expected,
                "Test failed for input: {}.\nExpected: {}, Got: {}",
                input, expected, result
            );
        }
    }
}
