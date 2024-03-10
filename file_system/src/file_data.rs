use std::ops::Add;

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "py-bindings", pyo3::pyclass)]
pub struct FileData {
    pub(crate) data: Vec<u8>,
}

impl FileData {
    pub fn new(data: Vec<u8>) -> Self {
        FileData { data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get_size(&self) -> usize {
        let serialized = bincode::serialize(&self).unwrap();
        serialized.len()
    }
}

impl From<&str> for FileData {
    fn from(data: &str) -> Self {
        FileData {
            data: data.as_bytes().to_vec(),
        }
    }
}

impl From<String> for FileData {
    fn from(data: String) -> Self {
        FileData {
            data: data.as_bytes().to_vec(),
        }
    }
}

impl From<Vec<u8>> for FileData {
    fn from(data: Vec<u8>) -> Self {
        FileData { data }
    }
}

impl From<FileData> for Vec<u8> {
    fn from(data: FileData) -> Self {
        data.data
    }
}

impl From<FileData> for String {
    fn from(data: FileData) -> Self {
        String::from_utf8(data.data).unwrap()
    }
}

impl Add for FileData {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut data = self.data;
        data.extend(other.data);
        FileData { data }
    }
}
