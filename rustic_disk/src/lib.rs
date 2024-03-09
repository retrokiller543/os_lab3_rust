#![allow(non_snake_case)]
#![allow(unused_variables)]
pub mod errors;
pub mod traits;

use crate::errors::DiskError;
use crate::traits::BlockStorage;
use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use log::error;
#[cfg(feature = "debug")]
use log::{debug, trace};
#[cfg(feature = "py-bindings")]
use pyo3::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{Read, Seek, SeekFrom, Write};

// Required imports for WASM
//#[cfg(target_arch = "wasm32")]
//use wasm_bindgen::prelude::*;

use std::io;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

use core::fmt::Debug;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::{File, OpenOptions};
use logger_macro::trace_log;
#[cfg(target_arch = "wasm32")]
use core::fmt::Debug;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex};

/// Name of the disk file on the filesystem.
#[cfg(not(target_arch = "wasm32"))]
const DISKNAME: &str = "diskfile.bin";

/// Represents a virtual disk with operations for reading and writing.
///
/// This struct encapsulates operations for interacting with a disk file, including
/// creating a new disk, reading and writing to disk blocks, and deleting the disk file.
/// It is designed to simulate block-level operations on a virtual disk file.
#[cfg_attr(feature = "py-bindings", pyclass)]
#[derive(Debug, Clone)]
pub struct Disk {
    /// The file handle for the disk file.
    #[cfg(not(target_arch = "wasm32"))]
    diskfile: Arc<Mutex<File>>,
    #[cfg(target_arch = "wasm32")]
    storage: Vec<u8>,
}

impl Disk {
    /// The size of each block on the disk in bytes.
    pub const BLOCK_SIZE: usize = 4096; // Adjust as needed

    /// The total number of blocks on the disk.
    pub const NUM_BLOCKS: usize = 2048;

    /// The total size of the disk in bytes (calculated from BLOCK_SIZE and NUM_BLOCKS).
    pub const DISK_SIZE: usize = Self::BLOCK_SIZE * Self::NUM_BLOCKS; // 8MB

