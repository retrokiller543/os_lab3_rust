// Helper functions and structs

use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor};
use std::fmt;
use std::fmt::Display;
use anyhow::Result;
use rustic_disk::traits::BlockStorage;
use crate::dir_entry::{Block, DirEntry};
use crate::FileSystem;

#[derive(Debug, Clone, PartialEq, thiserror::Error, Serialize, Deserialize)]
enum NameError {
    #[error("Name too long: found {0}, max length is 56 including null terminator.")]
    NameTooLong(usize),
    #[error("Invalid name: {0}")]
    InvalidName(String),
}

#[derive(Debug, Default, Clone, PartialEq)]
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
    pub(crate) fn new(value: String) -> Result<Self> {
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
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

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
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
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(FixedStringVisitor)
    }
}

impl FileSystem {
    pub fn read_dir_block(&self, entry: &DirEntry) -> Result<Block> {
        if entry.file_type != crate::dir_entry::FileType::Directory {
            return Err(crate::errors::FileError::NotADirectory(entry.clone().name).into());
        }
        let block_num = entry.blk_num;
        let mut block = self.disk.read_block::<Block>(block_num as usize)?;

        block.parent_entry = entry.clone();
        block.blk_num = block_num as u64;

        Ok(block)
    }
}
