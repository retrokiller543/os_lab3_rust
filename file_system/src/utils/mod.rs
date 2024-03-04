// Helper functions and structs

use anyhow::Result;
use log::{debug, trace};

use rustic_disk::traits::BlockStorage;

use crate::dir_entry::{Block, DirEntry};
use crate::errors::FileError;
use crate::{FileSystem, ROOT_BLK};

pub mod fixed_str;
pub(crate) mod path_handler;

impl FileSystem {
    pub fn read_dir_block(&self, entry: &DirEntry) -> Result<Block> {
        if entry.file_type != crate::dir_entry::FileType::Directory {
            return Err(FileError::NotADirectory(entry.clone().name).into());
        }

        let block_num = entry.blk_num;
        let mut block = self.disk.read_block::<Block>(block_num as usize)?;

        block.parent_entry = entry.clone();
        block.blk_num = block_num;

        Ok(block)
    }

    fn read_root_dir(&self) -> Result<Block> {
        let root_entry = DirEntry::new(
            fixed_str::FixedString::from("/"),
            crate::dir_entry::FileType::Directory,
            0,
            ROOT_BLK as u16,
        );

        let mut root_block = self.read_dir_block(&root_entry)?;
        root_block.path = "/".to_string();

        Ok(root_block)
    }

    pub fn change_dir(&mut self, path: &str) -> Result<()> {
        let abs_path = path_handler::absolutize_from(path, &self.curr_block.path);
        let (parent, name) = path_handler::split_path(abs_path.clone());

        if parent == "/" {
            let root = self.read_root_dir()?;

            if name.is_empty() {
                self.curr_block = root;
                return Ok(());
            }

            let entry = root.get_entry(&name.clone().into());
            match entry {
                Some(entry) => {
                    if entry.file_type != crate::dir_entry::FileType::Directory {
                        return Err(FileError::NotADirectory(name.into()).into());
                    }

                    let mut block = self.read_dir_block(entry)?;
                    let path_buf = std::path::PathBuf::from(&root.path);
                    block.path = path_buf.join(&name).to_str().unwrap().to_string();

                    self.curr_block = block;
                }
                None => {
                    return Err(FileError::FileNotFound.into());
                }
            }

            return Ok(());
        }

        let parent_block = self.traverse_dir(parent)?;

        let entry = match parent_block.get_entry(&name.clone().into()) {
            Some(entry) => {
                if entry.file_type != crate::dir_entry::FileType::Directory {
                    return Err(FileError::NotADirectory(name.into()).into());
                }

                let mut new_block = self.read_dir_block(entry)?;
                let path_buf = std::path::PathBuf::from(&parent_block.path);
                new_block.path = path_buf.join(&name).to_str().unwrap().to_string();

                self.curr_block = new_block;
            },
            None => return Err(FileError::FileNotFound.into()),
        };

        Ok(())
    }

    fn traverse_dir(&self, path: String) -> Result<Block> {
        #[cfg(feature = "trace")]
        {
            trace!("traverse_dir({})", path)
        }
        let names = path.split('/').filter(|&c| !c.is_empty()).collect::<Vec<&str>>();
        #[cfg(feature = "debug")]
        {
            debug!("Traversing path: {:?}", names)
        }
        let mut block = self.read_root_dir()?; // start from root since fuck being efficent :)

        for name in names {
            let entry = block.get_entry(&name.into());
            match entry {
                Some(entry) => {
                    if entry.file_type != crate::dir_entry::FileType::Directory {
                        return Err(FileError::NotADirectory(name.into()).into());
                    }

                    let mut new_block = self.read_dir_block(entry)?;
                    let path_buf = std::path::PathBuf::from(&block.path);
                    new_block.path = path_buf.join(&name).to_str().unwrap().to_string();
                    #[cfg(feature = "debug")]
                    {
                        debug!("New block: {:?}", new_block);
                    }

                    block = new_block;
                }
                None => {
                    return Err(FileError::FileNotFound.into());
                }
            }
        }

        Ok(block)
    }

    pub fn print_working_dir(&self) -> Result<()> {
        let path = if self.curr_block.path.is_empty() {
            "/".to_string()
        } else {
            self.curr_block.path.clone()
        };
        println!("{}", path);
        Ok(())
    }
}
