use anyhow::Result;
use logger_macro::trace_log;
use prettytable::{format, row, Table};

use crate::dir_entry::{DirBlock, DirEntry, FileType};
use crate::errors::FileError;
use crate::traits::Directory;
use crate::utils::path_handler::{absolutize_from, split_path};
use crate::{FileSystem, get_access_rights, READ, WRITE};
use crate::utils::check_access_level;

impl Directory for FileSystem {
    /// Creates a directory in the current directory
    #[trace_log]
    fn create_dir(&mut self, path: &str) -> Result<()> {
        let abs_path = absolutize_from(path, &self.curr_block.path);
        let (parent, name) = split_path(abs_path.clone());

        if name.len() > 55 {
            return Err(FileError::FilenameTooLong.into());
        } else if name.is_empty() {
            return Err(FileError::InvalidFilename(name.to_string()).into());
        }

        let mut parent_block = self.traverse_dir(parent)?;
        if !check_access_level(parent_block.parent_entry.access_level, WRITE) {
            return Err(FileError::NoPermissionToWrite(name).into());
        }

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
    #[trace_log]
    fn delete_dir(&mut self, path: &str) -> Result<()> {
        let abs_path = absolutize_from(path, &self.curr_block.path);
        let (parent, name) = split_path(abs_path.clone());

        let mut parent_block = self.traverse_dir(parent.clone())?;
        if !check_access_level(parent_block.parent_entry.access_level, WRITE) {
            return Err(FileError::NoPermissionToWrite(name).into());
        }
        let binding = parent_block.clone();
        let entry = binding
            .get_entry(&name.into())
            .ok_or(FileError::FileNotFound)?;

        if entry.file_type != FileType::Directory {
            return Err(FileError::NotADirectory(path.into()).into());
        }

        self.remove_dir_data(entry, path)?;
        parent_block.remove_entry(&entry.name)?;
        self.write_dir_block(&parent_block)?;
        Ok(())
    }

    #[trace_log]
    fn list_dir(&mut self) -> Result<()> {

        // Check if we have read permission for the current directory
        if !check_access_level(self.curr_block.parent_entry.access_level, READ) {
            return Err(FileError::NoPermissionToRead(self.curr_block.path.clone()).into());
        }

        let mut table = Table::new();
        table.set_titles(row![
            "Name".to_string(),
            "Type".to_string(),
            "Size (Bytes)".to_string(),
            "Block Number".to_string(),
            "Access Rights".to_string(),
        ]);

        // Print each entry with dynamic column widths and explicit padding
        for entry in &self.curr_block.entries {
            if !entry.name.is_empty() {
                let entry_type = match entry.file_type {
                    FileType::File => "File",
                    FileType::Directory => "Directory",
                };

                table.add_row(row![
                    entry.name.to_string(),
                    entry_type.to_string(),
                    entry.size.to_string(),
                    entry.blk_num.to_string(),
                    get_access_rights(entry.access_level).to_string(),
                ]);
            }
        }

        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        self.io_handler.write(table.to_string())?;

        Ok(())
    }
}
