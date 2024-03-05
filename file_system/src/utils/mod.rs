// Helper functions and structs

use anyhow::Result;

use crate::dir_entry::FileType;
use crate::errors::FileError;
use crate::prelude::File;
use crate::FileSystem;
use crate::utils::path_handler::absolutize_from;

pub mod dirs;
pub mod fixed_str;
pub(crate) mod path_handler;

impl FileSystem {
    /// The remove functon is used to delete a file from the current directory
    pub fn remove_entry(&mut self, name: &str) -> Result<()> {
        let abs_path = absolutize_from(name, &self.curr_block.path);
        let (parent, name) = path_handler::split_path(abs_path.clone());

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
