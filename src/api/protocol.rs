use crate::*;
use de::DeserializeOwned;

// 避免 Result<T> 和 T 均为 Deserialize, 所以需要明确一个trait
// 由 crate::api::Protocol 自动 Derive
// 让它能够限制 api 的返回值甚至Query 参数
pub trait Protocol: Serialize + DeserializeOwned {}

impl Protocol for () {}
