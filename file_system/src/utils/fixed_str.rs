use std::fmt;
use std::fmt::Display;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "py-bindings")]
use pyo3::prelude::*;

#[derive(Debug, Clone, PartialEq, thiserror::Error, Serialize, Deserialize)]
enum NameError {
    #[error("Name too long: found {0}, max length is 56 including null terminator.")]
    NameTooLong(usize),
    #[error("Invalid name: {0}")]
    InvalidName(String),
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "py-bindings", pyclass)]
pub struct FixedString {
    pub value: String,
}

impl From<String> for FixedString {
    fn from(value: String) -> Self {
        FixedString::new(value).unwrap()
    }
}

impl From<&str> for FixedString {
    fn from(value: &str) -> Self {
        FixedString::new(value.to_owned()).unwrap()
    }
}

impl Display for FixedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value.trim_end_matches('\0'))
    }
}

impl FixedString {
    pub(crate) fn new(value: String) -> anyhow::Result<Self> {
        if value.len() > 56 {
            return Err(NameError::NameTooLong(value.len()).into());
        }

        Ok(FixedString { value })
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl Serialize for FixedString {
    fn serialize<S>(&self, serializer: S) -> anyhow::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buffer = [0u8; 56];
        let bytes = self.value.as_bytes();
        let length = bytes.len().min(55);
        buffer[..length].copy_from_slice(&bytes[..length]);
        serializer.serialize_bytes(&buffer)
    }
}

struct FixedStringVisitor;

impl<'de> Visitor<'de> for FixedStringVisitor {
    type Value = FixedString;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte array of length 56")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> anyhow::Result<Self::Value, E>
    where
        E: de::Error,
    {
        let end = v.iter().position(|&b| b == 0).unwrap_or(v.len());
        match std::str::from_utf8(&v[..end]) {
            Ok(s) => Ok(FixedString::new(s.to_owned()).map_err(E::custom)?),
            Err(err) => Err(E::custom(err.to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for FixedString {
    fn deserialize<D>(deserializer: D) -> anyhow::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(FixedStringVisitor)
    }
}
