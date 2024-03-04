// Helper functions and structs

use anyhow::Result;
#[cfg(feature = "debug")]
use log::debug;
use rustic_disk::traits::BlockStorage;
use std::path::Path;
use path_absolutize::Absolutize;

use crate::dir_entry::FileType;
use crate::errors::{FileError, FSError};
use crate::FileSystem;
use crate::prelude::File;

pub mod fixed_str;
pub(crate) mod path_handler;
pub mod dirs;

impl FileSystem {
    /// The remove functon is used to delete a file from the current directory
    pub fn remove_entry(&mut self, name: &str) -> Result<()> {
        let binding = Path::new(name).absolutize()?;
        let path = binding.to_str().ok_or(FSError::PathError)?;
        let parent = Path::new(&path)
            .parent()
            .unwrap()
            .to_str()
            .ok_or(FSError::PathError)?;
        let name = Path::new(&path)
            .file_name()
            .unwrap()
            .to_str()
            .ok_or(FSError::PathError)?;

        let entry = self
            .curr_block
            .get_entry(&name.into())
            .ok_or(FileError::FileNotFound)?
            .clone();
        match entry.file_type {
            FileType::File => self.delete_file(&entry)?,
            FileType::Directory => {
                unimplemented!()
            }
        }
        Ok(())
    }
}
