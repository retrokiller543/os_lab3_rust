use anyhow::Result;

use crate::dir_entry::{DirBlock, DirEntry, FileType};
use crate::errors::FileError;
use crate::traits::Directory;
use crate::utils::path_handler::{absolutize_from, split_path};
use crate::FileSystem;

impl Directory for FileSystem {
    /// Creates a directory in the current directory
    fn create_dir(&mut self, path: &str) -> Result<()> {
        let abs_path = absolutize_from(path, &self.curr_block.path);
        let (parent, name) = split_path(abs_path.clone());

        if name.len() > 55 {
            return Err(FileError::FilenameTooLong.into());
        } else if name.is_empty() {
            return Err(FileError::InvalidFilename(name.to_string()).into());
        }

        let mut parent_block = self.traverse_dir(parent)?;

        match parent_block.get_entry(&name.clone().into()) {
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
                let new_block = DirBlock::new(new_entry.clone(), new_entry.blk_num);
                self.write_data::<DirBlock>(&new_block, new_entry.blk_num)?;
                parent_block.add_entry(new_entry)?;
                self.update_dir(&mut parent_block, abs_path)?;
            }
        }

        Ok(())
    }

    /// Deletes a directory in the current directory
    fn delete_dir(&mut self, path: &str) -> Result<()> {
        let abs_path = absolutize_from(path, &self.curr_block.path);
        let (parent, name) = split_path(abs_path.clone());

        let mut parent_block = self.traverse_dir(parent.clone())?;
        let binding = parent_block.clone();
        let entry = binding.get_entry(&name.into()).ok_or(FileError::FileNotFound)?;

        if entry.file_type != FileType::Directory {
            return Err(FileError::NotADirectory(path.into()).into());
        }

        self.remove_dir_data(entry.blk_num)?;
        parent_block.remove_entry(&entry.name)?;
        self.write_dir_block(&parent_block)?;
        Ok(())
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
