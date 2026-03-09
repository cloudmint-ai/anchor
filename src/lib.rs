pub mod __;
use __::config;

mod result;
pub use result::*;

pub use std::str::FromStr;
pub use strum::Display as EnumDisplay;
pub use strum::EnumIter;
pub use strum::EnumString;
pub use strum::IntoEnumIterator;

// TODO rm serde package
// TODO only use in engine
pub use serde as _serde;
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

/// 使用 async_trait 应该主动使用async feature
#[cfg(feature = "async")]
pub use async_trait::async_trait;

pub use std::sync::{Arc, LazyLock, OnceLock};

pub use std::borrow::Cow;

// 由于automock 修饰的代码往往在非tests场景下，故直接暴露
pub use mockall::automock;

#[cfg(feature = "api")]
pub mod api;

// TODO make RequestClient with Trace and check in_transaction
// TODO make reqwest use context cancel
#[cfg(any(feature = "api", feature = "wasm"))]
pub use reqwest as http;

pub mod key;

#[cfg(not(feature = "wasm"))]
pub use std::env;

#[cfg(feature = "async")]
pub use macros::main;
#[cfg(feature = "async")]
mod runtime;

#[cfg(feature = "async")]
pub use runtime::*;

#[cfg(feature = "async")]
mod context;
#[cfg(feature = "async")]
pub use context::*;

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
