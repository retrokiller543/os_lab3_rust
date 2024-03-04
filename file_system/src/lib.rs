#![allow(unused_variables)]

use std::fmt::Debug;
use std::ops::{Index, IndexMut};

use anyhow::Result;
#[cfg(feature = "debug")]
use log::{debug, trace};
use logger_macro::trace_log;
use serde::Serialize;
use serde_derive::Deserialize;

use rustic_disk::traits::BlockStorage;
use rustic_disk::Disk;

use crate::dir_entry::{Block, DirEntry, FileType};
use crate::errors::FSError;
use crate::files::FileData;

mod dir_entry;
mod directories;
mod errors;
mod files;
mod format;
mod other;
pub mod prelude;
mod tests;
pub mod traits;
mod utils;

const ROOT_BLK: u64 = 0;
const FAT_BLK: u64 = 1;

pub struct FileSystem {
    disk: Disk,
    curr_block: Block,
    fat: FAT,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum FatType {
    Free,
    Taken(u16),
    EOF,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct FAT(
    //#[serde(with = "BigArray")]
    Vec<FatType>,
);

impl Debug for FAT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // get number of free blocks
        let num_free = self.0.iter().filter(|&x| *x == FatType::Free).count();
        // get number of EOF blocks
        let num_eof = self.0.iter().filter(|&x| *x == FatType::EOF).count();
        // get number of taken blocks
        let num_taken = self
            .0
            .iter()
            .filter(|&x| matches!(x, FatType::Taken(_)))
            .count();
        // get number of blocks
        let num_blocks = self.0.len();
        write!(
            f,
            "FAT{{Free: {}, Taken: {}, EOF: {}, Total: {}}}",
            num_free, num_taken, num_eof, num_blocks
        )
    }
}

impl FAT {
    #[trace_log]
    pub fn new() -> Self {
        let mut fat = vec![FatType::Free; (Disk::BLOCK_SIZE >> 2) - 8]; // 8 bytes is from padding in FAT struct
        fat.fill(FatType::Free);
        FAT(fat)
    }

    // Create an iterator
    #[trace_log]
    pub fn iter(&self) -> FatIterator {
        FatIterator {
            fat: self,
            position: 0,
        }
    }

    #[trace_log]
    pub fn get(&self, index: usize) -> Option<&FatType> {
        self.0.get(index)
    }
}

impl Default for FAT {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for FAT {
    type Output = FatType;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for FAT {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

// Define the iterator struct
pub struct FatIterator<'a> {
    fat: &'a FAT,
    position: usize,
}

// Implement the Iterator trait for FatIterator
impl<'a> Iterator for FatIterator<'a> {
    type Item = &'a FatType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= Disk::NUM_BLOCKS {
            None
        } else {
            let result = &self.fat.0[self.position];
            self.position += 1;
            Some(result)
        }
    }
}

impl FileSystem {
    pub fn num_entries() -> usize {
        Disk::BLOCK_SIZE / DirEntry::calculate_max_size()
    }

    #[trace_log]
    pub fn new() -> Result<Self> {
        let (curr_block, fat, disk) = if !Disk::disk_exists() {
            let disk = Disk::new()?;
            let fat = FAT::new();
            let root_block = Block {
                path: "/".to_string(),
                parent_entry: DirEntry {
                    name: "/".into(),
                    file_type: FileType::Directory,
                    ..Default::default()
                },
                blk_num: 0,
                entries: vec![DirEntry::default(); Self::num_entries()],
            };
            disk.write_block(0, &root_block)?;
            disk.write_block(1, &fat)?;
            (root_block, fat, disk)
        } else {
            let disk = Disk::new()?;
            let root_block: Block = disk.read_block(0)?;
            let fat: FAT = disk.read_block(1)?;
            (root_block, fat, disk)
        };

        #[cfg(feature = "debug")]
        {
            trace!("Root block: {:?}", curr_block);
            trace!("FAT: {:?}", fat);
        }

        Ok(FileSystem {
            disk,
            curr_block,
            fat,
        })
    }

    #[trace_log]
    pub fn write_curr_blk(&self) -> Result<()> {
        let block_to_write = self.curr_block.blk_num;
        self.disk
            .write_block(block_to_write as usize, &self.curr_block)?;
        Ok(())
    }

    #[trace_log]
    pub fn get_free_block(&self) -> Result<u16> {
        let mut blk = 0;

        for (index, block) in self.fat.iter().enumerate() {
            match block {
                FatType::Free => {
                    blk = index as u16;
                    break;
                }
                _ => continue,
            }
        }

        if blk == 0 {
            return Err(FSError::NoFreeBlocks.into());
        }

        Ok(blk)
    }

