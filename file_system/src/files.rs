#![allow(unused_variables)]

use crate::dir_entry::{DirEntry, FileType};
use crate::errors::{FSError, FileError};
use crate::traits::File;
use crate::FileSystem;
#[cfg(feature = "debug")]
use log::debug;
use path_absolutize::*;
use serde_derive::{Deserialize, Serialize};
use std::io;
use std::io::BufRead;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FileData {
    pub(crate) data: Vec<u8>,
}

impl File for FileSystem {
    /// # Create a file in the current directory
    fn create_file(&mut self, path: &str) -> anyhow::Result<()> {
        let binding = Path::new(path).absolutize()?;
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
            if entry.name == name.into() {
                return Err(FileError::FileAlreadyExists.into());
            }
        }

	let mut has_space: bool = false;

	// do we have space in the block for the file?
	for item in self.curr_block.entries.iter() {
		if item.name.is_empty() { has_space = true; }
	}

	if has_space == false {
		return Err(FileError::FileNotFound.into()); // make new error here
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
        data.pop(); // Remove the last newline character

        #[cfg(feature = "debug")]
        {
            debug!("Data: {}", data);
        }

        let file_data = FileData {
            data: data.as_bytes().to_vec(),
        };

        // find the first free block
        let blk_num = self.get_free_block()?;

        self.write_data(&file_data, blk_num)?;

        let entry = DirEntry {
            name: name.into(),
            file_type: FileType::File,
            size: data.len() as u64,
            blk_num,
        };

        #[cfg(feature = "debug")]
        {
            debug!("Entry: {:?}", entry);
        }

        // update size of the parent block
        self.curr_block.parent_entry.size += entry.size;

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
            if entry.name == name.into() {
                file_entry = entry;
            }
        }

        if file_entry == &DirEntry::default() {
            return Err(FileError::FileNotFound.into());
        }

        // make sure it's of type file
        if file_entry.file_type != FileType::File {
            return Err(FileError::FileIsDirectory.into());
        }

        // recursivly check the fat until we reach EOF and read all blocks in order
        let data = self.read_file_data(file_entry.blk_num)?;
        let data = String::from_utf8(data.data)?;

        #[cfg(feature = "debug")]
        {
            debug!("Data: {}", data);
        }

        println!("{}", data);

        Ok(())
    }

    fn write_file(&mut self, name: &str) -> anyhow::Result<()> {
        todo!()
    }
}
