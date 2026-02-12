use crate::*;
use de::{self, Deserializer, Visitor};
use log::Level;

#[derive(Debug, Clone, Deserialize)]
pub struct Log {
    #[serde(default)]
    pub file_name: String,
    #[serde(deserialize_with = "deserialize_level")]
    pub level: Level,
}

impl Default for Log {
    fn default() -> Self {
        Self {
            file_name: "".to_owned(),
            level: Level::INFO,
        }
    }
}

fn deserialize_level<'de, D>(deserializer: D) -> std::result::Result<Level, D::Error>
where
    D: Deserializer<'de>,
{
    struct LevelVisitor;

    impl<'de> Visitor<'de> for LevelVisitor {
        type Value = Level;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string representing the log level")
        }

        fn visit_str<E>(self, value: &str) -> std::result::Result<Level, E>
        where
            E: de::Error,
        {
            value.parse::<Level>().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_str(LevelVisitor)
}
