use super::num::Num;
use fancy_regex::{Captures, Regex};
use lazy_static::lazy_static;

lazy_static! {
    // 普通日期 2023年10月25日
    static ref RE_DATE: Regex = Regex::new(r"(\d{4}|\d{2})年((1[0-2]|0?[1-9])月)?(([12]\d|30|31|0?[1-9])([日号]))?").unwrap();

    // 日期后带“日”或“号” 2023-10-25
    static ref RE_DATE2: Regex = Regex::new(r"(\d{4}|\d{2})[- /.](1[0-2]|0?[1-9])[- /.]([12]\d|30|31|0?[1-9])([日号])?").unwrap();

    // 时间范围，如8:30-12:30
    static ref RE_TIME_RANGE: Regex = Regex::new(r"([01]?\d|2[0-3]):([0-5]\d)(:([0-5]\d))?(~|-)([01]?\d|2[0-3]):([0-5]\d)(:([0-5]\d))?").unwrap();

    // 时刻表达式
    static ref RE_TIME: Regex = Regex::new(r"([01]?\d|2[0-3]):([0-5]\d)(:([0-5]\d))?").unwrap();

}

pub struct Chronology {
    num: Num,
}

impl Chronology {
    pub(crate) fn new() -> Self {
        Self { num: Num }
    }

    pub fn normalize(&self, chronology_str: &str) -> String {
        let funcs: Vec<fn(&Self, &str) -> String> = vec![
            Self::replace_date,
            Self::replace_date2,
            Self::replace_time_range,
            Self::replace_time,
        ];

        funcs
            .iter()
            .fold(chronology_str.to_string(), |result, func| {
                func(self, &result)
            })
    }

    fn time_num2str(&self, num_str: &str) -> String {
        let t = num_str.trim_start_matches("0");
        let mut result = self.num.num2str(t, false);
        if num_str.starts_with("0") && result != "零" {
            result.insert(0, '零');
        }
        result
    }

    /// 日期转化
    fn replace_date(&self, date_str: &str) -> String {
        RE_DATE
            .replace_all(date_str, |caps: &Captures| {
                let year = caps
                    .get(1)
                    .map(|m| format!("{}年", self.num.verbalize_digits(m.as_str(), false)));
                let month = caps
                    .get(3)
                    .map(|m| format!("{}月", self.num.verbalize_cardinal(m.as_str(), false)));
                let day = caps.get(5).map(|m| {
                    let suffix = caps.get(9).map_or("日", |m| m.as_str());
                    format!(
                        "{}{}",
                        self.num.verbalize_cardinal(m.as_str(), false),
                        suffix
                    )
                });
                format!(
                    "{}{}{}",
                    year.unwrap_or_default(),
                    month.unwrap_or_default(),
                    day.unwrap_or_default()
                )
            })
            .to_string()
    }

    fn replace_date2(&self, date_str: &str) -> String {
        RE_DATE2
            .replace_all(date_str, |caps: &Captures| {
                let year = caps
                    .get(1)
                    .map(|m| format!("{}年", self.num.verbalize_digits(m.as_str(), false)));
                let month = caps
                    .get(2)
                    .map(|m| format!("{}月", self.num.verbalize_cardinal(m.as_str(), false)));
                let day = caps.get(3).map(|m| {
                    let suffix = caps.get(4).map_or("日", |m| m.as_str());
                    format!(
                        "{}{}",
                        self.num.verbalize_cardinal(m.as_str(), false),
                        suffix
                    )
                });
                format!(
                    "{}{}{}",
                    year.unwrap_or_default(),
                    month.unwrap_or_default(),
                    day.unwrap_or_default()
                )
            })
            .to_string()
    }

    fn replace_time(&self, time_str: &str) -> String {
        self._replace_time(time_str, &RE_TIME)
    }

    fn replace_time_range(&self, time_str: &str) -> String {
        self._replace_time(time_str, &RE_TIME_RANGE)
    }

