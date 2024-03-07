// Helper functions and structs

use anyhow::Result;
use logger_macro::trace_log;

use crate::dir_entry::FileType;
use crate::errors::FileError;
use crate::prelude::{Directory, File};
use crate::utils::path_handler::absolutize_from;
use crate::FileSystem;

pub mod dirs;
pub mod fixed_str;
pub(crate) mod path_handler;

pub fn check_access_level(access_level: u8, required: u8) -> bool {
    (access_level & required) == required
}

impl FileSystem {
    /// The remove functon is used to delete a file from the current directory
    #[trace_log]
    pub fn remove_entry(&mut self, name: &str) -> Result<()> {
        let abs_path = absolutize_from(name, &self.curr_block.path);
        let (parent, name) = path_handler::split_path(abs_path.clone());

        let parent_block = self.traverse_dir(parent.clone())?;

        let entry = parent_block
            .get_entry(&name.clone().into())
            .ok_or(FileError::FileNotFound)?
            .clone();

        match entry.file_type {
            FileType::File => self.delete_file(&name)?,
            FileType::Directory => self.delete_dir(&name)?,
        }
        Ok(())
    }
}
