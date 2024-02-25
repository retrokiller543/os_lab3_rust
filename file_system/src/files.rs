use std::io;
use std::io::BufRead;
use crate::{FAT, FileSystem};
use crate::traits::File;
use std::path::Path;
use log::debug;
use path_absolutize::*;
use serde_derive::{Deserialize, Serialize};
use crate::dir_entry::{DirEntry, FileType};
use crate::errors::{FileError, FSError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FileData {
    pub(crate) data: Vec<u8>,
}

impl File for FileSystem {
    /// # Create a file in the current directory
    fn create_file(&mut self, path: &str) -> anyhow::Result<()> {
        let binding = Path::new(path).absolutize()?;
        let path = binding.to_str().ok_or(FSError::PathError)?;
        let parent = Path::new(&path).parent().unwrap().to_str().ok_or(FSError::PathError)?;
        let name = Path::new(&path).file_name().unwrap().to_str().ok_or(FSError::PathError)?;

        #[cfg(feature = "debug")]
        {
            debug!("Path: {}", path);
            debug!("Parent: {}", parent);
            debug!("Name: {}", name);
        }

        if name.len() > 55 {
            return Err(FileError::FilenameTooLong.into());
        } else if name.is_empty() {
            return Err(FileError::InvalidFilename(name.to_string()).into());
        }

        // make code to check if file exists and parent exists
        for entry in self.curr_block.entries.iter() {
            if entry.name == name {
                return Err(FileError::FileAlreadyExists.into());
            }
        }

        // read data from user
        let mut data = String::new();

        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let input_data = line.expect("Failed to read line");
            if input_data.is_empty() {
                break;
            }
            data.push_str(&input_data);
            data.push('\n'); // Add a newline character after each line
        }

        #[cfg(feature = "debug")]
        {
            debug!("Data: {}", data);
        }

        let file_data = FileData {
            data: data.as_bytes().to_vec(),
        };

        // find the first free block
        let mut blk_num = self.get_free_block()? as u64;

        self.write_data(&file_data, blk_num)?;

        let entry = DirEntry {
            name: name.to_string(),
            file_type: FileType::File,
            size: data.len() as u64,
            blk_num,
        };

        #[cfg(feature = "debug")]
        {
            debug!("Entry: {:?}", entry);
        }

        self.curr_block.entries.push(entry);

        self.write_curr_blk()?;

        Ok(())
    }

    fn delete_file(&mut self, name: &str) -> anyhow::Result<()> {
        todo!()
    }

    /// the cat function
    fn read_file(&self, name: &str) -> anyhow::Result<()> {
        let binding = Path::new(name).absolutize()?;
        let path = binding.to_str().ok_or(FSError::PathError)?;
        let parent = Path::new(&path).parent().unwrap().to_str().ok_or(FSError::PathError)?;
        let name = Path::new(&path).file_name().unwrap().to_str().ok_or(FSError::PathError)?;

        #[cfg(feature = "debug")]
        {
            debug!("Path: {}", path);
            debug!("Parent: {}", parent);
            debug!("Name: {}", name);
        }

        if name.is_empty() {
            return Err(FileError::InvalidFilename(name.to_string()).into());
        }

        let mut file_entry: &DirEntry = &DirEntry::default();

        for entry in self.curr_block.entries.iter() {
            if entry.name == name {
                file_entry = entry;
            }
        }

        if file_entry == &DirEntry::default() {
            return Err(FileError::FileNotFound.into());
        }

        // make sure its of type file
        if file_entry.file_type != FileType::File {
            return Err(FileError::FileIsDirectory.into());
        }

        // recursivly check the fat until we reach EOF and read all blocks in order
        let data = self.read_file_data(file_entry.blk_num)?;

        #[cfg(feature = "debug")]
        {
            debug!("Data: {}", String::from_utf8_lossy(&data.data));
        }

        Ok(())
    }

    fn write_file(&mut self, name: &str) -> anyhow::Result<()> {
        todo!()
    }
}