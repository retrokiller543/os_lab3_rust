#[cfg(feature = "debug")]
use log::debug;

use logger_macro::trace_log;
use rustic_disk::traits::BlockStorage;

use crate::dir_entry::{DirBlock, DirEntry};
use crate::errors::FileError;
use crate::utils::{check_access_level, fixed_str, path_handler};
use crate::{FileSystem, READ, ROOT_BLK};

impl FileSystem {
    #[trace_log]
    pub fn read_dir_block(&self, entry: &DirEntry) -> anyhow::Result<DirBlock> {
        if entry.file_type != crate::dir_entry::FileType::Directory {
            return Err(FileError::NotADirectory(entry.clone().name).into());
        }

        let block_num = entry.blk_num;
        let mut block = self.disk.read_block::<DirBlock>(block_num as usize)?;

        block.parent_entry = entry.clone();
        block.blk_num = block_num;

        Ok(block)
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[trace_log]
    pub fn write_dir_block(&self, block: &DirBlock) -> anyhow::Result<()> {
        self.disk.write_block(block.blk_num as usize, block)?;
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn write_dir_block(&mut self, block: &DirBlock) -> anyhow::Result<()> {
        self.disk.write_block(block.blk_num as usize, block)?;
        Ok(())
    }

    #[trace_log]
    pub fn update_dir(&mut self, block: &mut DirBlock, path: String) -> anyhow::Result<()> {
        let abs_path = path_handler::absolutize_from(&path, "/");
        let (parent, name) = path_handler::split_path(abs_path);
        // iter over all parents and update their size and make sure that the new entry is added
        let mut dirs = self.get_all_dirs(parent)?;
        dirs.pop(); // remove the last one since we are going to update it
        dirs.push(block.clone());
        let size_to_add = match block.get_entry(&name.clone().into()) {
            Some(entry) => entry.size,
            None => return Err(FileError::FileNotFound.into()),
        };

        let mut dirs_iter = dirs.iter_mut().peekable();
        while let Some(dir) = dirs_iter.next() {
            if let Some(next_dir) = dirs_iter.peek() {
                match dir.get_entry_mut(&next_dir.parent_entry.name) {
                    Some(entry) => {
                        entry.size += size_to_add;
                    }
                    None => {
                        return Err(FileError::FileNotFound.into());
                    }
                }
            }

            self.write_dir_block(dir)?;
        }

        // update working dir
        let cwd = self.curr_block.path.clone();
        self.curr_block = self.traverse_dir(cwd)?;

        Ok(())
    }

    #[trace_log]
    fn read_root_dir(&self) -> anyhow::Result<DirBlock> {
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

    //#[trace_log]
    pub fn change_dir(&mut self, path: &str) -> anyhow::Result<()> {
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

        // Do we have read permission for the parent directory?
        if !check_access_level(parent_block.parent_entry.access_level, READ) {
            return Err(FileError::NoPermissionToRead(name).into());
        }

        match parent_block.get_entry(&name.clone().into()) {
            Some(entry) => {
                if entry.file_type != crate::dir_entry::FileType::Directory {
                    return Err(FileError::NotADirectory(name.into()).into());
                }

                let mut new_block = self.read_dir_block(entry)?;
                let path_buf = std::path::PathBuf::from(&parent_block.path);
                new_block.path = path_buf.join(&name).to_str().unwrap().to_string();

                self.curr_block = new_block;
            }
            None => return Err(FileError::FileNotFound.into()),
        };

        Ok(())
    }

    #[trace_log]
    pub(crate) fn traverse_dir(&self, path: String) -> anyhow::Result<DirBlock> {
        let names = path
            .split('/')
            .filter(|&c| !c.is_empty())
            .collect::<Vec<&str>>();
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
                    new_block.path = path_buf.join(name).to_str().unwrap().to_string();
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

    #[trace_log]
    pub fn get_all_dirs(&self, path: String) -> anyhow::Result<Vec<DirBlock>> {
        let names = path
            .split('/')
            .filter(|&c| !c.is_empty())
            .collect::<Vec<&str>>();
        #[cfg(feature = "debug")]
        {
            debug!("Traversing path: {:?}", names)
        }
        let mut block = self.read_root_dir()?; // start from root since fuck being efficent :)
        let mut blocks = Vec::new();
        blocks.push(block.clone());

        for name in names {
            let entry = block.get_entry(&name.into());
            match entry {
                Some(entry) => {
                    if entry.file_type != crate::dir_entry::FileType::Directory {
                        return Err(FileError::NotADirectory(name.into()).into());
                    }

                    let mut new_block = self.read_dir_block(entry)?;
                    let path_buf = std::path::PathBuf::from(&block.path);
                    new_block.path = path_buf.join(name).to_str().unwrap().to_string();
                    #[cfg(feature = "debug")]
                    {
                        debug!("New block: {:?}", new_block);
                    }
                    blocks.push(new_block.clone());

                    block = new_block;
                }
                None => {
                    return Err(FileError::FileNotFound.into());
                }
            }
        }

        Ok(blocks)
    }

    #[trace_log]
    pub fn print_working_dir(&mut self) -> anyhow::Result<()> {
        let path = if self.curr_block.path.is_empty() {
            "/".to_string()
        } else {
            self.curr_block.path.clone()
        };
        self.io_handler.write(path)?;
        Ok(())
    }
}
