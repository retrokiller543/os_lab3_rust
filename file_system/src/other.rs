use std::path::Path;

use anyhow::Result;
use path_absolutize::Absolutize;

use crate::dir_entry::FileType;
use crate::errors::{FileError, FSError};
use crate::FileSystem;
use crate::traits::{DirEntryHandling, File};
use crate::utils::fixed_str::FixedString;

impl FileSystem {
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

        let entry = self.curr_block.get_entry(&name.into()).ok_or(FileError::FileNotFound)?.clone();
        match entry.file_type {
            FileType::File => {
                self.delete_file(&entry)?
            },
            FileType::Directory => { unimplemented!() }
        }
        Ok(())
    }
}

impl DirEntryHandling for FileSystem {
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
            .ok_or(FSError::PathError)?.into();

        let dest_binding = Path::new(dest).absolutize()?;
        let dest_path = dest_binding.to_str().ok_or(FSError::PathError)?;
        let dest_parent = Path::new(&dest_path).parent().unwrap().to_str().ok_or(FSError::PathError)?;
        let dest_name: FixedString = Path::new(&dest_path).file_name().unwrap().to_str().ok_or(FSError::PathError)?.into();

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
            .ok_or(FSError::PathError)?.into();

        let dest_binding = Path::new(dest).absolutize()?;
        let dest_path = dest_binding.to_str().ok_or(FSError::PathError)?;
        let dest_parent = Path::new(&dest_path).parent().unwrap().to_str().ok_or(FSError::PathError)?;
        let dest_name: FixedString = Path::new(&dest_path).file_name().unwrap().to_str().ok_or(FSError::PathError)?.into();

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