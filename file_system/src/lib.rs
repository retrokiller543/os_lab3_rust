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
mod execute_py;

#[cfg(feature = "py-bindings")]
use pyo3::prelude::*;
use std::io;
use log::info;

/// The `StdIOHandler` struct is a standard input/output handler.
///
/// It implements the `IOHandler` trait, which requires an `Input`
/// associated type and an `Output` associated type, both of which are `String` in this case.
/// The `IOHandler` trait also requires the implementation of two methods: `read` and `write`.
/// The `read` method reads input from the user and returns a `Result<String>`.
/// The `write` method takes a `String` as input and writes it as output, returning a `Result<()>`.
///
/// The `StdIOHandler` struct also implements the `Debug` trait,
/// which requires the implementation of the `fmt` method.
/// The `fmt` method formats the struct for output, returning a `std::fmt::Result`.
#[derive(Clone)]
#[cfg_attr(feature = "py-bindings", pyclass)]
pub struct StdIOHandler;

impl IOHandler for StdIOHandler {
    type Input = String;
    type Output = String;

    /// Reads input from the user.
    ///
    /// This method reads a line from the standard input,
    /// trims the trailing newline character, and returns the input as a `Result<String>`.
    /// If an error occurs during reading,
    /// it is converted to an `IOHandlerError::IOError` and then to an `anyhow::Error`.
    fn read(&mut self) -> Result<String> {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| IOHandlerError::IOError(e.to_string()).into()) // Convert to anyhow::Error
            .map(|_| input.trim_end().to_string())
    }

    /// Writes output to the user.
    ///
    /// This method takes a `String` as input and writes it to the standard output,
    /// returning a `Result<()>`.
    fn write(&mut self, content: String) -> Result<()> {
        println!("{}", content);
        Ok(())
    }
}

impl Debug for StdIOHandler {
    /// Formats the `StdIOHandler` struct for output.
    ///
    /// This method writes "StdIOHandler" to the provided formatter and returns a `std::fmt::Result`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StdIOHandler")
    }
}

const ROOT_BLK: u64 = 0;
const FAT_BLK: u64 = 1;

/// The `FileSystem` struct represents a file system.
///
/// ## Fields
///
/// * `disk`: A `Disk` object representing the disk on which the file system is stored.
///
/// * `curr_block`: A `DirBlock`
/// object representing the current directory block that the file system is interacting with.
/// * `fat`: A `FAT` object representing the File Allocation Table of the file system.
///
/// * `io_handler`: A boxed dynamic `IOHandler` trait object.
/// This is used for handling input and output operations in the file system.
///   The `IOHandler` trait requires an `Input` associated type and an `Output` associated type,
/// both of which are `String` in this case.
///   The `IOHandler` trait also requires the implementation of two methods: `read` and `write`.
///   The `read` method reads input from the user and returns a `Result<String>`.
///   The `write` method takes a `String` as input and writes it as output, returning a `Result<()>`.
///   The `IOHandler` trait object is also required to be both `Send`
/// and `Sync`, allowing it to be safely shared across threads.
///
/// ## Example with `StdIOHandler`
///
/// The following example demonstrates how to create a new `FileSystem`
/// object using the `StdIOHandler` as the `IOHandler` trait object.
///
/// ```rust
/// # use anyhow::Result;
/// # use file_system::prelude::*;
/// # fn main() -> Result<()> {
/// let mut fs = FileSystem::new(Box::new(StdIOHandler))?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
#[cfg_attr(feature = "py-bindings", pyclass)]
pub struct FileSystem {
    disk: Disk,
    curr_block: DirBlock,
    fat: FAT,
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

