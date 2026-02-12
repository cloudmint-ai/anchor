use crate::*;
/// 使用 regex! 提升初始化性能
pub use regex::Regex as _Regex;

#[macro_export]
macro_rules! regex {
    // 支持返回 String 类型的闭包
    (|| $body:expr) => {{
        static LOCK: LazyLock<_Regex> = LazyLock::new(|| {
            let closure = || $body;
            let pattern = closure();
            _Regex::new(&pattern).expect(&format!("{} is not valid regex", pattern))
        });
        let regex: _Regex = LOCK.clone();
        regex
    }};

    ($pattern:expr) => {{
        static LOCK: LazyLock<_Regex> = LazyLock::new(|| {
            _Regex::new($pattern).expect(&format!("{} is not valid regex", $pattern))
        });
        let regex: _Regex = LOCK.clone();
        regex
    }};
}

tests! {
    fn test_regex() {
        let r = regex!(r"[a-zA-Z']+");
        assert!(r.is_match("aaa"));

        let r = regex!(|| r"kk".to_string() + r"[a-zA-Z']+");
        assert!(!r.is_match("aaa"));
        assert!(r.is_match("kka"));
    }
}
