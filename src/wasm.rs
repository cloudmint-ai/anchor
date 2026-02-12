use crate::*;
// TODO, napi 可以直接 通过 Into 用 Result，看wasm 是否也能
pub use wasm_bindgen::prelude::{JsValue, wasm_bindgen};

#[cfg(not(feature = "insecure"))]
use key::Verifier;

// Js 开头的各个变量类型 仅 用于 wasm 最外层被js调用的函数定义
pub type JsResult<T> = std::result::Result<T, JsValue>;

// JsResult 内的 Error 位，为 JsError 的序列化后的 JsValue
#[derive(Serialize)]
struct JsError {
    pub code: i64,
    pub message: String,
}

impl JsError {
    const UNEXPECTED_CODE: i64 = 500;

    pub fn engine(code: i64) -> Self {
        return Self {
            code,
            message: "".to_string(),
        };
    }
    pub fn unexpected(message: String) -> JsError {
        return Self {
            code: Self::UNEXPECTED_CODE,
            message,
        };
    }
}

// TODO , impl From will auto impl Into, impl INto not auto impl From
impl Into<JsValue> for JsError {
    fn into(self) -> JsValue {
        match serde_wasm_bindgen::to_value(&self) {
            Ok(value) => value,
            Err(e) => JsValue::from_str(&format!("JsError into JsValue fail {:?}", e)),
        }
    }
}

impl From<Error> for JsValue {
    fn from(error: Error) -> Self {
        match error {
            Error::EngineError(code) => JsError::engine(code).into(),
            Error::UnexpectedError(message) => JsError::unexpected(message).into(),
        }
    }
}

pub struct Engine {
    #[cfg(not(feature = "insecure"))]
    verifier: Verifier,
}

impl Engine {
    #[cfg(not(feature = "insecure"))]
    pub fn new() -> Result<Self> {
        let verifier = Verifier::new(&config!().wasm.public_key_hexes)?;
        Ok(Self { verifier })
    }

    #[cfg(not(feature = "insecure"))]
    pub fn from_value<T>(&self, value: JsValue, signature_base64: &str) -> Result<T>
    where
        T: de::DeserializeOwned,
    {
        let hash_value = self.hash(value.clone())?;
        self.verifier.verify(&hash_value, signature_base64)?;
        Ok(serde_wasm_bindgen::from_value(value)?)
    }

    // ONLY WORK IN DEVELOPMENT
    #[cfg(not(feature = "insecure"))]
    pub fn force_from_value<T>(&self, value: JsValue) -> Result<T>
    where
        T: de::DeserializeOwned,
    {
        if config::is_production() {
            return Unexpected!("force only work in development");
        } else {
            Ok(serde_wasm_bindgen::from_value(value)?)
        }
    }

    pub fn to_value<T>(&self, data: &T) -> Result<JsValue>
    where
        T: serde::ser::Serialize + ?Sized,
    {
        Ok(serde_wasm_bindgen::to_value(&data)?)
    }

    // 需要 base64 decode 之后再签名
    pub fn hash_base64(&self, value: JsValue) -> Result<String> {
        Ok(base64::encode(self.hash(value)?))
    }

    // 注意，util::hash 与 sm2/3 自带的 hash 逻辑有差别，不是平替关系
    // 实际上签名验签相当于做了两次不同的hash，外层hash仅用于降低网络传递消耗
    // TODO 后续 hash 改成动态公钥的签名？
    // TODO wasm 进行对称加密，内置对称加密密钥，然后外置私钥，方便更新
    fn hash(&self, value: JsValue) -> Result<Vec<u8>> {
        let json_value: json::Value = serde_wasm_bindgen::from_value(value)?;
        Ok(utils::hash(json::to_vec(&json_value)?))
    }
}

#[cfg(feature = "insecure")]
impl Engine {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn from_value<T>(&self, value: JsValue) -> Result<T>
    where
        T: de::DeserializeOwned,
    {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}

tests! {
    #[cfg(not(feature = "insecure"))]
    fn test_engine() {
        test::config!(config::Root{
            wasm: config::Wasm{
                public_key_hexes: vec!["04b9c9a6e04e9c91f7ba880429273747d7ef5ddeb0bb2ff6317eb00bef331a83081a6994b8993f3f5d6eadddb81872266c87c018fb4162f5af347b483e24620207".to_string()],
            },
            ..default!()
        });
        use key::{PrivateKey, verify_key_pair_matching};
        let private_key =
            PrivateKey::new("b9ab0b828ff68872f21a837fc303668428dea11dcd1b24429d0c99e24eed83d5")?;
        let public_key = key::PublicKey::new(
            "04b9c9a6e04e9c91f7ba880429273747d7ef5ddeb0bb2ff6317eb00bef331a83081a6994b8993f3f5d6eadddb81872266c87c018fb4162f5af347b483e24620207",
        )?;
        verify_key_pair_matching(&private_key, &public_key)?;

        let engine = Engine::new()?;
        let value = JsValue::from_str("a");
        let hash_base64 = engine.hash_base64(value.clone())?;
        let signature = format!(
            "{}.{}",
            public_key.brief(),
            private_key.sign(&base64::decode(hash_base64)?)?
        );
        let s: String = engine.from_value(value, &signature)?;
        assert_eq!(s, "a");

        let hash_base64 = "eyJmZmYiOiJjY2NjIiwibmFtZSI6ImhhaGFoYWhhIiwid29yZCI6IjExMSJ9";
        let signature = private_key.sign(&base64::decode(hash_base64)?)?;
        assert_eq!(
            signature,
            "MEQCICfwz+x0A3ns5sk6VNrjARczDZk2GksCk6GR8GOy4ydxAiBfM4ebK/wNYIwElBmnFj0XD5pRhh4T1UHS67umIYyR+A=="
        );
    }

    #[cfg(feature = "insecure")]
    fn test_insecure_engine() {
        let engine = Engine::new()?;
        let value = JsValue::from_str("a");
        let s: String = engine.from_value(value)?;
        assert_eq!(s, "a");
    }
}
