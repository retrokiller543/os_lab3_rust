use thiserror::Error;

use crate::utils::fixed_str::FixedString;

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
    #[error("Python error: {0}")]
    PyError(String),
    #[error("Embeded Python not supported on this platform, please see https://pyo3.rs/v0.20.2/building_and_distribution.html?highlight=pypy%20embeded#dynamically-embedding-the-python-interpreter for more information.\nIt might work in certain cases but its hard to support them all sadly. A new feature might be added in the future to allow to compile anyway but this will never be used in the precompiled versions!")]
    PythonNotSupported
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
    #[error("File is not a directory{0}")]
    NotADirectory(FixedString),
    #[error("File Already exists with name: {0}")]
    FileExists(FixedString),
    #[error("Directory already exists with name: {0}")]
    DirectoryExists(FixedString),
    #[error("Invalid access level: {0}")]
    InvalidAccessLevel(u8),
    #[error("No premission to write to file: {0}")]
    NoPermissionToWrite(String),
    #[error("No premission to read file: {0}")]
    NoPermissionToRead(String),
    #[error("No premission to execute: {0}")]
    NoPermissionToExecute(String),
    #[error("Python error: {0}")]
    PyError(String),
}

#[derive(Error, Debug)]
pub enum IOHandlerError {
    #[error("Input/Output error: {0}")]
    IOError(String),
    #[error("Python error: {0}")]
    PyError(String),
    // You can add more specific error types here as needed
}
