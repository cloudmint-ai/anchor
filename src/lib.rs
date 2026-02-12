mod result;
pub use result::*;

pub use std::str::FromStr;
pub use strum::Display as EnumDisplay;
pub use strum::EnumIter;
pub use strum::EnumString;
pub use strum::IntoEnumIterator;

pub use serde::*;
pub use std::fmt;

pub use rand;
pub use rand::prelude::*;

pub mod log;
pub use tracing::{debug, error, info, warn};

mod types;
pub use types::*;

// utils 内包含无主题的函数，若有明确主题，应外置至 crate
pub mod utils;

// 外置的 utils 主题
pub mod base64;
pub mod json;
pub use hex;
pub use toml;

pub mod time;
pub use std::time::Duration;
pub use time::{TimeDelta, TimeElapsed, Timestamp};

pub use async_trait::async_trait;

pub use std::sync::Arc;
pub use std::sync::LazyLock;

pub use std::borrow::Cow;

mod supply;

#[cfg(any(test, feature = "test"))]
pub mod test;

#[cfg(feature = "api")]
pub mod api;

// TODO make RequestClient with Trace and check in_transaction
// TODO make reqwest use context cancel
#[cfg(any(feature = "api", feature = "wasm"))]
pub use reqwest as http;

// TODO #[cfg(feature = "api")]
#[cfg(feature = "cloud")]
pub mod database;

#[cfg(any(feature = "runtime", feature = "wasm"))]
mod init;

pub mod key;

#[cfg(not(feature = "wasm"))]
pub use std::env;

#[cfg(feature = "runtime")]
pub use macros::main;
// export tokio for macro
#[cfg(feature = "runtime")]
pub use tokio;

#[cfg(feature = "runtime")]
pub mod runtime;

mod context;
pub use context::{_Transaction, Context};

pub mod _config;
use _config as config;

pub use mockall::automock;

#[cfg(feature = "napi")]
pub mod napi;

// 注意，间接引用的napi宏无法正常的使用 napi(constructor)
#[cfg(feature = "napi")]
pub use napi_derive_ohos::napi;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "python")]
pub mod python;

#[macro_export]
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),* $(,)?) => {{
        let mut map = HashMap::new();
        $(
            map.insert($key, $val);
        )*
        map
    }};

    ($val_type:ty; $( $key:expr => $val:expr ),* $(,)?) => {{
        let mut map: HashMap<_, $val_type> = HashMap::new();
        $(
            map.insert($key, $val);
        )*
        map
    }};
}

#[macro_export]
macro_rules! default {
    () => {
        Default::default()
    };
}

// TODO 使用扩展让 vscode 对非 mod.rs 自动加上调用
// TODO 追踪自定义 prelude 的 进展
#[macro_export]
macro_rules! __ {
    () => {
        use crate::anchor::*;
    };
}
