use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{Capability, Surface};
use crate::Error;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Power {
    pub capability: Capability,
    pub detail: Option<String>,
}

impl Power {
    pub fn new(capability: Capability) -> Self {
        Self {
            capability,
            detail: None,
        }
    }

    pub fn with_detail(self, detail: impl Into<String>) -> Self {
        Self {
            detail: Some(detail.into()),
            ..self
        }
    }
}

impl fmt::Display for Power {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(formatter, "{}:{detail}", self.capability),
            None => write!(formatter, "{}", self.capability),
        }
    }
}

impl FromStr for Power {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text.split_once(':') {
            Some((name, detail)) => Ok(Self::new(name.parse()?).with_detail(detail)),
            None => Ok(Self::new(text.parse()?)),
        }
    }
}

impl Serialize for Power {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Power {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Scope {
    pub surface: Surface,
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Evidence {
    pub path: String,
    pub line: usize,
    pub rule: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub call_path: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Detection {
    pub power: Power,
    pub line: usize,
    pub rule: String,
}

#[derive(Clone, Copy, Debug)]
pub struct SourceFile<'a> {
    pub path: &'a str,
    pub text: &'a str,
}
