use crate::*;
use de::{self, Unexpected, Visitor};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Hash, Eq)]
pub struct Id(i64);

impl Id {
    pub const ZERO: Self = Self(0);
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Id> for i64 {
    fn from(value: Id) -> i64 {
        value.0
    }
}

impl From<i64> for Id {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(IdVisitor)
    }
}

struct IdVisitor;

impl<'de> Visitor<'de> for IdVisitor {
    type Value = Id;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an integer or a string representing an integer")
    }

    fn visit_i64<E>(self, value: i64) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Id(value))
    }

    fn visit_u64<E>(self, value: u64) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Id(value.must_into()))
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value.parse::<i64>() {
            Ok(parsed) => Ok(Id(parsed)),
            Err(_) => Err(de::Error::invalid_value(Unexpected::Str(value), &self)),
        }
    }
}

// 支持 Id 类型直接参与 sqlx 宏拼接，目前需要手动书写 as Id
#[cfg(feature = "cloud")]
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for Id {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> std::result::Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i64 as sqlx::Encode<'_, sqlx::Postgres>>::encode_by_ref(&self.0, buf)
    }
}

// 支持 Id 类型直接参与 sqlx 宏拼接，目前需要手动书写 as Id
#[cfg(feature = "cloud")]
impl sqlx::Type<sqlx::Postgres> for Id {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <i64 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

tests! {
    fn test_id() {
        let id = Id(123);
        assert_eq!(format!("{}", id), "123");

        let serialized = json::to_string(&id)?;
        assert_eq!(serialized, "\"123\"");

        let deserialized: Id = json::from_str("\"123\"")?;
        assert_eq!(deserialized, id);
        let deserialized: i64 = deserialized.into();
        assert_eq!(deserialized, 123);

        let deserialized: Id = json::from_str("123")?;
        assert_eq!(deserialized, id);
        let deserialized: i64 = deserialized.into();
        assert_eq!(deserialized, 123);
    }
}
