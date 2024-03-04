use std::path::Path;

use anyhow::Result;
use path_absolutize::Absolutize;

use crate::errors::{FSError, FileError};
use crate::traits::DirEntryHandling;
use crate::utils::fixed_str::FixedString;
use crate::FileSystem;

impl DirEntryHandling for FileSystem {
    /// The move function is used to move a file from one directory to another
    fn move_entry(&mut self, source: &str, dest: &str) -> Result<()> {
        let src_binding = Path::new(source).absolutize()?;
        let src_path = src_binding.to_str().ok_or(FSError::PathError)?;
        let src_parent = Path::new(&src_path)
            .parent()
            .unwrap()
            .to_str()
            .ok_or(FSError::PathError)?;
        let src_name: FixedString = Path::new(&src_path)
            .file_name()
            .unwrap()
            .to_str()
            .ok_or(FSError::PathError)?
            .into();

        let dest_binding = Path::new(dest).absolutize()?;
        let dest_path = dest_binding.to_str().ok_or(FSError::PathError)?;
        let dest_parent = Path::new(&dest_path)
            .parent()
            .unwrap()
            .to_str()
            .ok_or(FSError::PathError)?;
        let dest_name: FixedString = Path::new(&dest_path)
            .file_name()
            .unwrap()
            .to_str()
            .ok_or(FSError::PathError)?
            .into();

        // NOTE: this will need to be updated to handle directories

        if let Some(entry) = self.curr_block.get_entry(&src_name) {
            let mut new_entry = entry.clone();
            new_entry.name = dest_name;

            self.curr_block.add_entry(new_entry)?;
            self.curr_block.remove_entry(&src_name)?;
        } else {
            return Err(FileError::FileNotFound.into());
        }

        self.write_curr_blk()?;

        Ok(())
    }

    /// The copy function is used to copy a file from one directory to another
    fn copy_entry(&mut self, source: &str, dest: &str) -> Result<()> {
        let src_binding = Path::new(source).absolutize()?;
        let src_path = src_binding.to_str().ok_or(FSError::PathError)?;
        let src_parent = Path::new(&src_path)
            .parent()
            .unwrap()
            .to_str()
            .ok_or(FSError::PathError)?;
        let src_name: FixedString = Path::new(&src_path)
            .file_name()
            .unwrap()
            .to_str()
            .ok_or(FSError::PathError)?
            .into();

        let dest_binding = Path::new(dest).absolutize()?;
        let dest_path = dest_binding.to_str().ok_or(FSError::PathError)?;
        let dest_parent = Path::new(&dest_path)
            .parent()
            .unwrap()
            .to_str()
            .ok_or(FSError::PathError)?;
        let dest_name: FixedString = Path::new(&dest_path)
            .file_name()
            .unwrap()
            .to_str()
            .ok_or(FSError::PathError)?
            .into();

        // NOTE: this will need to be updated to handle directories

        if let Some(entry) = self.curr_block.get_entry(&src_name) {
            let mut new_entry = entry.clone();
            new_entry.name = dest_name;
            let data = self.read_file_data(new_entry.blk_num)?;
            new_entry.blk_num = self.get_free_block()?;
            self.write_data(&data, new_entry.blk_num)?;
            self.curr_block.add_entry(new_entry)?;
        } else {
            return Err(FileError::FileNotFound.into());
        }

        self.write_curr_blk()?;

        Ok(())
    }
}