    /// Creates a new `FileSystem` object.
    ///
    /// This method takes a boxed dynamic `IOHandler` trait object as input and returns a `Result<Self>`.
    /// The `IOHandler` trait requires an `Input` associated type and an `Output` associated type,
    /// both of which are `String` in this case.
    /// The `IOHandler` trait also requires the implementation of two methods: `read` and `write`.
    /// The `read` method reads input from the user and returns a `Result<String>`.
    /// The `write` method takes a `String` as input and writes it as output, returning a `Result<()>`.
    /// The `IOHandler` trait object is also required to be both `Send`
    /// and `Sync`, allowing it to be safely shared across threads.
    ///
    ///
    /// # Arguments
    ///
    /// * `io_handler: Box<dyn IOHandler<Input = String, Output = String> + Send + Sync>`
    ///     - A boxed dynamic `IOHandler` trait object.
    ///
    /// # Returns
    ///
    /// A `Result<Self>` containing the new `FileSystem` object.
    ///
    /// The `new` method first checks if a disk exists.
    /// If not, it creates a new disk, a new `FAT` object, and a new root directory block.
    /// It then writes the root directory block and the `FAT` object to the disk.
    /// If a disk does exist, it reads the root directory block and the `FAT` object from the disk.
    /// Finally, it creates a new `FileSystem` object with the disk,
    /// the root directory block, the `FAT` object, and the `IOHandler` trait object,
    /// and returns it.
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

    /// Updates the current directory block of the file system.
    ///
    /// This method reads the directory block of the parent entry of the current directory block
    /// and sets it as the new current directory block.
    /// It returns a `Result<()>` indicating the success or failure of the operation.
    #[trace_log]
    pub fn update_curr_dir(&mut self) -> Result<()> {
        self.curr_block = self.read_dir_block(&self.curr_block.parent_entry)?;
        Ok(())
    }

