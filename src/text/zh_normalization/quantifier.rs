use super::num::Num;
use fancy_regex::{Captures, Regex};
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    // 匹配温度格式：-10°C、20℃、35度、40.5摄氏度
    static ref RE_TEMPERATURE: Regex = Regex::new(r"(-?)(\d+(\.\d+)?)(°C|℃|度|摄氏度)").unwrap();

    // 单位转换表，按长度优先匹配
    static ref MEASURE_DICT: HashMap<&'static str, &'static str> = HashMap::from([
        // 面积和体积相关
        ("cm2", "平方厘米"), ("cm²", "平方厘米"), ("cm3", "立方厘米"), ("cm³", "立方厘米"),
        ("cm", "厘米"), ("m2", "平方米"), ("m²", "平方米"), ("m³", "立方米"),
        ("m3", "立方米"), ("ml", "毫升"), ("m", "米"), ("mm", "毫米"),

        // 质量相关
        ("kg", "千克"), ("g", "克"),

        // 时间相关
        ("s", "秒"), ("ds", "毫秒"),

        // 声学相关
        ("db", "分贝"),

        // 长度相关
        ("km", "千米"),

        // 速度相关
        ("m/s", "米每秒"), ("km/s", "千米每秒"), ("km/h", "千米每小时"), ("mm/s", "毫米每秒")
    ]);

    // 按长度降序排列的单位键，用于匹配替换
    static ref MEASURE_KEYS: Vec<&'static str> = {
        let mut keys: Vec<&str> = MEASURE_DICT.keys().cloned().collect();
        keys.sort_by_key(|b| std::cmp::Reverse(b.len()));
        keys
    };
}

pub struct Quantifier {
    num: Num,
}

impl Quantifier {
    pub(crate) fn new() -> Self {
        Quantifier { num: Num::new() }
    }

    pub fn normalize(&self, sentence: &str) -> String {
        let funcs: Vec<fn(&Self, &str) -> String> =
            vec![Self::replace_measure, Self::replace_temperature];

        funcs
            .iter()
            .fold(sentence.to_string(), |result, func| func(self, &result))
    }

    fn replace_temperature(&self, temperature_str: &str) -> String {
        RE_TEMPERATURE
            .replace_all(temperature_str, |caps: &Captures| {
                let sign = caps.get(1).map_or("", |m| m.as_str());
                let sign = if !sign.is_empty() { "零下" } else { "" };

                let temperature = caps.get(2).map_or("", |m| m.as_str());
                let temperature = self.num.num2str(temperature, false);

                let unit = match caps.get(4).map_or("", |m| m.as_str()) {
                    "摄氏度" => "摄氏度",
                    _ => "度",
                };
                format!("{}{}{}", sign, temperature, unit)
            })
            .to_string()
    }

    pub fn replace_measure(&self, measure_str: &str) -> String {
        MEASURE_KEYS
            .iter()
            .fold(measure_str.to_string(), |result, &key| {
                let regex = Regex::new(&format!(r"(\d+(\.\d+)?)(\s*){}", key)).unwrap();
                regex
                    .replace_all(&result, |caps: &Captures| {
                        let number_part = caps.get(1).map_or("", |m| m.as_str());
                        let number_str = self.num.num2str(number_part, false);
                        let unit_str = MEASURE_DICT.get(key).unwrap_or(&key);
                        format!("{}{}", number_str, unit_str)
                    })
                    .to_string()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_temperature() {
        let quantifier = Quantifier::new();

        let cases = vec![
            ("-10°C", "零下十度"),
            ("20℃", "二十度"),
            ("35度", "三十五度"),
            ("40.5摄氏度", "四十点五摄氏度"),
            ("-5摄氏度", "零下五摄氏度"),
            ("50K", "50K"), // 无效温度
        ];

        for (input, expected) in cases {
            assert_eq!(quantifier.replace_temperature(input), expected);
        }
    }

    #[test]
    fn test_replace_measure() {
        let quantifier = Quantifier::new();

        let cases = vec![
            ("面积是10cm2", "面积是十平方厘米"),
            ("体积是5cm³", "体积是五立方厘米"),
            ("长度为20m", "长度为二十米"),
            ("质量为70kg", "质量为七十千克"),
            ("速度单位为20km/h", "速度单位为二十千米每小时"),
            ("时间是10s", "时间是十秒"),
            ("没有任何单位", "没有任何单位"),
            (
                "2983.07g或12345.60m",
                "二千九百八十三点零七克或一万二千三百四十五点六米",
            ),
        ];

        for (input, expected) in cases {
            assert_eq!(quantifier.replace_measure(input), expected);
        }
    }

    #[test]
    fn test_combined_replace() {
        let quantifier = Quantifier::new();

        let input = "温度是-10°C，面积是5m2，时间是10s";
        let expected = "温度是零下十度，面积是五平方米，时间是十秒";
        let result = quantifier.replace_measure(&quantifier.replace_temperature(input));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_replace_measure_with_speed() {
        let quantifier = Quantifier::new();

        let cases = vec![
            ("速度是10m/s", "速度是十米每秒"),
            ("光速大约为300000km/s", "光速大约为三十万千米每秒"),
            ("普通车速是60km/h", "普通车速是六十千米每小时"),
            ("蜗牛移动速度为5mm/s", "蜗牛移动速度为五毫米每秒"),
        ];

        for (input, expected) in cases {
            assert_eq!(quantifier.replace_measure(input), expected);
        }
    }

    #[test]
    fn test_combined_replace_with_speed() {
        let quantifier = Quantifier::new();

        let input = "速度是10m/s，温度是-20°C，长度是50cm";
        let expected = "速度是十米每秒，温度是零下二十度，长度是五十厘米";
        let result = quantifier.replace_measure(&quantifier.replace_temperature(input));
        assert_eq!(result, expected);
    }
}
