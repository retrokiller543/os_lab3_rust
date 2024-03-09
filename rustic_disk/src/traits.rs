use std::fmt::Debug;
use crate::errors::DiskError;
use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

pub trait BlockStorage {
    fn read_block<T: DeserializeOwned + std::fmt::Debug>(
        &self,
        block_index: usize,
    ) -> Result<T, DiskError>;
    #[cfg(not(target_arch = "wasm32"))]
    fn write_block<T: Serialize + Debug>(&self, block_index: usize, data: &T) -> Result<(), DiskError>;
    #[cfg(not(target_arch = "wasm32"))]
    fn write_raw_data(&self, block_index: usize, data: &[u8]) -> Result<(), DiskError>;
    #[cfg(target_arch = "wasm32")]
    fn write_block<T: Serialize + Debug>(&mut self, block_index: usize, data: &T) -> Result<(), DiskError>;
    #[cfg(target_arch = "wasm32")]
    fn write_raw_data(&mut self, block_index: usize, data: &[u8]) -> Result<(), DiskError>;
    fn read_raw_data(&self, block_index: usize) -> Result<Vec<u8>, DiskError>;
}
