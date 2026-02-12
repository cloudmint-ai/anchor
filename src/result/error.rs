use crate::*;
use std::{backtrace::Backtrace, fmt::Display};

#[derive(Serialize, Deserialize)]
pub enum Error {
    EngineError(i64),
    UnexpectedError(String),
}

impl Error {
    fn get_full_message(message: String) -> String {
        if config::is_production() {
            return message;
        }
        let backtrace = Backtrace::force_capture();
        let backtrace_str = format!("{:?}", backtrace);
        let frame_strings: Vec<&str> = backtrace_str.split("}, {").collect();
        let mut result = String::new();
        for frame_string in frame_strings {
            let frame_string = &frame_string.replace("\\", "/");
            if !frame_string.contains(", file: ") {
                continue;
            }
            if frame_string.contains("Error::get_full_message") {
                continue;
            }
            if frame_string.contains("Error::unexpected") {
                continue;
            }
            if frame_string.contains("src/result/error.rs") {
                continue;
            }
            if frame_string.contains("/rustc/") {
                continue;
            }
            if frame_string.contains("/registry/src/") {
                continue;
            }
            if frame_string.contains("rustup/toolchains") {
                continue;
            }
            if !result.is_empty() {
                result.push_str("}\n{");
            }
            result.push_str(frame_string);
        }
        format!("{}, backtrace[\n{{{}}}\n]", message, result)
    }

    // 业务逻辑应该直接使用 unexpected! 和 Unexpected！ 宏
    pub fn _unexpected<D>(d: D) -> Error
    where
        D: Display,
    {
        let full_message = Self::get_full_message(d.to_string());
        Error::UnexpectedError(full_message)
    }

    // 业务逻辑应该直接使用 none! 宏
    pub fn _none() -> Error {
        let full_message = Self::get_full_message("none".to_string());
        Error::UnexpectedError(full_message)
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::EngineError(code) => {
                write!(f, "EngineError({})", code)
            }
            Error::UnexpectedError(message) => {
                write!(f, "UnexpectedError({})", message)
            }
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl<E> From<E> for Error
where
    E: std::error::Error,
{
    fn from(err: E) -> Self {
        Error::_unexpected(err)
    }
}

#[cfg(feature = "python")]
impl From<Error> for pyo3::prelude::PyErr {
    fn from(err: Error) -> pyo3::prelude::PyErr {
        let message = match err {
            Error::EngineError(code) => format!("EngineError({})", code),
            Error::UnexpectedError(message) => format!("UnexpectedError({})", message),
        };
        pyo3::exceptions::PyValueError::new_err(message)
    }
}

#[macro_export]
macro_rules! Unexpected {
    ($($arg:tt)*) => {
        Err(Error::_unexpected(format!($($arg)*)))
    };
}

#[macro_export]
macro_rules! unexpected {
    ($($arg:tt)*) => {
        Error::_unexpected(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! none {
    () => {
        || Error::_none()
    };
}
