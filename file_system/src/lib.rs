mod utils;
pub mod traits;
mod dir_entry;
mod format;
mod errors;
mod files;
pub mod prelude;
mod directories;

use rustic_disk::Disk;
use rustic_disk::traits::BlockStorage;
use anyhow::Result;
use serde::Serialize;
use serde_derive::Deserialize;
use crate::dir_entry::{Block, DirEntry, FileType};
use crate::errors::FSError;
use crate::files::FileData;

const ROOT_BLK: u64 = 0;
const FAT_BLK: u64 = 1;

pub struct FileSystem {
    disk: Disk,
    curr_block: Block,
    fat: Vec<FAT> // this is the amount of blocks in the disk
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum FAT {
    Free,
    Taken(u64),
    EOF,
}

impl FileSystem {
    pub fn new() -> Result<Self> {
        let disk = Disk::new()?;

        let (curr_block, fat) = if !Disk::disk_exists() {
            let fat = vec![FAT::Free; Disk::BLOCK_SIZE / std::mem::size_of::<Block>()];
            let root_block = Block {
                parent_entry: DirEntry {
                    name: "/".to_string(),
                    file_type: FileType::Directory,
                    ..Default::default()
                },
                blk_num: 0,
                entries: vec![DirEntry::default(); 64],
            };
            disk.write_block(0, &root_block)?;
            disk.write_block(1, &fat)?;
            (root_block, fat)
        } else {
            let root_block: Block = disk.read_block(0)?;
            let fat: Vec<FAT> = disk.read_block(1)?;
            (root_block, fat)
        };


        Ok(FileSystem { disk, curr_block, fat })
    }

    pub fn write_curr_blk(&self) -> Result<()> {
        let block_to_write = self.curr_block.blk_num;
        self.disk.write_block(block_to_write as usize, &self.curr_block)?;
        Ok(())
    }

    pub fn get_free_block(&self) -> Result<usize> {
        let mut blk = 0;

        for (index, block) in self.fat.iter().enumerate() {
            match block {
                FAT::Free => {
                    blk = index as u64;
                    break;
                },
                _ => continue,
            }
        }

        if blk == 0 {
            return Err(FSError::NoFreeBlocks.into());
        }

        Ok(blk as usize)
    }

    pub fn write_data<T: Serialize>(&mut self, data: &T, start_blk: u64) -> Result<()> {
        // Serialize the data
        let serialized_data = bincode::serialize(data).map_err(FSError::SerializationError)?;

        // If the data fits within a single block, write it directly
        if serialized_data.len() <= Disk::BLOCK_SIZE {
            self.disk.write_serilized_data(start_blk as usize, &serialized_data)?;
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
                blk = self.get_free_block()? as u64; // Get a new block if not the first iteration
            } else {
                first_iteration = false;
            }
            self.disk.write_serilized_data(blk as usize, chunk)?;
            let next_blk = if chunks.peek().is_some() {
                Some(self.get_free_block()? as u64)
            } else {
                None
            };

            // Update FAT for blk. If next_blk is None, it's the last chunk
            self.update_fat(blk, next_blk)?;
        }

        Ok(())
    }

    pub fn update_fat(&mut self, blk: u64, next_blk: Option<u64>) -> Result<()> {
        match next_blk {
            Some(next_blk) => {
                self.fat[blk as usize] = FAT::Taken(next_blk);
            },
            None => {
                self.fat[blk as usize] = FAT::EOF;
            },
        }
        self.disk.write_block(FAT_BLK as usize, &self.fat)?;
        Ok(())
    }

    // Method to read all blocks of a file in order following the FAT table
    pub fn read_file_data(&self, start_blk: u64) -> Result<FileData> {
        let mut data = FileData::default();
        let mut blk_num = start_blk;

        // Recursive closure to read blocks following the FAT
        let mut read_blocks_recursively = |blk_num: &mut u64, data: &mut Vec<u8>| -> Result<()> {
            loop {
                match self.fat.get(*blk_num as usize) {
                    Some(FAT::Taken(next_blk)) => {
                        let block: FileData = self.disk.read_block(*blk_num as usize)?;
                        data.extend_from_slice(&block.data);
                        *blk_num = *next_blk;
                    },
                    Some(FAT::EOF) => {
                        let block: FileData = self.disk.read_block(*blk_num as usize)?;
                        data.extend_from_slice(&block.data);
                        break;
                    },
                    _ => return Err(FSError::InvalidBlockReference.into()),
                }
            }
            Ok(())
        };

        // Call the recursive read function
        read_blocks_recursively(&mut blk_num, &mut data.data)?;

        Ok(data)
    }

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
            name: "test".to_string(),
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