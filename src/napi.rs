use crate::*;

pub use napi_ohos::{
    Env,
    threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};

use napi_ohos::Error as JsError;

impl From<Error> for JsError {
    fn from(value: Error) -> Self {
        match value {
            Error::EngineError(code) => JsError::from_reason(code.to_string()),
            Error::UnexpectedError(message) => JsError::from_reason(message),
        }
    }
}
