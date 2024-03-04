use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Debug;

use crate::errors::FileError;
use crate::utils::fixed_str::FixedString;
use crate::FileSystem;

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
    pub fn new(name: FixedString, file_type: FileType, size: u64, blk_num: u16) -> Self {
        DirEntry {
            name,
            file_type,
            size,
            blk_num,
        }
    }

    pub fn calculate_max_size() -> usize {
        let example_entry = Self::gen_max_size_entry();
        let serialized = bincode::serialize(&example_entry).unwrap();
        serialized.len()
    }

    pub fn get_size(&self) -> usize {
        let serialized = bincode::serialize(&self.clone()).unwrap();
        serialized.len()
    }

    pub fn gen_max_size_entry() -> DirEntry {
        DirEntry {
            name: FixedString::new(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            )
            .unwrap(),
            file_type: FileType::File,
            size: u64::MAX,
            blk_num: u16::MAX,
        }
    }
}

#[derive(Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct Block {
    #[serde(skip_deserializing, skip_serializing)]
    pub(crate) path: String,
    #[serde(skip_deserializing, skip_serializing)]
    pub(crate) parent_entry: DirEntry,
    #[serde(skip_deserializing, skip_serializing)]
    pub(crate) blk_num: u16,
    pub(crate) entries: Vec<DirEntry>,
}

impl Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // get number of filled entries vs total entries
        let filled = self
            .entries
            .iter()
            .filter(|entry| !entry.name.is_empty())
            .count();
        let total = self.entries.len();

        let filled_entries = self
            .entries
            .iter()
            .filter(|entry| !entry.name.is_empty())
            .collect::<Vec<_>>();
        write!(
            f,
            "Block {{ path: {}, parent_entry: {:?}, blk_num: {}, entries: {:?}, filled: {}/{} }}",
            self.path, self.parent_entry, self.blk_num, filled_entries, filled, total
        )
    }
}

impl Block {
    pub fn new(parent_entry: DirEntry, blk_num: u16) -> Self {
        let entries = vec![DirEntry::default(); FileSystem::num_entries()];
        Block {
            path: "".to_string(),
            parent_entry,
            blk_num,
            entries,
        }
    }

    pub fn get_size(&self) -> usize {
        let serialized = bincode::serialize(&self.clone()).unwrap();
        serialized.len()
    }

    pub fn gen_max_size_block() -> Block {
        Block {
            path: "".to_string(),
            parent_entry: DirEntry::gen_max_size_entry(),
            blk_num: u16::MAX,
            entries: vec![DirEntry::gen_max_size_entry(); FileSystem::num_entries()],
        }
    }

    pub fn calculate_max_size() -> usize {
        let example_block = Self::gen_max_size_block();
        let serialized = bincode::serialize(&example_block).unwrap();
        serialized.len()
    }

    pub fn get_entry(&self, name: &FixedString) -> Option<&DirEntry> {
        self.entries.iter().find(|entry| entry.name == *name)
    }

    pub fn get_entry_mut(&mut self, name: &FixedString) -> Option<&mut DirEntry> {
        self.entries.iter_mut().find(|entry| entry.name == *name)
    }

    pub fn add_entry(&mut self, entry: DirEntry) -> Result<()> {
        if let Some(index) = self.entries.iter().position(|item| item.name.is_empty()) {
            self.entries[index] = entry;
            Ok(())
        } else {
            Err(FileError::FileNotFound.into())
        }
    }

    /// Removes an entry from the block.
    /// Get the block num before this as this will be deleted after this operation
    pub fn remove_entry(&mut self, name: &FixedString) -> Result<()> {
        if let Some(index) = self.entries.iter().position(|item| item.name == *name) {
            self.entries[index] = DirEntry::default();
            Ok(())
        } else {
            Err(FileError::FileNotFound.into())
        }
    }
}
