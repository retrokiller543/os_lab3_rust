use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::sync::PoisonError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiskError {
    #[error("Error creating disk file: {0}")]
    CreateDiskError(#[from] std::io::Error),
    #[error("Serialized data exceeds block size")]
    DataExceedsBlockSize,
    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),
    #[error("Deserialization error: {0}")]
    DeserializationError(bincode::Error),
    #[error("Block position overflow")]
    PositionOverflow,
    #[error("Seek error: {0}")]
    SeekError(std::io::Error),
    #[error("Error reading disk file: {0}")]
    ReadDiskError(std::io::Error),
    #[error("Error writing to disk file: {0}")]
    WriteDiskError(std::io::Error),
    #[error("Error truncating disk file")]
    FileLockError(#[from] MyPoisonError),
}

// Define a custom error type for poison errors
#[derive(Debug)]
pub struct MyPoisonError {
    msg: String,
}

impl Display for MyPoisonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for MyPoisonError {}

impl<T> From<PoisonError<T>> for MyPoisonError {
    fn from(err: PoisonError<T>) -> Self {
        MyPoisonError {
            msg: format!("Mutex lock poisoned: {:?}", err),
        }
    }
}