    fn _replace_time(&self, time_str: &str, regex: &Regex) -> String {
        regex
            .replace_all(time_str, |caps: &Captures| {
                let hour = caps.get(1).map(|m| self.num.num2str(m.as_str(), false));
                let minute = caps.get(2).map(|m| self.time_num2str(m.as_str()));
                let second = caps
                    .get(4)
                    .map(|m| format!("{}秒", self.time_num2str(m.as_str())));

                let mut result = hour.map(|h| format!("{}点", h)).unwrap_or_default();

                if let Some(minute) = minute {
                    if minute == "三十" {
                        result.push('半');
                    } else if minute != "零" {
                        result.push_str(&format!("{}分", minute));
                    }
                }
                if let Some(second) = second {
                    result.push_str(&second);
                }

                if caps.len() > 5 {
                    result.push('至');
                    let hour2 = caps.get(6).map(|m| self.num.num2str(m.as_str(), false));
                    let minute2 = caps.get(7).map(|m| self.time_num2str(m.as_str()));
                    let second2 = caps
                        .get(9)
                        .map(|m| format!("{}秒", self.time_num2str(m.as_str())));

                    if let Some(h2) = hour2 {
                        result.push_str(&format!("{}点", h2));
                    }
                    if let Some(minute2) = minute2 {
                        if minute2 == "三十" {
                            result.push('半');
                        } else if minute2 != "零" {
                            result.push_str(&format!("{}分", minute2));
                        }
                    }
                    if let Some(second2) = second2 {
                        result.push_str(&second2);
                    }
                }

                result
            })
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_replace_date() {
        let chronology = Chronology::new();

        // 普通日期
        assert_eq!(
            chronology.replace_date("2023年10月25日"),
            "二零二三年十月二十五日"
        );

        // 缺少月、日
        assert_eq!(chronology.replace_date("2023年"), "二零二三年");
        assert_eq!(chronology.replace_date("2023年10月"), "二零二三年十月");

        // 非法日期
        assert_eq!(chronology.replace_date("abc年"), "abc年");
    }

    #[test]
    fn test_replace_date2() {
        let chronology = Chronology::new();

        // 普通日期
        assert_eq!(
            chronology.replace_date2("2023-10-25"),
            "二零二三年十月二十五日"
        );

        assert_eq!(
            chronology.replace_date2("2023/10/25"),
            "二零二三年十月二十五日"
        );

        // 日期后带“日”或“号”
        assert_eq!(
            chronology.replace_date2("2023-10-25号"),
            "二零二三年十月二十五号"
        );

        // 非法日期
        assert_eq!(chronology.replace_date2("2023-13-40"), "2023-13-40");
    }

    #[test]
    fn test_replace_time() {
        let chronology = Chronology::new();

        // 普通时间
        assert_eq!(chronology.replace_time("8:30"), "八点半");
        assert_eq!(chronology.replace_time("8:05"), "八点零五分");

        // 带秒的时间
        assert_eq!(chronology.replace_time("8:05:30"), "八点零五分三十秒");

        // 时间范围
        assert_eq!(
            chronology.replace_time_range("8:05-9:30"),
            "八点零五分至九点半"
        );

        // 时间范围
        assert_eq!(
            chronology.replace_time_range("8:30-12:00"),
            "八点半至十二点"
        );

        // 非法时间
        assert_eq!(chronology.replace_time("25:60"), "25:60");
    }
    #[test]
    fn test_time_num2str() {
        let chronology = Chronology::new();

        // 普通数字
        assert_eq!(chronology.time_num2str("5"), "五");
        assert_eq!(chronology.time_num2str("15"), "十五");

        // 以零开头的数字
        assert_eq!(chronology.time_num2str("05"), "零五");
        assert_eq!(chronology.time_num2str("0015"), "零十五");

        // 边界情况
        assert_eq!(chronology.time_num2str("0"), "零");
        assert_eq!(chronology.time_num2str(""), "");
    }
    #[test]
    fn test_comprehensive() {
        let chronology = Chronology::new();

        let input = "2023年10月25日，会议时间为8:30-12:00";
        let expected = "二零二三年十月二十五日，会议时间为八点半至十二点";
        let replaced_date = chronology.replace_date(input);
        let replaced_time = chronology.replace_time_range(&replaced_date);
        assert_eq!(replaced_time, expected);
        assert_eq!(chronology.normalize(input), expected);

        let input = "2023-10-25 8:30";
        let expected = "二零二三年十月二十五日 八点半";
        let replaced_date = chronology.replace_date2(input);
        let replaced_time = chronology.replace_time(&replaced_date);
        assert_eq!(replaced_time, expected);
        assert_eq!(chronology.normalize(input), expected);

        let input = "日期：2025/01/14~2025/12/31。";
        let expected = "日期：二零二五年一月十四日~二零二五年十二月三十一日。";
        assert_eq!(chronology.normalize(input), expected);
    }
}