    /// Creates a new Disk instance, initializing the disk file if it does not exist.
    ///
    /// This method checks for the existence of the disk file, creating it and setting its
    /// size to `DISK_SIZE` if it does not exist. If the file already exists, it simply opens
    /// the file for reading and writing.
    ///
    /// Returns:
    /// - `Ok(Self)`: A new instance of `Disk`.
    /// - `Err(e)`: An error if the file cannot be created or opened.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Result<Self, io::Error> {
        if !Path::new(DISKNAME).exists() {
            let file = File::create(DISKNAME)?;
            file.set_len(Self::DISK_SIZE as u64)?;
        }
        let diskfile = OpenOptions::new().read(true).write(true).open(DISKNAME)?;
        Ok(Disk {
            diskfile: Arc::new(Mutex::new(diskfile)),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new() -> anyhow::Result<Self> {
        let storage = vec![0; Self::DISK_SIZE];
        Ok(Disk { storage })
    }

    /// Calculates the file position for a given block index.
    ///
    /// This method computes the byte position in the file for the start of a specified
    /// block index, facilitating direct access to any block on the disk.
    ///
    /// Parameters:
    /// - `block_index`: The index of the block whose position is to be calculated.
    ///
    /// Returns:
    /// - `Ok(u64)`: The byte position of the start of the specified block.
    /// - `Err(DiskError)`: An error if the calculation results in an overflow.
    fn get_block_position(&self, block_index: usize) -> Result<u64, DiskError> {
        #[cfg(feature = "debug")]
        {
            let position = block_index
                .checked_mul(Self::BLOCK_SIZE)
                .map(|x| x as u64)
                .ok_or(DiskError::PositionOverflow);
            trace!("Block position: {:?}", position);
            position
        }
        #[cfg(not(feature = "debug"))]
        {
            block_index
                .checked_mul(Self::BLOCK_SIZE)
                .map(|x| x as u64)
                .ok_or(DiskError::PositionOverflow)
        }
    }

    /// Checks if the disk file exists on the filesystem.
    ///
    /// Returns:
    /// - `true`: If the disk file exists.
    /// - `false`: Otherwise.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn disk_exists() -> bool {
        #[cfg(feature = "debug")]
        {
            trace!("Checking if disk with name {} exists", DISKNAME);
        }
        Path::new(DISKNAME).exists()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn disk_exists() -> bool {
        false // disk will always exist in WASM since it's in memory
    }

    /// Deletes the disk file from the filesystem.
    ///
    /// This method removes the disk file, effectively deleting the virtual disk.
    ///
    /// Returns:
    /// - `Ok(())`: If the file was successfully deleted.
    /// - `Err(e)`: An error if the file cannot be deleted.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn delete_disk(&mut self) -> io::Result<()> {
        #[cfg(feature = "debug")]
        {
            trace!("Deleting disk with name {}", DISKNAME);
        }
        fs::remove_file(DISKNAME)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn delete_disk(&mut self) -> anyhow::Result<()> {
        self.storage = vec![0; Self::DISK_SIZE];
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl BlockStorage for Disk {
    /// Reads a block from the disk and deserializes it into the specified type `T`.
    ///
    /// This method seeks to the specified block index, reads a block of data, and then
    /// deserializes the data into an instance of type `T` using bincode.
    ///
    /// Parameters:
    /// - `block_index`: The index of the block to read from the disk.
    ///
    /// Returns:
    /// - `Ok(T)`: The deserialized data from the disk block.
    /// - `Err(DiskError)`: An error if seeking, reading, or deserialization fails.
    ///
    /// Note: The generic type `T` must implement `DeserializeOwned` and `Debug`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rustic_disk::Disk;
    /// # use anyhow::Result;
    /// # use serde::{Serialize, Deserialize};
    /// #[derive(PartialEq, Debug, Serialize, Deserialize)]
    /// struct TestData {
    ///    num: u32,
    ///    data: String,
    ///    vec: Vec<String>,
    /// }
    /// # fn main() -> Result<()> {
    /// # use rustic_disk::traits::BlockStorage;
    /// let mut disk = Disk::new()?;
    /// let data = TestData {
    ///    num: 42069,
    ///    data: "test data".to_string(),
    ///    vec: vec!["a".to_string(), "b".to_string()],
    /// };
    /// disk.write_block(0, &data)?;
    /// let read_data = disk.read_block::<TestData>(0)?;
    /// assert_eq!(data, read_data);
    /// # disk.delete_disk()?;
    /// # Ok(())
    /// # }
    /// ```
    #[trace_log]
    fn read_block<T: DeserializeOwned + std::fmt::Debug>(
        &self,
        block_index: usize,
    ) -> Result<T, DiskError> {
        let file = &self.diskfile;
        let position = self.get_block_position(block_index)?;
        let mut file_lock = file
            .lock()
            .map_err(|e| DiskError::FileLockError(e.into()))?;
        file_lock
            .seek(SeekFrom::Start(position))
            .map_err(DiskError::SeekError)?;
        let mut buffer = vec![0u8; Self::BLOCK_SIZE];
        file_lock
            .read_exact(&mut buffer)
            .map_err(DiskError::ReadDiskError)?;
        let data = bincode::deserialize(&buffer).map_err(DiskError::DeserializationError)?;
        #[cfg(feature = "debug")]
        {
            trace!("data from the disk: {:?}", data);
        }
        Ok(data)
    }

    /// Writes serialized data of type `T` to a specified block on the disk.
    ///
    /// This method serializes the given data using bincode and writes it to the disk at
    /// the specified block index. It checks to ensure the serialized data does not exceed
    /// the block size limit.
    ///
    /// Parameters:
    /// - `block_index`: The index of the block where the data will be written.
    /// - `data`: The data to serialize and write to the disk.
    ///
    /// Returns:
    /// - `Ok(())`: If the data is successfully written to the disk.
    /// - `Err(DiskError)`: An error if serialization, seeking, or writing fails.
    ///
    /// Note: The generic type `T` must implement `Serialize`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rustic_disk::Disk;
    /// # use anyhow::Result;
    /// # use serde::{Serialize, Deserialize};
    /// #[derive(PartialEq, Debug, Serialize, Deserialize)]
    /// struct TestData {
    ///    num: u32,
    ///    data: String,
    ///    vec: Vec<String>,
    /// }
    /// # fn main() -> Result<()> {
    /// # use rustic_disk::traits::BlockStorage;
    /// let mut disk = Disk::new()?;
    /// let data = TestData {
    ///    num: 42069,
    ///    data: "test data".to_string(),
    ///    vec: vec!["a".to_string(), "b".to_string()],
    /// };
    /// disk.write_block(0, &data)?;
    /// let read_data = disk.read_block::<TestData>(0)?; // Read the data back
    /// assert_eq!(data, read_data);
    /// # disk.delete_disk()?;
    /// # Ok(())
    /// # }
    /// ```
    #[trace_log]
    fn write_block<T: Serialize + Debug>(&self, block_index: usize, data: &T) -> Result<(), DiskError> {
        let serialized_data = bincode::serialize(data).map_err(DiskError::SerializationError)?;
        if serialized_data.len() > Self::BLOCK_SIZE {
            error!(
                "Data is {} bytes, which exceeds the block size of {}",
                serialized_data.len(),
                Self::BLOCK_SIZE
            );
            return Err(DiskError::DataExceedsBlockSize);
        }
        let file = &self.diskfile;
        let position = self.get_block_position(block_index)?;
        let mut file = file
            .lock()
            .map_err(|e| DiskError::FileLockError(e.into()))?;
        file.seek(SeekFrom::Start(position))
            .map_err(DiskError::SeekError)?;
        file.write_all(&serialized_data)
            .map_err(DiskError::WriteDiskError)?;
        #[cfg(feature = "debug")]
        {
            debug!("{:?} bytes written to the disk", serialized_data.len());
        }
        Ok(())
    }

    /// Writes raw data to a specified block on the disk.
    ///
    /// This method writes the provided raw data to the disk at the specified block index,
    /// ensuring the data does not exceed the block size limit.
    ///
    /// Parameters:
    /// - `block_index`: The index of the block where the data will be written.
    /// - `data`: The raw data to write to the disk.
    ///
    /// Returns:
    /// - `Ok(())`: If the data is successfully written to the disk.
    /// - `Err(DiskError)`: An error if the data exceeds the block size or if seeking or
    ///   writing fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rustic_disk::Disk;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # use rustic_disk::traits::BlockStorage;
    /// let mut disk = Disk::new()?;
    /// disk.write_raw_data(0, &[1, 2, 3, 4])?;
    /// let data = disk.read_raw_data(0)?; // Read the raw data back
    /// # let mut expected = vec![0; Disk::BLOCK_SIZE];
    /// # expected[..4].copy_from_slice(&[1, 2, 3, 4]);
    /// assert_eq!(data, expected);
    /// # disk.delete_disk()?;
    /// # Ok(())
    /// # }
    /// ```
    #[trace_log]
    fn write_raw_data(&self, block_index: usize, data: &[u8]) -> Result<(), DiskError> {
        if data.len() > Self::BLOCK_SIZE {
            error!(
                "Data is {} bytes, which exceeds the block size of {}",
                data.len(),
                Self::BLOCK_SIZE
            );
            return Err(DiskError::DataExceedsBlockSize);
        }
        let file = &self.diskfile;
        let position = self.get_block_position(block_index)?;
        let mut file = file
            .lock()
            .map_err(|e| DiskError::FileLockError(e.into()))?;
        file.seek(SeekFrom::Start(position))
            .map_err(DiskError::SeekError)?;
        file.write_all(data).map_err(DiskError::WriteDiskError)?;
        Ok(())
    }

    /// Reads raw data from a specified block on the disk.
    ///
    /// This method reads a block of raw data from the disk at the specified block index.
    /// It returns the raw data as a vector of bytes.
    ///
    /// Parameters:
    /// - `block_index`: The index of the block to read from the disk.
    ///
    /// Returns:
    /// - `Ok(Vec<u8>)`: The raw data read from the disk block.
    /// - `Err(DiskError)`: An error if seeking or reading fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rustic_disk::Disk;
    /// # use anyhow::Result;
    /// # fn main() -> Result<()> {
    /// # use rustic_disk::traits::BlockStorage;
    /// let mut disk = Disk::new()?;
    /// disk.write_raw_data(0, &[1, 2, 3, 4])?;
    /// let data = disk.read_raw_data(0)?;
    /// let mut expected = vec![0; Disk::BLOCK_SIZE];
    /// expected[..4].copy_from_slice(&[1, 2, 3, 4]);
    /// assert_eq!(data, expected);
    /// # disk.delete_disk()?;
    /// # Ok(())
    /// # }
    /// ```
    #[trace_log]
    fn read_raw_data(&self, block_index: usize) -> Result<Vec<u8>, DiskError> {
        let file = &self.diskfile;
        let position = self.get_block_position(block_index)?;
        let mut file = file
            .lock()
            .map_err(|e| DiskError::FileLockError(e.into()))?;
        file.seek(SeekFrom::Start(position))
            .map_err(DiskError::SeekError)?;
        let mut buffer = vec![0u8; Self::BLOCK_SIZE];
        file.read_exact(&mut buffer)
            .map_err(DiskError::ReadDiskError)?;
        Ok(buffer)
    }
}

#[cfg(target_arch = "wasm32")]
impl BlockStorage for Disk {
    #[trace_log]
    fn read_block<T: DeserializeOwned + Debug>(&self, block_index: usize) -> Result<T, DiskError> {
        let position = self.get_block_position(block_index)? as usize;
        let end = position + Self::BLOCK_SIZE;
        if end > self.storage.len() {
            return Err(DiskError::ReadDiskError(io::Error::new(
                io::ErrorKind::Other,
                "Read error",
            )));
        }
        let data_slice = &self.storage[position..end];
        bincode::deserialize(data_slice).map_err(DiskError::DeserializationError)
    }

    #[trace_log]
    fn write_block<T: Serialize + Debug>(&mut self, block_index: usize, data: &T) -> Result<(), DiskError> {
        let serialized_data = bincode::serialize(data).map_err(DiskError::SerializationError)?;
        if serialized_data.len() > Self::BLOCK_SIZE {
            return Err(DiskError::DataExceedsBlockSize);
        }
        let position = self.get_block_position(block_index)? as usize;
        let end = position + serialized_data.len();
        if end > self.storage.len() {
            return Err(DiskError::WriteDiskError(io::Error::new(
                io::ErrorKind::Other,
                "Write error",
            )));
        }
        self.storage[position..end].copy_from_slice(&serialized_data);
        Ok(())
    }

    #[trace_log]
    fn write_raw_data(&mut self, block_index: usize, data: &[u8]) -> Result<(), DiskError> {
        if data.len() > Self::BLOCK_SIZE {
            return Err(DiskError::DataExceedsBlockSize);
        }
        let position = self.get_block_position(block_index)? as usize;
        let end = position + data.len();
        if end > self.storage.len() {
            return Err(DiskError::WriteDiskError(io::Error::new(
                io::ErrorKind::Other,
                "Write error",
            )));
        }
        self.storage[position..end].copy_from_slice(&data);
        Ok(())
    }

    #[trace_log]
    fn read_raw_data(&self, block_index: usize) -> Result<Vec<u8>, DiskError> {
        let position = self.get_block_position(block_index)? as usize;
        let end = position + Self::BLOCK_SIZE;
        if end > self.storage.len() {
            return Err(DiskError::ReadDiskError(io::Error::new(
                io::ErrorKind::Other,
                "Read error",
            )));
        }
        Ok(self.storage[position..end].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_derive::Deserialize;
    use std::fs;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestData {
        num: u32,
        data: String,
        vec: Vec<String>,
    }

    fn setup_data() -> TestData {
        TestData {
            num: 42069,
            data: "test data".to_string(),
            vec: vec!["a".to_string(), "b".to_string()],
        }
    }

    #[test]
    fn disk_creation_creates_new_file_if_not_exists() {
        let _ = fs::remove_file(DISKNAME);
        assert!(!Path::new(DISKNAME).exists());
        let _ = Disk::new().unwrap();
        assert!(Path::new(DISKNAME).exists());
        let _ = fs::remove_file(DISKNAME);
    }

    #[test]
    fn disk_creation_does_not_overwrite_existing_file() {
        let _ = fs::remove_file(DISKNAME);
        let data = setup_data();
        let mut disk = Disk::new().unwrap();
        disk.write_block(0, &data).unwrap();
        assert_eq!(data, disk.read_block::<TestData>(0).unwrap());
        //let _ = fs::remove_file(DISKNAME);
    }

    #[test]
    fn write_block_writes_correct_data() {
        let mut disk = Disk::new().unwrap();
        let write_result = disk.write_block(0, &"new data");
        assert!(write_result.is_ok());
        let read_result: Result<String, _> = disk.read_block(0);
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), "new data");
        let _ = fs::remove_file(DISKNAME);
    }

    #[test]
    fn write_block_returns_error_if_data_exceeds_block_size() {
        let mut disk = Disk::new().unwrap();
        let large_data = "a".repeat(Disk::BLOCK_SIZE + 1);
        let result = disk.write_block(0, &large_data);
        assert!(result.is_err());
        let _ = fs::remove_file(DISKNAME);
    }
}
