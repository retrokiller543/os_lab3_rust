use thiserror::Error;

#[derive(Error, Debug)]
pub enum FSError {
    #[error("Error Serilizing data with error: {0}")]
    SerializationError(#[from] bincode::Error),
    #[error("Error Constructing path")]
    PathError,
    #[error("Error no free blocks in the FAT")]
    NoFreeBlocks,
    #[error("Error reading block")]
    InvalidBlockReference,
}

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Filename is too long")]
    FilenameTooLong,
    #[error("File not found")]
    FileNotFound,
    #[error("File already exists")]
    FileAlreadyExists,
    #[error("File is a directory")]
    FileIsDirectory,
    #[error("Filename is invalid: {0}")]
    InvalidFilename(String),
}