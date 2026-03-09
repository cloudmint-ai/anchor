use crate::*;
use de::{self, Unexpected, Visitor};

// TODO 仅在测试框架里自动组装？
pub trait Versioned: Entity {
    // 仅用于测试框架
    fn _current_version(&self) -> Version;
    // 仅用于测试框架
    fn _increase_version(&mut self);
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Hash, Eq)]
pub struct Version(i64);

impl Default for Version {
    fn default() -> Self {
        Self(0)
    }
}

impl std::fmt::Debug for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 0 表示未锁未存储的内存状态
/// 一旦第一次持久化，预期持久化的 version 为 1, 数据库里会从1 开始
/// 初始化时 使用 default!() 返回 0
/// 暂定Version 在内存里不应该有变化，在数据库提交后应该重新获取新版本Version
impl Version {
    fn new(value: i64) -> Self {
        Self(value)
    }
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    // TODO rm increase
    pub fn increase(&mut self) {
        self.0 += 1;
    }
}

impl From<i64> for Version {
    fn from(value: i64) -> Version {
        Version::new(value)
    }
}

impl From<Version> for i64 {
    fn from(value: Version) -> Self {
        value.0
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(VersionVisitor)
    }
}

struct VersionVisitor;

impl<'de> Visitor<'de> for VersionVisitor {
    type Value = Version;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an integer or a string representing an integer")
    }

    fn visit_i64<E>(self, value: i64) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Version::from(value))
    }

    fn visit_u64<E>(self, value: u64) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        let value: i64 = value.must_into();
        Ok(Version::from(value))
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value.parse::<i64>() {
            Ok(parsed) => Ok(Version::from(parsed)),
            Err(_) => Err(de::Error::invalid_value(Unexpected::Str(value), &self)),
        }
    }
}

// 支持 Version 类型直接参与 sqlx 宏拼接，目前需要手动书写 as Version
#[cfg(feature = "cloud")]
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for Version {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> std::result::Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <i64 as sqlx::Encode<'_, sqlx::Postgres>>::encode_by_ref(&self.0, buf)
    }
}

// 支持 Version 类型直接参与 sqlx 宏拼接，目前需要手动书写 as Version
#[cfg(feature = "cloud")]
impl sqlx::Type<sqlx::Postgres> for Version {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <i64 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

tests! {
    fn test_version() {
        let version = Version::from(123);
        assert_eq!(format!("{}", version), "123");

        let serialized = json::to_string(&version)?;
        assert_eq!(serialized, "\"123\"");

        let deserialized: Version = json::from_str("\"123\"")?;
        assert_eq!(i64::from(deserialized), i64::from(version));

        let deserialized: Version = json::from_str("123")?;
        assert_eq!(i64::from(deserialized), i64::from(version));

        let default_version: Version = default!();
        assert_eq!(i64::from(default_version), 0);

        let default_version: Version = default!();
        assert!(default_version.is_zero());
    }
}
