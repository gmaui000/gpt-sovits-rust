use super::num::Num;
use fancy_regex::{Captures, Regex};
use lazy_static::lazy_static;

lazy_static! {
    static ref RE_MOBILE_PHONE: Regex =
        Regex::new(r"(?<!\d)((\+?86 ?)?1([38]\d|5[0-35-9]|7[678]|9[89]) ?\d{4} ?\d{4})(?!\d)")
            .unwrap();
    static ref RE_TELEPHONE: Regex =
        Regex::new(r"(?<!\d)((0(10|2[1-3]|[3-9]\d{2})-?)?[1-9]\d{6,7})(?!\d)").unwrap();
    static ref RE_NATIONAL_UNIFORM_NUMBER: Regex = Regex::new(r"400(-)?\d{3}(-)?\d{4}").unwrap();
}

pub struct Phonecode {
    num: Num,
}

impl Phonecode {
    pub(crate) fn new() -> Self {
        Self { num: Num }
    }

    pub fn normalize(&self, phone_str: &str) -> String {
        let funcs: Vec<fn(&Self, &str) -> String> = vec![
            Self::replace_mobile,
            Self::replace_phone,
            Self::replace_phone2,
        ];

        funcs
            .iter()
            .fold(phone_str.to_string(), |result, func| func(self, &result))
    }

    pub fn phone2str(&self, phone_str: &str, mobile: bool) -> String {
        let parts: Vec<&str> = if mobile && phone_str.starts_with('+') {
            phone_str
                .trim_start_matches('+')
                .split_whitespace()
                .collect()
        } else {
            phone_str.split('-').collect()
        };

        parts
            .iter()
            .map(|part| self.num.verbalize_digits(part, true))
            .collect::<Vec<_>>()
            .join("，")
    }

    fn _replace(&self, phone_string: &str, re: &Regex, mobile: bool) -> String {
        re.replace_all(phone_string, |caps: &Captures| {
            caps.get(0)
                .map(|m| self.phone2str(m.as_str(), mobile))
                .unwrap_or_default()
        })
        .to_string()
    }

    fn replace_phone(&self, phone_str: &str) -> String {
        self._replace(phone_str, &RE_TELEPHONE, false)
    }

    fn replace_phone2(&self, phone_str: &str) -> String {
        self._replace(phone_str, &RE_NATIONAL_UNIFORM_NUMBER, false)
    }

    fn replace_mobile(&self, phone_str: &str) -> String {
        self._replace(phone_str, &RE_MOBILE_PHONE, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_mobile_phone() {
        let phonecode = Phonecode::new();

        // 常见手机号码
        assert_eq!(phonecode.normalize("13512345678"), "幺三五幺二三四五六七八");

        // 带国家代码的手机号码
        assert_eq!(
            phonecode.normalize("+8613512345678"),
            "八六幺三五幺二三四五六七八"
        );

        // 带空格的手机号码
        assert_eq!(
            phonecode.normalize("+86 13512345678"),
            "八六，幺三五幺二三四五六七八"
        );
        assert_eq!(
            phonecode.normalize("+86 135 12345678"),
            "八六，幺三五，幺二三四五六七八"
        );
        assert_eq!(
            phonecode.normalize("+86 135 1234 5678"),
            "八六，幺三五，幺二三四，五六七八"
        );
        assert_eq!(
            phonecode.normalize("+861351234 5678"),
            "八六幺三五幺二三四，五六七八"
        );

        assert_eq!(phonecode.normalize("+8613512345 678"), "+8613512345 678");

        // 不符合规则的号码
        assert_eq!(phonecode.normalize("11111111111"), "11111111111");

        // 空字符串
        assert_eq!(phonecode.normalize(""), "");
    }

    #[test]
    fn test_normalize_telephone() {
        let phonecode = Phonecode::new();

        // 带区号的电话号码
        assert_eq!(
            phonecode.normalize("021-12345678"),
            "零二幺，幺二三四五六七八"
        );

        // 不带区号的电话号码
        assert_eq!(phonecode.normalize("12345678"), "幺二三四五六七八");

        // 长区号
        assert_eq!(
            phonecode.normalize("0531-7654321"),
            "零五三幺，七六五四三二幺"
        );

        // 错误格式的电话号码
        assert_eq!(phonecode.normalize("123-456"), "123-456");

        // 空字符串
        assert_eq!(phonecode.normalize(""), "");
    }

    #[test]
    fn test_normalize_national_uniform_number() {
        let phonecode = Phonecode::new();

        // 标准 400 开头号码
        assert_eq!(
            phonecode.normalize("400-123-4567"),
            "四零零，幺二三，四五六七"
        );

        // 无分隔符的 400 号码
        assert_eq!(phonecode.normalize("4001234567"), "四零零幺二三四五六七");

        // 错误格式的 400 号码
        assert_eq!(phonecode.normalize("400-123"), "400-123");

        // 空字符串
        assert_eq!(phonecode.normalize(""), "");
    }

    #[test]
    fn test_phone2str() {
        let phonecode = Phonecode::new();

        // 手机号码分段
        assert_eq!(
            phonecode.phone2str("13512345678", true),
            "幺三五幺二三四五六七八"
        );

        // 电话号码分段
        assert_eq!(
            phonecode.phone2str("021-12345678", false),
            "零二幺，幺二三四五六七八"
        );

        // 无分隔符电话号码
        assert_eq!(
            phonecode.phone2str("4001234567", false),
            "四零零幺二三四五六七"
        );

        // 空字符串
        assert_eq!(phonecode.phone2str("", true), "");
    }

    #[test]
    fn test_corner_cases() {
        let phonecode = Phonecode::new();

        // 非数字字符混合
        assert_eq!(
            phonecode.normalize("Call me at 13512345678 or 021-12345678."),
            "Call me at 幺三五幺二三四五六七八 or 零二幺，幺二三四五六七八."
        );

        // 多个电话号码混合
        assert_eq!(
            phonecode.normalize("400-123-4567 and +8613512345678"),
            "四零零，幺二三，四五六七 and 八六幺三五幺二三四五六七八"
        );

        // 错误前缀的号码
        assert_eq!(phonecode.normalize("99913512345678"), "99913512345678");

        // 极短或极长号码
        assert_eq!(phonecode.normalize("1"), "1");
        assert_eq!(
            phonecode.normalize("13512345678123456789"),
            "13512345678123456789"
        );
    }
}
