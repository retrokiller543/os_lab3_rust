use anyhow::Result;
use crate::dir_entry::{DirBlock, FileType};

use crate::errors::FileError;
use crate::file_data::FileData;
use crate::traits::DirEntryHandling;
use crate::FileSystem;
use crate::prelude::Permissions;
use crate::utils::path_handler::{absolutize_from, split_path};

impl DirEntryHandling for FileSystem {
    /// The move function is used to move a file from one directory to another
    fn move_entry(&mut self, source: &str, dest: &str) -> Result<()> {
        let abs_src = absolutize_from(source, &self.curr_block.path);
        let abs_dest = absolutize_from(dest, &self.curr_block.path);

        let (src_parent, src_name) = split_path(abs_src.clone());
        let (dest_parent, dest_name) = split_path(abs_dest.clone());

        let mut src_parent_block = self.traverse_dir(src_parent)?;
        let mut dest_parent_block = self.traverse_dir(dest_parent)?;

        let mut dest_is_dir = false;

        if let Some(dest_entry) = dest_parent_block.get_entry(&dest_name.clone().into()) {
            if dest_entry.file_type == FileType::Directory {
                dest_parent_block = self.traverse_dir(abs_dest.clone())?;
                dest_is_dir = true;
            }
        }

        // NOTE: this will need to be updated to handle directories

        if let Some(entry) = src_parent_block.get_entry(&src_name.clone().into()) {
            let mut new_entry = entry.clone();

            if !dest_is_dir  {
                new_entry.name = dest_name.clone().into();
            }

            if dest_parent_block.get_entry(&dest_name.clone().into()).is_some() {
                return Err(FileError::FileExists(dest_name.into()).into());
            }

            if dest_parent_block == src_parent_block {
                dest_parent_block.remove_entry(&src_name.clone().into())?;
            }

            dest_parent_block.add_entry(new_entry)?;
            src_parent_block.remove_entry(&src_name.into())?;
            self.write_dir_block(&src_parent_block)?;
            self.write_dir_block(&dest_parent_block)?;
            //self.update_dir(&mut src_parent_block, abs_src)?;
            //self.update_dir(&mut dest_parent_block, abs_dest)?;
            self.update_curr_dir()?;
        } else {
            return Err(FileError::FileNotFound.into());
        }

        Ok(())
    }

    /// The copy function is used to copy a file from one directory to another
    fn copy_entry(&mut self, source: &str, dest: &str) -> Result<()> {
        let abs_src = absolutize_from(source, &self.curr_block.path);
        let abs_dest = absolutize_from(dest, &self.curr_block.path);

        let (src_parent, src_name) = split_path(abs_src);
        let (dest_parent, dest_name) = split_path(abs_dest.clone());

        let src_parent_block = self.traverse_dir(src_parent)?;
        let mut dest_parent_block = self.traverse_dir(dest_parent)?;

        let mut dest_is_dir = false;

        if let Some(dest_entry) = dest_parent_block.get_entry(&dest_name.clone().into()) {
            if dest_entry.file_type == FileType::Directory {
                dest_parent_block = self.traverse_dir(abs_dest.clone())?;
                dest_is_dir = true;
            }
        }

        // NOTE: this will need to be updated to handle directories

        if let Some(entry) = self.curr_block.get_entry(&src_name.into()) {
            let mut new_entry = entry.clone();

            if !dest_is_dir  {
                new_entry.name = dest_name.clone().into();
            }

            if dest_parent_block.get_entry(&dest_name.clone().into()).is_some() {
                return Err(FileError::FileExists(dest_name.into()).into());
            }

            match new_entry.file_type {
                FileType::File => {
                    let data = self.read_file_data(new_entry.blk_num)?;
                    new_entry.blk_num = self.get_free_block()?;
                    self.write_data(&data, new_entry.blk_num)?;
                }
                FileType::Directory => {
                    new_entry.size = 0;
                    let block = DirBlock::default();
                    new_entry.blk_num = self.get_free_block()?;
                    self.write_data(&block, new_entry.blk_num)?;
                }
            }

            dest_parent_block.add_entry(new_entry)?;
            self.write_dir_block(&dest_parent_block)?;
            self.update_curr_dir()?;
        } else {
            return Err(FileError::FileNotFound.into());
        }

        Ok(())
    }
}

impl Permissions for FileSystem {
    fn change_permissions(&mut self, path: &str, permissions: u8) -> Result<()> {
        todo!()
    }
}