// TODO 完善所有内容的 三斜线文档注释
/// 添加 mod tests {} 在内的必须的测试架构代码
/// 对 test_ 开头的函数自动添加 test::case 宏
#[macro_export]
macro_rules! tests {
    ($($item:item)*) => {
        #[cfg(test)]
        #[__::test::__::cases]
        mod tests {
            #[allow(unused_imports)]
            use __::test;
            #[allow(unused_imports)]
            use super::*;

            $($item)*
        }
    };
}
