use crate::*;
pub use serde_json::json as value;
pub use serde_json::{
    Number, Value, from_slice as inner_from_slice, from_str as inner_from_str, from_value,
    to_string, to_value, to_vec,
};

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    if s.is_empty() {
        Ok(inner_from_str("null")?)
    } else {
        Ok(inner_from_str(s)?)
    }
}

pub fn from_slice<'a, T>(v: &'a [u8]) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    if v.is_empty() {
        Ok(inner_from_str("null")?)
    } else {
        Ok(inner_from_slice(v)?)
    }
}

#[cfg(feature = "api")]
pub async fn from_body<R>(body: api::Body) -> Result<R>
where
    R: for<'a> de::Deserialize<'a>,
{
    let bytes = axum::body::to_bytes(body, std::usize::MAX).await?;
    let result = from_slice(&bytes)?;
    Ok(result)
}

tests! {
    fn test_from_empty() {
        let empty: Result<()> = from_str("");
        assert!(empty.is_ok());
        let empty: Result<()> = from_str("null");
        assert!(empty.is_ok());
        let empty: Result<()> = from_str("()");
        assert!(empty.is_err());
        let empty: Result<()> = from_str("{}");
        assert!(empty.is_err());
        let empty: Result<()> = from_str("[]");
        assert!(empty.is_err());

        let empty: Result<()> = from_slice("".as_bytes());
        assert!(empty.is_ok());
        let empty: Result<()> = from_slice("null".as_bytes());
        assert!(empty.is_ok());
        let empty: Result<()> = from_slice("()".as_bytes());
        assert!(empty.is_err());
    }
}
