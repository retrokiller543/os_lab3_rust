// Helper functions and structs

use anyhow::Result;

use rustic_disk::traits::BlockStorage;

use crate::{FileSystem, ROOT_BLK};
use crate::dir_entry::{Block, DirEntry};
use crate::errors::FileError;

pub mod fixed_str;
pub(crate) mod path_handler;

impl FileSystem {
    pub fn read_dir_block(&self, entry: &DirEntry) -> Result<Block> {
        if entry.file_type != crate::dir_entry::FileType::Directory {
            return Err(crate::errors::FileError::NotADirectory(entry.clone().name).into());
        }
        let block_num = entry.blk_num;
        let mut block = self.disk.read_block::<Block>(block_num as usize)?;

        block.parent_entry = entry.clone();
        block.blk_num = block_num;
        block.path = self.curr_block.path.clone() + &entry.name.to_string() + "/";

        Ok(block)
    }

    pub fn change_dir(&mut self, path: &str) -> Result<()> {
        let abs_path = path_handler::absolutize_from(path, &self.curr_block.path);
        let (parent, name) = path_handler::split_path(abs_path.clone());

        // there are two parts to this, one is to move to the parent of the directory,
        // the second is
        // to move to the directory or error if it does not exist or is a file
        if parent == "/".to_string() {
            let root_entry = DirEntry::new(fixed_str::FixedString::from("/"), crate::dir_entry::FileType::Directory, 0, ROOT_BLK as u16);
            let root_block = self.read_dir_block(&root_entry)?;
            self.curr_block = root_block;

            if name.is_empty() {
                return Ok(());
            }

            let entry = self.curr_block.get_entry(&name.clone().into());
            match entry {
                Some(entry) => {
                    if entry.file_type != crate::dir_entry::FileType::Directory {
                        return Err(FileError::NotADirectory(name.into()).into());
                    }
                    self.curr_block = self.read_dir_block(&entry)?;
                },
                None => {
                    return Err(FileError::FileNotFound.into());
                }
            }

            return Ok(());
        }

        Ok(())
    }
}
