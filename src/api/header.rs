use crate::*;

pub const AUTHORIZATION: &str = "Authorization";
const SIGNATURE_PREFIX: &str = "Signature ";

pub const TRACE: &'static str = "Trace";

pub fn signature_header(signature_base64: String) -> String {
    format!("{}{}", SIGNATURE_PREFIX, signature_base64)
}

pub fn parse_signature(signature_header: String) -> Result<String> {
    if !signature_header.starts_with(SIGNATURE_PREFIX) {
        return Unexpected!("header from token");
    }
    Ok(signature_header
        .trim_start_matches(SIGNATURE_PREFIX)
        .to_string())
}
