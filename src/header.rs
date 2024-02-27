use std::borrow::Cow;

use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Header {
    pub datatype: Vec<ColumnSpecifier>,

    #[serde(default)]
    pub delimiter: Delimiter,

    #[serde(default)]
    pub meta: serde_yaml::Value,

    pub schema: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColumnSpecifier {
    name: String,

    datatype: DataType,

    subtype: Option<String>,

    unit: Option<String>,

    format: Option<String>,

    description: Option<String>,

    #[serde(default)]
    meta: serde_yaml::Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum DataType {
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Unt8,
    Unt16,
    Unt32,
    Unt64,
    Float16,
    Float32,
    Float64,
    Float128,
    Complex64,
    Complex128,
    Complex256,
    String,
}

impl Serialize for DataType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s: &'static str = self.into();
        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DataType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: Cow<'de, str> = Deserialize::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::EnumString)]
pub enum Delimiter {
    Space,
    Comma,
}

impl Default for Delimiter {
    fn default() -> Self {
        Self::Space
    }
}

impl Serialize for Delimiter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s: &'static str = self.into();
        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Delimiter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: Cow<'de, str> = Deserialize::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}
