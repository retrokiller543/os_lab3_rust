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
}
