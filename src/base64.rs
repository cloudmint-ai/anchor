use crate::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as base64};

pub fn encode<T: AsRef<[u8]>>(input: T) -> String {
    base64.encode(input)
}

pub fn decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>> {
    Ok(base64.decode(input)?)
}
