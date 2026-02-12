mod error;
pub use error::*;

// 注意，panic 无法完美恢复和重试，因此异常尽可能用Result
pub type Result<T> = std::result::Result<T, Error>;
