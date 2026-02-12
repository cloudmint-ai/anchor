use crate::*;
use de::{self, Unexpected, Visitor};

#[cfg(not(feature = "async"))]
use std::sync::{
    RwLock,
    atomic::{AtomicI64, Ordering},
};

pub trait Versioned: Entity {
    // 仅用于测试框架
    fn _current_version(&self) -> &Version;
}

#[derive(Clone)]
pub struct Version {
    current: Arc<AtomicI64>,
    rw_lock: Arc<RwLock<i64>>,
}

impl Default for Version {
    fn default() -> Self {
        Self {
            current: Arc::new(AtomicI64::new(default!())),
            rw_lock: Arc::new(RwLock::new(default!())),
        }
    }
}

impl std::fmt::Debug for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.into_inner())
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.into_inner())
    }
}

// 0 表示未锁未存储的内存状态
// 一旦进入事务加锁，则会自增, 数据库里会从1开始
// 初始化时 使用 default!() 返回 0
impl Version {
    const ORDERING: Ordering = Ordering::SeqCst;

    fn into_inner(&self) -> i64 {
        self.current.load(Self::ORDERING)
    }
    fn new(value: i64) -> Self {
        Self {
            current: Arc::new(AtomicI64::new(value)),
            rw_lock: Arc::new(RwLock::new(value)),
        }
    }

    // 仅用于框架
    pub fn _decrease(&self) -> Result<()> {
        let mut guard = self.rw_lock.try_write()?;
        *guard -= 1;
        self.current.store(*guard, Self::ORDERING);
        Ok(())
    }

    // 仅用于框架
    pub fn _increase(&self) -> Result<()> {
        let mut guard = self.rw_lock.try_write()?;
        *guard += 1;
        self.current.store(*guard, Self::ORDERING);
        Ok(())
    }

    pub fn is_zero(&self) -> bool {
        self.into_inner() == 0
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.into_inner() == other.into_inner()
    }
}

impl From<i64> for Version {
    fn from(value: i64) -> Version {
        Version::new(value)
    }
}

impl From<Version> for i64 {
    fn from(value: Version) -> Self {
        value.into_inner()
    }
}

impl From<&Version> for i64 {
    fn from(value: &Version) -> Self {
        value.into_inner()
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.into_inner().to_string())
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
        <i64 as sqlx::Encode<'_, sqlx::Postgres>>::encode_by_ref(&self.into_inner(), buf)
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
        assert_eq!(i64::from(deserialized), i64::from(&version));

        let deserialized: Version = json::from_str("123")?;
        assert_eq!(i64::from(deserialized), i64::from(&version));

        let default_version: Version = default!();
        assert_eq!(i64::from(&default_version), 0);

        default_version._increase()?;
        assert_eq!(i64::from(&default_version), 1);
        default_version._increase()?;
        assert_eq!(i64::from(&default_version), 2);
        default_version._increase()?;
        assert_eq!(default_version, Version::from(3));
        assert_eq!(i64::from(&default_version), 3);

        let three: i64 = default_version.into();
        assert_eq!(three, 3);

        let default_version: Version = default!();
        assert!(default_version.is_zero());
    }
}
