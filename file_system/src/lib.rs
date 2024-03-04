#![allow(unused_variables)]

use std::fmt::Debug;

use anyhow::Result;
#[cfg(feature = "debug")]
use log::{debug, trace};
use serde::Serialize;

use file_data::FileData;
use logger_macro::trace_log;
use rustic_disk::Disk;
use rustic_disk::traits::BlockStorage;

use crate::dir_entry::{DirBlock, DirEntry, FileType};
use crate::errors::FSError;
use crate::fat::{FAT, FatType};

mod dir_entry;
mod directories;
mod errors;
mod fat;
mod file_data;
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
    curr_block: DirBlock,
    fat: FAT,
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
            let root_block = DirBlock {
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
            let root_block: DirBlock = disk.read_block(0)?;
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
    pub fn read_blk(&self, blk: u64) -> Result<DirBlock> {
        let block: DirBlock = self.disk.read_block(blk as usize)?;
        Ok(block)
    }
}
