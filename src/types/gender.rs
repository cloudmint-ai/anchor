use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Gender {
    Male,
    Female,
}

impl Serialize for Gender {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let gender_str = match self {
            Gender::Male => "M",
            Gender::Female => "F",
        };
        serializer.serialize_str(gender_str)
    }
}

impl<'de> Deserialize<'de> for Gender {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        if s == "M" {
            Ok(Gender::Male)
        } else if s == "F" {
            Ok(Gender::Female)
        } else {
            Err(de::Error::invalid_value(de::Unexpected::Str(&s), &"M or F"))
        }
    }
}
