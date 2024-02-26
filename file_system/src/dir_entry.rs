use serde_derive::{Deserialize, Serialize};
use std::mem;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum FileType {
    #[default]
    File,
    Directory,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct DirEntry {
    pub(crate) name: String,
    pub(crate) file_type: FileType,
    pub(crate) size: u64,
    pub(crate) blk_num: u16,
}

impl DirEntry {
    pub const MAX_SIZE: usize =
        55 + mem::size_of::<FileType>() + mem::size_of::<u64>() + mem::size_of::<u64>();
}

impl Default for DirEntry {
    fn default() -> Self {
        DirEntry {
            name: "".to_string(),
            file_type: FileType::default(),
            size: 0,
            blk_num: 0,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct Block {
    pub(crate) parent_entry: DirEntry,
    #[serde(skip_deserializing, skip_serializing)]
    pub(crate) blk_num: u64,
    pub(crate) entries: Vec<DirEntry>,
}