    /// Writes the current directory block to the disk.
    ///
    /// This method is only available when the target architecture is not `wasm32`.
    /// It writes the current directory block to the disk at the block number of the current directory block.
    /// It returns a `Result<()>` indicating the success or failure of the operation.
    #[trace_log]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn write_curr_blk(&self) -> Result<()> {
        let block_to_write = self.curr_block.blk_num;
        self.disk
            .write_block(block_to_write as usize, &self.curr_block)?;
        Ok(())
    }

    /// Writes the current directory block to the disk.
    ///
    /// This method is only available when the target architecture is `wasm32`.
    /// It writes the current directory block to the disk at the block number of the current directory block.
    /// It returns a `Result<()>` indicating the success or failure of the operation.
    #[cfg(target_arch = "wasm32")]
    pub fn write_curr_blk(&mut self) -> Result<()> {
        let block_to_write = self.curr_block.blk_num;
        self.disk
            .write_block(block_to_write as usize, &self.curr_block)?;
        Ok(())
    }

    /// Returns the block number of the first free block in the file allocation table (FAT).
    ///
    /// This method iterates over the FAT
    /// and returns the block number of the first free block it encounters.
    /// If no free blocks are found, it returns an `FSError::NoFreeBlocks` error.
    ///
    /// # Errors
    /// Returns `FSError::NoFreeBlocks` if no free blocks are found in the FAT.
    #[trace_log]
    pub fn get_free_block(&mut self) -> Result<u16> {
        let mut blk = 0;

        for (index, block) in self.fat.iter().enumerate() {
            match block {
                FatType::Free => {
                    blk = index as u16;
                    self.fat[index] = FatType::Taken(0);
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

    /// Writes the serialized form of the given data to the disk, starting at the specified block.
    ///
    /// This method first serializes the given data.
    /// If the serialized data fits within a single block, it writes it directly to the disk.
    /// If the serialized data is larger than a single block,
    /// it splits the data into chunks and writes each chunk to a separate block.
    /// It updates the FAT for each block it writes to.
    /// If a block is the last block of the data, it updates the FAT for that block to EOF.
    /// If a block is not the last block,
    /// it gets a new free block and updates the FAT for the current block to point to the new block.
    ///
    /// # Arguments
    /// * `data: &T where T: Serialize + Debug` -
    /// The data to write to the disk. This data is serialized before being written.
    /// * `start_blk: u16` - The block number to start writing the data at.
    ///
    /// # Errors
    /// Returns `FSError::SerializationError` if an error occurs during serialization.
    /// Returns `FSError::NoFreeBlocks` if no free blocks are found in the FAT when needed.
    //#[trace_log]
    pub fn write_data<T: Serialize + Debug>(&mut self, data: &T, start_blk: u16) -> Result<()> {
        // Serialize the data
        let serialized_data = bincode::serialize(data).map_err(FSError::SerializationError)?;

        // If the data fits within a single block, write it directly
        if serialized_data.len() <= Disk::BLOCK_SIZE {
            self.disk.write_raw_data(start_blk as usize, &serialized_data)?;
            // Directly update FAT for start_blk to EOF
            self.set_fat_block(start_blk, FatType::EOF)?;
            // write the updated FAT to the disk
            self.disk.write_block(FAT_BLK as usize, &self.fat)?;
            return Ok(());
        }

        // For larger data, split into chunks
        let mut chunks = serialized_data.chunks(Disk::BLOCK_SIZE).peekable();
        let mut blk = start_blk;

        while let Some(chunk) = chunks.next() {
            // Write the current chunk
            self.disk.write_raw_data(blk as usize, chunk)?;

            if chunks.peek().is_some() {
                // If there's more data, get a new block and update the FAT to link to it
                let new_blk = self.get_free_block()?;
                self.set_fat_block(blk, FatType::Taken(new_blk))?;
                blk = new_blk; // Update blk to the new block for the next iteration
            } else {
                // If it's the last chunk, update the FAT to EOF
                self.set_fat_block(blk, FatType::EOF)?;
            }
        }

        // write the updated FAT to the disk
        self.disk.write_block(FAT_BLK as usize, &self.fat)?;

        Ok(())
    }


    pub fn set_fat_block(&mut self, blk: u16, new_val: FatType) -> Result<()> {
        self.fat[blk as usize] = new_val;
        Ok(())
    }

    /// Updates the File Allocation Table (FAT) for a given block.
    ///
    /// This method takes a block number and an optional next block number as input.
    /// If the next block number is `Some`,
    /// it updates the FAT entry for the given block to `Taken(next_blk)`.
    /// If the next block number is `None`, it updates the FAT entry for the given block to `EOF`.
    /// It then writes the updated FAT to the disk.
    ///
    /// # Arguments
    /// * `blk: u16` - The block number to update in the FAT.
    /// * `next_blk: Option<u16>` - The optional next block number.
    /// If `Some`, the FAT entry for `blk` is updated to `Taken(next_blk)`.
    /// If `None`, the FAT entry for `blk` is updated to `EOF`.
    ///
    /// # Errors
    /// Returns an error if writing the updated FAT to the disk fails.
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

    /// Reads all blocks of a file in order following the File Allocation Table (FAT).
    ///
    /// This method takes a starting block number as input and returns a `Result<FileData>`.
    /// It creates a new `FileData` object and sets the starting block number.
    /// It then defines a recursive closure to read blocks following the FAT.
    /// The closure loops over the FAT, starting at the given block number.
    /// For each block, it reads the block data and appends it to the `FileData` object.
    /// If the FAT entry for a block is `Taken(next_blk)`, it updates the block number to `next_blk`
    /// and continues the loop.
    /// If the FAT entry for a block is `EOF`, it reads the block data,
    /// appends it to the `FileData` object, and breaks the loop.
    /// If the FAT entry for a block is neither `Taken` nor `EOF`,
    /// it returns an `FSError::InvalidBlockReference` error.
    /// After defining the closure,
    /// it calls the closure with the starting block number and the data of the `FileData` object.
    /// Finally, it returns the `FileData` object.
    ///
    /// # Arguments
    /// * `start_blk: u16` - The block number to start reading the file data at.
    ///
    /// # Errors
    /// Returns `FSError::InvalidBlockReference`
    /// if a block reference in the FAT is neither `Taken` nor `EOF`.
    /// Returns an error if reading a block from the disk fails.
    #[trace_log]
    pub fn read_file_data(&self, start_blk: u16) -> Result<FileData> {
        let mut data = Vec::new();
        let mut blk_num = start_blk;

        // Recursive closure to read blocks following the FAT
        let read_blocks_recursively = |blk_num: &mut u16, data: &mut Vec<u8>| -> Result<()> {
            loop {
                match self.fat.get(*blk_num as usize) {
                    Some(&FatType::Taken(next_blk)) => {
                        info!("Reading block (Taken({})): {}", next_blk, blk_num);
                        let block = self.disk.read_raw_data(*blk_num as usize)?;
                        data.extend_from_slice(&block);
                        *blk_num = next_blk;
                    }
                    Some(&FatType::EOF) => {
                        info!("Reading block (EOF): {}", blk_num);
                        let block = self.disk.read_raw_data(*blk_num as usize)?;
                        data.extend_from_slice(&block);
                        break;
                    }
                    _ => return Err(FSError::InvalidBlockReference.into()),
                }
            }
            Ok(())
        };

        // Call the recursive read function
        read_blocks_recursively(&mut blk_num, &mut data)?;
        // deserialize the data into file data
        let file_data: FileData = bincode::deserialize(&data)?;

        Ok(file_data)
    }

    /// Clears the file data starting from a specified block.
    ///
    /// This method takes a starting block number as input
    /// and clears all the blocks of the file following the File Allocation Table
    /// (FAT).
    /// It creates a vector of zeroes with the size of a block
    /// and defines a recursive closure to clear blocks following the FAT.
    /// The closure loops over the FAT, starting at the given block number.
    /// For each block, it writes zeroes to the block and updates the FAT entry for the block to `Free`.
    /// If the FAT entry for a block is `Taken(next_blk)`, it updates the block number to `next_blk`
    /// and continues the loop.
    /// If the FAT entry for a block is `EOF`, it writes zeroes to the block,
    /// updates the FAT entry for the block to `Free`, and breaks the loop.
    /// If the FAT entry for a block is neither `Taken` nor `EOF`,
    /// it returns an `FSError::InvalidBlockReference` error.
    /// After defining the closure, it calls the closure with the starting block number.
    ///
    /// # Arguments
    /// * `start_blk: u16` - The block number to start clearing the file data at.
    ///
    /// # Errors
    /// Returns `FSError::InvalidBlockReference`
    /// if a block reference in the FAT is neither `Taken` nor `EOF`.
    /// Returns an error if writing zeroes to a block or updating the FAT fails.
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

    /// Removes the directory data for a given directory entry.
    ///
    /// This method takes a directory entry and a path as input
    /// and removes all the entries in the directory following the File Allocation Table
    /// (FAT).
    /// It reads the directory block of the given directory entry and iterates over all the entries in the directory block.
    /// For each entry, if the entry is a file, it deletes the file.
    /// If the entry is a directory, it recursively removes the directory data for the entry.
    /// After iterating over all the entries,
    /// it writes zeroes to the block of the given directory entry
    /// and updates the FAT entry for the block to `Free`.
    ///
    /// # Arguments
    /// * `dir_entry: &DirEntry` - The directory entry to remove the directory data for.
    /// * `path: &str` - The path of the directory.
    ///
    /// # Errors
    /// Returns an error if deleting a file, removing directory data,
    /// reading a directory block, writing zeroes to a block, or updating the FAT fails.
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

    /// Reads a directory block from the disk.
    ///
    /// This method takes a block number as input
    /// and reads the directory block from the disk at the given block number.
    /// It returns a `Result<DirBlock>` containing the read directory block.
    ///
    /// # Arguments
    /// * `blk: u64` - The block number to read the directory block from.
    ///
    /// # Errors
    /// Returns an error if reading the directory block from the disk fails.
    #[trace_log]
    pub fn read_blk(&self, blk: u64) -> Result<DirBlock> {
        let block: DirBlock = self.disk.read_block(blk as usize)?;
        Ok(block)
    }
}
