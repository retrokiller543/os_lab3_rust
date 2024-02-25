use serde::{Serialize, de::DeserializeOwned};
use anyhow::Result;
use crate::errors::DiskError;

pub trait BlockStorage {
    fn read_block<T: DeserializeOwned + std::fmt::Debug>(&self, block_index: usize) -> Result<T, DiskError>;
    fn write_block<T: Serialize>(&self, block_index: usize, data: &T) -> Result<(), DiskError>;
    fn write_serilized_data(&self, block_index: usize, data: &[u8]) -> Result<(), DiskError>;
    fn read_serilized_data(&self, block_index: usize) -> Result<Vec<u8>, DiskError>;
}
