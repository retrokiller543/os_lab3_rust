use serde_derive::{Deserialize, Serialize};
use std::mem;
use crate::utils::FixedString;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum FileType {
    #[default]
    File,
    Directory,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct DirEntry {
    pub(crate) name: FixedString,
    pub(crate) file_type: FileType,
    pub(crate) size: u64,
    pub(crate) blk_num: u16,
}

impl DirEntry {
    pub const MAX_SIZE: usize =
        55 + mem::size_of::<FileType>() + mem::size_of::<u64>() + mem::size_of::<u64>();
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct Block {
    #[serde(skip_deserializing, skip_serializing)]
    pub(crate) parent_entry: DirEntry,
    #[serde(skip_deserializing, skip_serializing)]
    pub(crate) blk_num: u64,
    pub(crate) entries: Vec<DirEntry>,
}
