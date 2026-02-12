use super::Protocol;
use crate::*;

#[derive(Debug, Serialize, Deserialize, Protocol)]
pub struct Request {
    pub timestamp: time::Timestamp,
    pub request_id: Id,
}

impl Request {
    pub const EMPTY: EmptyRequest = EmptyRequest {};

    pub fn new(id: Id) -> Self {
        Self {
            timestamp: time::now(),
            request_id: id.into(),
        }
    }
}

#[derive(Protocol)]
pub struct EmptyRequest;

impl Serialize for EmptyRequest {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str("")
    }
}

impl<'de> Deserialize<'de> for EmptyRequest {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(EmptyRequest)
        } else {
            Err(serde::de::Error::custom("expected an empty string"))
        }
    }
}
