#![allow(unused_variables)]

use std::fmt::Debug;

use anyhow::Result;
#[cfg(feature = "debug")]
use log::{debug, trace};
use serde::Serialize;

use file_data::FileData;
use logger_macro::trace_log;
use rustic_disk::traits::BlockStorage;
use rustic_disk::Disk;

use crate::dir_entry::{DirBlock, DirEntry, FileType};
use crate::errors::{FSError, IOHandlerError};
use crate::fat::{FatType, FAT};
use crate::prelude::{File, IOHandler};

mod dir_entry;
mod directories;
mod errors;
mod fat;
mod file_data;
mod files;
mod format;
mod other;
pub mod prelude;
#[cfg(feature = "py-bindings")]
mod py_bindings;
mod tests;
mod traits;
mod utils;

#[cfg(feature = "py-bindings")]
use pyo3::pyclass;
use std::io;

#[derive(Clone)]
#[cfg_attr(feature = "py-bindings", pyclass)]
pub struct StdIOHandler;

impl IOHandler for StdIOHandler {
    type Input = String;
    type Output = String;

    fn read(&mut self) -> Result<String> {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| IOHandlerError::IOError(e.to_string()).into()) // Convert to anyhow::Error
            .map(|_| input.trim_end().to_string())
    }

    fn write(&mut self, content: String) -> Result<()> {
        println!("{}", content);
        Ok(())
    }
}

impl Debug for StdIOHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StdIOHandler")
    }
}

const ROOT_BLK: u64 = 0;
const FAT_BLK: u64 = 1;

#[derive(Debug)]
pub struct FileSystem {
    disk: Disk,
    curr_block: DirBlock,
    fat: FAT,
    //#[cfg(not(target_arch = "wasm32"))]
    pub io_handler: Box<dyn IOHandler<Input = String, Output = String> + Send + Sync>,
}

impl Clone for FileSystem {
    fn clone(&self) -> Self {
        FileSystem {
            #[cfg(not(target_arch = "wasm32"))]
            disk: Disk::new().unwrap(),
            #[cfg(target_arch = "wasm32")]
            disk: self.disk.clone(),
            curr_block: self.curr_block.clone(),
            fat: self.fat.clone(),
            io_handler: self.io_handler.clone_box(),
        }
    }
}

const READ: u8 = 0x04;
const WRITE: u8 = 0x02;
const EXECUTE: u8 = 0x01;
const READ_WRITE: u8 = READ | WRITE;
const READ_EXECUTE: u8 = READ | EXECUTE;
const WRITE_EXECUTE: u8 = WRITE | EXECUTE;
const READ_WRITE_EXECUTE: u8 = READ | WRITE | EXECUTE;
const NONE: u8 = 0x00;

#[trace_log]
fn get_access_rights(access: u8) -> String {
    match access {
        READ_WRITE_EXECUTE => "rwx".to_string(),
        READ_WRITE => "rw-".to_string(),
        READ_EXECUTE => "r-x".to_string(),
        READ => "r--".to_string(),
        WRITE_EXECUTE => "-wx".to_string(),
        WRITE => "-w-".to_string(),
        EXECUTE => "--x".to_string(),
        NONE => "---".to_string(),
        _ => "???".to_string(),
    }
}

impl FileSystem {
    pub fn num_entries() -> usize {
        Disk::BLOCK_SIZE / DirEntry::calculate_max_size()
    }

    #[trace_log]
    pub fn new(
        io_handler: Box<dyn IOHandler<Input = String, Output = String> + Send + Sync>,
    ) -> Result<Self> {
        #[cfg(feature = "debug")]
        {
            debug!("Creating new file system");
            debug!("Max entries per block: {}", Self::num_entries());
        }
        let (curr_block, fat, disk) = if !Disk::disk_exists() {
            #[cfg(target_arch = "wasm32")]
            let mut disk = Disk::new()?;
            #[cfg(not(target_arch = "wasm32"))]
            let disk = Disk::new()?;
            let fat = FAT::new();
            let root_block = DirBlock {
                path: "/".to_string(),
                parent_entry: DirEntry {
                    name: "/".into(),
                    file_type: FileType::Directory,
                    access_level: READ_WRITE_EXECUTE,
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
            let mut root_block: DirBlock = disk.read_block(0)?;
            root_block.parent_entry.file_type = FileType::Directory;
            root_block.parent_entry.access_level = READ_WRITE_EXECUTE;
            root_block.parent_entry.name = "/".into();
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
            io_handler,
        })
    }

    #[trace_log]
    pub fn update_curr_dir(&mut self) -> Result<()> {
        self.curr_block = self.read_dir_block(&self.curr_block.parent_entry)?;
        Ok(())
    }

    #[trace_log]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn write_curr_blk(&self) -> Result<()> {
        let block_to_write = self.curr_block.blk_num;
        self.disk
            .write_block(block_to_write as usize, &self.curr_block)?;
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn write_curr_blk(&mut self) -> Result<()> {
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
    pub fn clear_file_data(&mut self, start_blk: u16) -> Result<()> {
        let mut blk_num = start_blk;
        let zero_data = vec![0u8; Disk::BLOCK_SIZE];

        // Recursive closure to clear blocks following the FAT
        let mut clear_blocks_recursively = |blk_num: &mut u16| -> Result<()> {
            loop {
                match self.fat.get(*blk_num as usize) {
                    Some(&FatType::Taken(next_blk)) => {
                        // Instead of reading, we write zeroes to the block
                        self.disk.write_raw_data(*blk_num as usize, &zero_data)?;

                        let lol: usize = blk_num.clone() as usize;
                        self.fat[lol] = FatType::Free;
                        self.disk.write_block(FAT_BLK as usize, &self.fat)?;
                        *blk_num = next_blk;
                    }
                    Some(&FatType::EOF) => {
                        // Clear the EOF block as well
                        self.disk.write_raw_data(*blk_num as usize, &zero_data)?;
                        let lol: usize = blk_num.clone() as usize;
                        self.fat[lol] = FatType::Free;
                        self.disk.write_block(FAT_BLK as usize, &self.fat)?;
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
    pub fn remove_dir_data(&mut self, dir_entry: &DirEntry, path: &str) -> Result<()> {
        let block: DirBlock = self.read_dir_block(dir_entry)?;

        for entry in &block.entries {
            if entry.name.is_empty() {
                continue;
            }

            let new_path = format!("{}/{}", path, entry.name);

            match entry.file_type {
                FileType::File => self.delete_file(&new_path)?,
                FileType::Directory => self.remove_dir_data(entry, &new_path)?,
            }
        }

        let zero_data = vec![0u8; Disk::BLOCK_SIZE];
        self.disk
            .write_raw_data(dir_entry.blk_num as usize, &zero_data)?;

        self.fat[dir_entry.blk_num as usize] = FatType::Free;
        self.disk.write_block(FAT_BLK as usize, &self.fat)?;
        Ok(())
    }

    #[trace_log]
    pub fn read_blk(&self, blk: u64) -> Result<DirBlock> {
        let block: DirBlock = self.disk.read_block(blk as usize)?;
        Ok(block)
    }
}
