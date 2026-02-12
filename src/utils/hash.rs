use crate::*;
use sm3::{Digest, Sm3};

pub fn hash(message: impl AsRef<[u8]>) -> Vec<u8> {
    let mut hasher = Sm3::new();
    hasher.update(message);
    hasher.finalize().to_vec()
}

tests! {
    fn test_hash() {
        assert_eq!(
            hex::encode(hash(b"hello world")),
            "44f0061e69fa6fdfc290c494654a05dc0c053da7e5c52b84ef93a9d67d3fff88"
        );
        assert_eq!(
            base64::encode(hash(b"hello world")),
            "RPAGHmn6b9/CkMSUZUoF3AwFPaflxSuE75Op1n0//4g="
        );
        assert_eq!(
            hex::encode(hash("hello world")),
            "44f0061e69fa6fdfc290c494654a05dc0c053da7e5c52b84ef93a9d67d3fff88"
        );
        assert_eq!(
            base64::encode(hash("hello world")),
            "RPAGHmn6b9/CkMSUZUoF3AwFPaflxSuE75Op1n0//4g="
        );
    }
}
