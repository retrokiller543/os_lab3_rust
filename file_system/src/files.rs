#![allow(unused_variables)]

use std::io;
use std::io::BufRead;
use std::ops::Add;
use std::path::Path;

#[cfg(feature = "debug")]
use log::{debug, trace};
use path_absolutize::*;
use serde_derive::{Deserialize, Serialize};

use crate::dir_entry::{DirEntry, FileType};
use crate::errors::{FSError, FileError};
use crate::traits::File;
use crate::utils::fixed_str::FixedString;
use crate::utils::path_handler::{absolutize_from, split_path};
use crate::FileSystem;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FileData {
    pub(crate) data: Vec<u8>,
}

impl FileData {
    pub fn new(data: Vec<u8>) -> Self {
        FileData { data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get_size(&self) -> usize {
        let serialized = bincode::serialize(&self).unwrap();
        serialized.len()
    }
}

impl From<&str> for FileData {
    fn from(data: &str) -> Self {
        FileData {
            data: data.as_bytes().to_vec(),
        }
    }
}

impl From<String> for FileData {
    fn from(data: String) -> Self {
        FileData {
            data: data.as_bytes().to_vec(),
        }
    }
}

impl Add for FileData {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut data = self.data;
        data.extend(other.data);
        FileData { data }
    }
}

impl File for FileSystem {
    /// # Create a file in the current directory
    fn create_file(&mut self, path: &str) -> anyhow::Result<()> {
        let abs_path = absolutize_from(path, &self.curr_block.path);
        let (parent, name) = split_path(abs_path.clone());

        #[cfg(feature = "debug")]
        {
            debug!("Path: {}", path);
            debug!("Abs path: {}", abs_path);
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
            if entry.name == name.clone().into() {
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
        data.pop(); // Remove the last newline character

        #[cfg(feature = "debug")]
        {
            debug!("Data: {}", data);
        }

        let file_data = FileData::from(data);

        // find the first free block
        let blk_num = self.get_free_block()?;

        #[cfg(feature = "debug")]
        {
            trace!("Writing file data");
            debug!("Free block: {}", blk_num);
            debug!("Data size on disk: {}", file_data.get_size());
        }

        self.write_data(&file_data, blk_num)?;

        let entry = DirEntry {
            name: name.into(),
            file_type: FileType::File,
            size: file_data.get_size() as u64,
            blk_num,
        };

        #[cfg(feature = "debug")]
        {
            debug!("Entry: {:?}", entry);
            debug!("New entry size: {}", entry.get_size());
        }

        // update size of the parent block
        self.curr_block.parent_entry.size += entry.size;

        self.curr_block.add_entry(entry)?;

        #[cfg(feature = "debug")]
        {
            trace!("Writing block to disk");
            debug!("Block: {:?}", self.curr_block);
            debug!("Block size on disk: {}", self.curr_block.get_size());
        }

        self.write_curr_blk()?;

        Ok(())
    }

    fn delete_file(&mut self, entry: &DirEntry) -> anyhow::Result<()> {
        self.clear_file_data(entry.blk_num)?;
        self.curr_block.remove_entry(&entry.name)?;

        self.write_curr_blk()?;
        Ok(())
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

    fn append_file(&mut self, source: &str, dest: &str) -> anyhow::Result<()> {
        let src_binding = Path::new(source).absolutize()?;
        let src_path = src_binding.to_str().ok_or(FSError::PathError)?;
        let src_parent = Path::new(&src_path)
            .parent()
            .unwrap()
            .to_str()
            .ok_or(FSError::PathError)?;
        let src_name: FixedString = Path::new(&src_path)
            .file_name()
            .unwrap()
            .to_str()
            .ok_or(FSError::PathError)?
            .into();

        let dest_binding = Path::new(dest).absolutize()?;
        let dest_path = dest_binding.to_str().ok_or(FSError::PathError)?;
        let dest_parent = Path::new(&dest_path)
            .parent()
            .unwrap()
            .to_str()
            .ok_or(FSError::PathError)?;
        let dest_name: FixedString = Path::new(&dest_path)
            .file_name()
            .unwrap()
            .to_str()
            .ok_or(FSError::PathError)?
            .into();

        let new_data: FileData;

        {
            let src_entry = self
                .curr_block
                .get_entry(&src_name)
                .ok_or(FileError::FileNotFound)?;
            let dest_entry = self
                .curr_block
                .get_entry(&dest_name)
                .ok_or(FileError::FileNotFound)?;

            if src_entry.file_type != FileType::File || dest_entry.file_type != FileType::File {
                return Err(FileError::FileIsDirectory.into());
            }

            let src_data = self.read_file_data(src_entry.blk_num)?;
            let dest_data = self.read_file_data(dest_entry.blk_num)?;

            new_data = dest_data + "\n".into() + src_data;

            self.clear_file_data(dest_entry.blk_num)?;
            self.write_data(&new_data, dest_entry.blk_num)?;
        }

        let dest_entry = self
            .curr_block
            .get_entry_mut(&dest_name)
            .ok_or(FileError::FileNotFound)?;

        // update size of the dest entry
        dest_entry.size = new_data.len() as u64;

        self.write_curr_blk()?;

        Ok(())
    }
}
