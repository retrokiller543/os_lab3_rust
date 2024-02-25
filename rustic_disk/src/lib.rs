#![allow(non_snake_case)]
pub mod traits;
pub mod errors;

use std::fs;
use anyhow::Result;
use crate::traits::BlockStorage;
use std::fs::{OpenOptions, File};
use std::io::{self, Seek, SeekFrom, Read, Write};
use std::path::Path;
use serde::{Serialize, de::DeserializeOwned};
use bincode;
use log::{debug, error, trace};
use crate::errors::DiskError;

const DISKNAME: &str = "diskfile.bin";

pub struct Disk {
    diskfile: File,
}

impl Disk {
    pub const BLOCK_SIZE: usize = 4096; // Adjust as needed
    pub const DISK_SIZE: usize = Self::BLOCK_SIZE * Self::NUM_BLOCKS;
    pub const NUM_BLOCKS: usize = 2048;
    pub fn new() -> io::Result<Self> {
        if !Path::new(DISKNAME).exists() {
            #[cfg(feature = "debug")]
            {
                trace!("Creating disk with name {}", DISKNAME);
            }
            let file = File::create(DISKNAME)?;
            file.set_len(Self::DISK_SIZE as u64)?;
            #[cfg(feature = "debug")]
            {
                trace!("Disk with size {} created", Self::DISK_SIZE);
            }
        }
        let diskfile = OpenOptions::new()
            .read(true)
            .write(true)
            .open(DISKNAME)?;
        Ok(Disk { diskfile })
    }

    fn get_block_position(&self, block_index: usize) -> Result<u64, DiskError> {
        let position = block_index.checked_mul(Self::BLOCK_SIZE)
            .and_then(|p| Some(p as u64)) // Convert to u64, this step should not overflow given BLOCK_SIZE is usize
            .ok_or(DiskError::PositionOverflow); // Convert None to a DiskError

        #[cfg(feature = "debug")]
        {
            trace!("Block position: {:?}", position);
        }

        position
    }

    pub fn disk_exists() -> bool {
        #[cfg(feature = "debug")]
        {
            trace!("Checking if disk with name {} exists", DISKNAME);
        }
        Path::new(DISKNAME).exists()
    }

    pub fn delete_disk() -> io::Result<()> {
        #[cfg(feature = "debug")]
        {
            trace!("Deleting disk with name {}", DISKNAME);
        }
        fs::remove_file(DISKNAME)
    }
}

impl BlockStorage for Disk {
    /// Read data from the disk using the bincode deserialization
    fn read_block<T: DeserializeOwned + std::fmt::Debug>(&self, block_index: usize) -> Result<T, DiskError> {
        let mut file = &self.diskfile;
        let position = self.get_block_position(block_index)?;
        file.seek(SeekFrom::Start(position)).map_err(DiskError::SeekError)?;
        let mut buffer = vec![0u8; Self::BLOCK_SIZE];
        file.read_exact(&mut buffer).map_err(DiskError::ReadDiskError)?;
        let data = bincode::deserialize(&buffer).map_err(DiskError::DeserializationError)?;
        #[cfg(feature = "debug")]
        {
            trace!("data from the disk: {:?}", data);
        }
        Ok(data)
    }

    /// Write data to the disk using the bincode serialization
    fn write_block<T: Serialize>(&self, block_index: usize, data: &T) -> Result<(), DiskError> {
        let serialized_data = bincode::serialize(data).map_err(DiskError::SerializationError)?;
        if serialized_data.len() > Self::BLOCK_SIZE {
            error!("Data is {} bytes, which exceeds the block size of {}", serialized_data.len(), Self::BLOCK_SIZE);
            return Err(DiskError::DataExceedsBlockSize);
        }
        let mut file = &self.diskfile;
        let position = self.get_block_position(block_index)?;
        file.seek(SeekFrom::Start(position)).map_err(DiskError::SeekError)?;
        file.write_all(&serialized_data).map_err(DiskError::WriteDiskError)?;
        #[cfg(feature = "debug")]
        {
            debug!("{:?} bytes written to the disk", serialized_data.len());
        }
        Ok(())
    }

    /// Write raw data to the disk
    fn write_serilized_data(&self, block_index: usize, data: &[u8]) -> Result<(), DiskError> {
        if data.len() > Self::BLOCK_SIZE {
            error!("Data is {} bytes, which exceeds the block size of {}", data.len(), Self::BLOCK_SIZE);
            return Err(DiskError::DataExceedsBlockSize);
        }
        let mut file = &self.diskfile;
        let position = self.get_block_position(block_index)?;
        file.seek(SeekFrom::Start(position)).map_err(DiskError::SeekError)?;
        file.write_all(data).map_err(DiskError::WriteDiskError)?;
        Ok(())
    }

    /// Read raw data from the disk
    fn read_serilized_data(&self, block_index: usize) -> Result<Vec<u8>, DiskError> {
        let mut file = &self.diskfile;
        let position = self.get_block_position(block_index)?;
        file.seek(SeekFrom::Start(position)).map_err(DiskError::SeekError)?;
        let mut buffer = vec![0u8; Self::BLOCK_SIZE];
        file.read_exact(&mut buffer).map_err(DiskError::ReadDiskError)?;
        Ok(buffer)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use serde_derive::Deserialize;

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
        let disk = Disk::new().unwrap();
        disk.write_block(0, &data).unwrap();
        assert_eq!(data, disk.read_block::<TestData>(0).unwrap());
        //let _ = fs::remove_file(DISKNAME);
    }

    #[test]
    fn write_block_writes_correct_data() {
        let disk = Disk::new().unwrap();
        let write_result = disk.write_block(0, &"new data");
        assert!(write_result.is_ok());
        let read_result: Result<String, _> = disk.read_block(0);
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), "new data");
        let _ = fs::remove_file(DISKNAME);
    }

    #[test]
    fn write_block_returns_error_if_data_exceeds_block_size() {
        let disk = Disk::new().unwrap();
        let large_data = "a".repeat(Disk::BLOCK_SIZE + 1);
        let result = disk.write_block(0, &large_data);
        assert!(result.is_err());
        let _ = fs::remove_file(DISKNAME);
    }
}