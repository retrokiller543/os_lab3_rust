use crate::dir_entry::{DirBlock, FileType};
use anyhow::Result;
use logger_macro::trace_log;

use crate::errors::FileError;

use crate::{FileSystem, READ, READ_WRITE, READ_WRITE_EXECUTE, WRITE};
use crate::prelude::Permissions;
use crate::utils::check_access_level;
use crate::traits::DirEntryHandling;
use crate::utils::path_handler::{absolutize_from, split_path};

impl DirEntryHandling for FileSystem {
    /// The move function is used to move a file from one directory to another
    #[trace_log]
    fn move_entry(&mut self, source: &str, dest: &str) -> Result<()> {
        let abs_src = absolutize_from(source, &self.curr_block.path);
        let abs_dest = absolutize_from(dest, &self.curr_block.path);

        let (src_parent, src_name) = split_path(abs_src.clone());
        let (dest_parent, dest_name) = split_path(abs_dest.clone());

        let mut src_parent_block = self.traverse_dir(src_parent)?;
        let mut dest_parent_block = self.traverse_dir(dest_parent)?;

        // check if we have write permission for destnation and read permission for source
        if !check_access_level(src_parent_block.parent_entry.access_level, READ) {
            return Err(FileError::NoPermissionToRead(dest_name).into());
        }
        if !check_access_level(dest_parent_block.parent_entry.access_level, WRITE) {
            return Err(FileError::NoPermissionToWrite(dest_name).into());
        }

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

            if !check_access_level(new_entry.access_level, READ_WRITE) {
                return Err(FileError::NoPermissionToWrite(new_entry.name.to_string()).into());
            }

            if !dest_is_dir {
                new_entry.name = dest_name.clone().into();
            }

            if dest_parent_block
                .get_entry(&dest_name.clone().into())
                .is_some()
            {
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
    #[trace_log]
    fn copy_entry(&mut self, source: &str, dest: &str) -> Result<()> {
        let abs_src = absolutize_from(source, &self.curr_block.path);
        let abs_dest = absolutize_from(dest, &self.curr_block.path);

        let (src_parent, src_name) = split_path(abs_src);
        let (dest_parent, dest_name) = split_path(abs_dest.clone());

        let src_parent_block = self.traverse_dir(src_parent)?;
        let mut dest_parent_block = self.traverse_dir(dest_parent)?;

        // check if we have write permission for destnation and read permission for source
        if !check_access_level(src_parent_block.parent_entry.access_level, READ) {
            return Err(FileError::NoPermissionToRead(dest_name).into());
        }
        if !check_access_level(dest_parent_block.parent_entry.access_level, WRITE) {
            return Err(FileError::NoPermissionToWrite(dest_name).into());
        }

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

            if !check_access_level(new_entry.access_level, READ_WRITE) {
                return Err(FileError::NoPermissionToWrite(new_entry.name.to_string()).into());
            }

            if !dest_is_dir {
                new_entry.name = dest_name.clone().into();
            }

            if dest_parent_block
                .get_entry(&dest_name.clone().into())
                .is_some()
            {
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

#[trace_log]
fn convert_str_to_u8_digit(s: &str) -> Result<u8, std::num::ParseIntError> {
    s.parse::<u8>()
}

impl Permissions for FileSystem {
    #[trace_log]
    fn change_permissions(&mut self, path: &str, permissions: &str) -> Result<()> {
        let abs_path = absolutize_from(path, &self.curr_block.path);
        let (parent, name) = split_path(abs_path);
        let mut parent_block = self.traverse_dir(parent)?;

        // convert safley the &str permissions into a u8
        let permissions = convert_str_to_u8_digit(permissions)?;

        if let Some(entry) = parent_block.clone().get_entry_mut(&name.into()) {
            if permissions > READ_WRITE_EXECUTE {
                return Err(FileError::InvalidAccessLevel(permissions).into());
            }

            entry.access_level = permissions;
            parent_block.update_entry(entry)?;
            self.write_dir_block(&parent_block)?;

            if entry.file_type == FileType::Directory {
                let mut block = self.read_dir_block(entry)?;
                block
                    .entries
                    .iter_mut()
                    .for_each(|entry| entry.access_level = permissions);
                self.write_dir_block(&block)?;
            }

            self.update_curr_dir()?;
        } else {
            return Err(FileError::FileNotFound.into());
        }
        Ok(())
    }
}
