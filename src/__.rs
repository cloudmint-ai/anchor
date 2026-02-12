// TODO 按需在__ 下放置额外的架构目录，如 engine 啥的

// __ 内的逻辑 仅由架构宏引入
#[cfg(any(feature = "async", feature = "wasm"))]
mod init;
#[cfg(feature = "async")]
mod supply_async;

#[cfg(not(feature = "async"))]
mod supply;

pub mod config;
pub mod test;

pub use macros::engine_for_runtime;
pub use macros::{data_for_engine, engine_for_engine};

// TODO 使用扩展让 vscode 对非 mod.rs 自动加上调用
// TODO 追踪自定义 prelude 的 进展
// 注意，通过 super::* 层层联动，可以使得内部能够看到 super::super::super.. 的所有代码
// 因此 各个函数尽可能保证正确的可见性配置
#[macro_export]
macro_rules! __ {
    (engine) => {
        // TODO engine feature?
        use anchor::__::data_for_engine as data;
        use anchor::__::engine_for_engine as engine;
        use anchor::*;
    };
    (runtime) => {
        use anchor::__::engine_for_runtime as engine;
        use anchor::__init_for_runtime as init;
        use anchor::*;
        __use_supply_for_runtime!();
    };
    (service) => {
        use anchor::__init_for_runtime as init;
        use anchor::*;
        use runtime::*;
    };
    (config) => {
        pub use anchor::__::config::*;
        use anchor::*;
    };
    (build) => {
        use anchor::__init_for_runtime as init;
        use anchor::*;
    };
}
