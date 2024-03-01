use anyhow::Result;

use crate::dir_entry::{Block, DirEntry, FileType};
use crate::errors::FileError;
use crate::traits::Directory;
use crate::utils::path_handler::{absolutize_from, split_path};
use crate::FileSystem;

impl Directory for FileSystem {
    fn create_dir(&mut self, path: &str) -> Result<()> {
        let abs_path = absolutize_from(path, &self.curr_block.path);
        let (parent, name) = split_path(abs_path);

        if name.len() > 55 {
            return Err(FileError::FilenameTooLong.into());
        } else if name.is_empty() {
            return Err(FileError::InvalidFilename(name.to_string()).into());
        }

        match self.curr_block.get_entry(&name.clone().into()) {
            Some(entry) => {
                return if entry.file_type == FileType::Directory {
                    Err(FileError::DirectoryExists(name.into()).into())
                } else {
                    Err(FileError::FileExists(name.into()).into())
                }
            }
            None => {
                let new_entry =
                    DirEntry::new(name.into(), FileType::Directory, 0, self.get_free_block()?);
                let new_block = Block::new(new_entry.clone(), new_entry.blk_num.clone());
                self.write_data::<Block>(&new_block, new_entry.blk_num)?;
                self.curr_block.add_entry(new_entry)?;
                self.write_curr_blk()?;
            }
        }
        Ok(())
    }

    fn delete_dir(&mut self, name: &str) -> Result<()> {
        unimplemented!()
    }

    fn list_dir(&self) -> Result<()> {
        // Print column headers
        println!(
            "{:20} {:10} {:15} {:10}",
            "Name", "Type", "Size (Bytes)", "Block Number"
        );

        for entry in &self.curr_block.entries {
            if !entry.name.is_empty() {
                let entry_type = match entry.file_type {
                    FileType::File => "File",
                    FileType::Directory => "Directory",
                };
                // Format and print each entry according to the column widths
                println!(
                    "{:20} {:10} {:15} {:10}",
                    entry.name, entry_type, entry.size, entry.blk_num
                );
            }
        }

        Ok(())
    }
}