    #[trace_log]
    pub fn write_data<T: Serialize + Debug>(&mut self, data: &T, start_blk: u16) -> Result<()> {
        // Serialize the data
        let serialized_data = bincode::serialize(data).map_err(FSError::SerializationError)?;

        // If the data fits within a single block, write it directly
        if serialized_data.len() <= Disk::BLOCK_SIZE {
            self.disk
                .write_raw_data(start_blk as usize, &serialized_data)?;
            // Update FAT for start_blk to EOF since it's the last block
            self.update_fat(start_blk, None)?; // Assuming update_fat takes an Option<u64> for the second param
            return Ok(());
        }

        // Split into chunks for larger data
        let mut chunks = serialized_data.chunks(Disk::BLOCK_SIZE).peekable();
        let mut blk = start_blk;
        let mut first_iteration = true;

        while let Some(chunk) = chunks.next() {
            if !first_iteration {
                blk = self.get_free_block()?; // Get a new block if not the first iteration
            } else {
                first_iteration = false;
            }
            self.disk.write_raw_data(blk as usize, chunk)?;
            let next_blk = if chunks.peek().is_some() {
                Some(self.get_free_block()?)
            } else {
                None
            };

            // Update FAT for blk. If next_blk is None, it's the last chunk
            self.update_fat(blk, next_blk)?;
        }

        Ok(())
    }

    #[trace_log]
    pub fn update_fat(&mut self, blk: u16, next_blk: Option<u16>) -> Result<()> {
        match next_blk {
            Some(next_blk) => {
                self.fat[blk as usize] = FatType::Taken(next_blk);
            }
            None => {
                self.fat[blk as usize] = FatType::EOF;
            }
        }
        self.disk.write_block(FAT_BLK as usize, &self.fat)?;
        Ok(())
    }

    // Method to read all blocks of a file in order following the FAT table
    #[trace_log]
    pub fn read_file_data(&self, start_blk: u16) -> Result<FileData> {
        let mut data = FileData::default();
        let mut blk_num = start_blk;

        // Recursive closure to read blocks following the FAT
        let read_blocks_recursively = |blk_num: &mut u16, data: &mut Vec<u8>| -> Result<()> {
            loop {
                match self.fat.get(*blk_num as usize) {
                    Some(&FatType::Taken(next_blk)) => {
                        let block: FileData = self.disk.read_block(*blk_num as usize)?;
                        data.extend_from_slice(&block.data);
                        *blk_num = next_blk;
                    }
                    Some(&FatType::EOF) => {
                        let block: FileData = self.disk.read_block(*blk_num as usize)?;
                        data.extend_from_slice(&block.data);
                        break;
                    }
                    _ => return Err(FSError::InvalidBlockReference.into()),
                }
            }
            Ok(())
        };

        // Call the recursive read function
        read_blocks_recursively(&mut blk_num, &mut data.data)?;

        Ok(data)
    }

    #[trace_log]
    pub fn clear_file_data(&self, start_blk: u16) -> Result<()> {
        let mut blk_num = start_blk;
        let zero_data = vec![0u8; Disk::BLOCK_SIZE];

        // Recursive closure to clear blocks following the FAT
        let clear_blocks_recursively = |blk_num: &mut u16| -> Result<()> {
            loop {
                match self.fat.get(*blk_num as usize) {
                    Some(&FatType::Taken(next_blk)) => {
                        // Instead of reading, we write zeroes to the block
                        self.disk.write_raw_data(*blk_num as usize, &zero_data)?;
                        *blk_num = next_blk;
                    }
                    Some(&FatType::EOF) => {
                        // Clear the EOF block as well
                        self.disk.write_raw_data(*blk_num as usize, &zero_data)?;
                        break;
                    }
                    _ => return Err(FSError::InvalidBlockReference.into()),
                }
            }
            Ok(())
        };

        // Call the recursive clear function
        clear_blocks_recursively(&mut blk_num)?;

        Ok(())
    }

    #[trace_log]
    pub fn read_blk(&self, blk: u64) -> Result<Block> {
        let block: Block = self.disk.read_block(blk as usize)?;
        Ok(block)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_file_system_creation() {
        let fs = FileSystem::new().unwrap();
        assert_eq!(fs.curr_block.blk_num, 0);
        Disk::delete_disk().unwrap();
    }

    #[test]
    fn test_file_system_write_curr_blk() {
        let mut fs = FileSystem::new().unwrap();
        let entry = DirEntry {
            name: "test".into(),
            file_type: FileType::File,
            size: 0,
            blk_num: 0,
        };
        fs.curr_block.entries.push(entry.clone());
        //fs.curr_block.entries[0] = entry.clone();
        fs.write_curr_blk().unwrap();
        let read_block = fs.read_blk(0).unwrap();
        assert_eq!(read_block.entries[0].name, entry.name);
        Disk::delete_disk().unwrap();
    }
}
