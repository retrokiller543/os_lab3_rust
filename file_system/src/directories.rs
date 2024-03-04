use std::path::Path;
use anyhow::Result;

use crate::dir_entry::{Block, DirEntry, FileType};
use crate::errors::{FileError, FSError};
use crate::traits::Directory;
use crate::utils::path_handler::{absolutize_from, split_path};
use crate::FileSystem;

impl Directory for FileSystem {
    /// Creates a directory in the current directory
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
                let new_block = Block::new(new_entry.clone(), new_entry.blk_num);
                self.write_data::<Block>(&new_block, new_entry.blk_num)?;
                self.curr_block.add_entry(new_entry)?;
                self.write_curr_blk()?;
            }
        }
        Ok(())
    }

    /// Deletes a directory in the current directory
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



fn chmod(access_rights: &str, file_path: &str) -> Result<()> {
    // Initialize with single path?
    let binding = Path::new(file_path).absolutize()?; // nåt är fel här me know
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

    let access_level: u32 = match access_rights.parse() {
        Ok(num) => num,
        Err(_) => {
            return Err(FileError::InvalidAccessMode(access_rights.to_string()).into());
        }
    };

    if access_level < 0 || (0o7) < access_level {
        return Err(FileError::InvalidAccessMode(access_rights.to_string()).into());
    };

    let (mut file_name, mut dir_name) = match Path::new(file_path).file_name() {
        Some(name) => (name.to_string_lossy().to_string(), file_path.to_string()),
        None => {
            return Err(FileError::InvalidAccessMode(file_path.to_string()).into());
        }
    };

    if file_name.is_empty() {
        file_name = dir_name.clone();
        dir_name = parent.to_string();
    }

    // update workong directory
    let mut dir = self.get_dir(&dir_name)?;
    let entry = dir.get_entry(&file_name.into()).ok_or(FileError::FileNotFound)?;
    entry.access = access_level as u8;
    dir.write_curr_blk()?;
    Ok(())

}

// Är nog ganska säker på detta iaf
fn get_access_rights(access: u8) -> &'static str{
    match access {
        0x07 => "rwx",
        0x06 => "rw-",
        0x05 => "r-x",
        0x04 => "r--",
        0x03 => "-wx",
        0x02 => "-w-",
        0x01 => "--x",
        0x00 => "---",
        _ => "???",
    }
}

