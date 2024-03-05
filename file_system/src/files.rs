#![allow(unused_variables)]

use std::fmt::Debug;
use std::io::BufRead;

#[cfg(feature = "debug")]
use log::{debug, trace};

use logger_macro::trace_log;

use crate::dir_entry::{DirEntry, FileType};
use crate::errors::FileError;
use crate::file_data::FileData;
use crate::prelude::Input;
use crate::tests::MockInput;
use crate::traits::File;
use crate::utils::path_handler::{absolutize_from, split_path};
use crate::FileSystem;

pub struct StdinInput;

impl Debug for StdinInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StdinInput")
    }
}

impl Input for StdinInput {
    fn read_lines(&self) -> String {
        let mut data = String::new();
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let input_data = line.expect("Failed to read line");
            if input_data.is_empty() {
                break;
            }
            data.push_str(&input_data);
            data.push('\n');
        }
        data.pop(); // Remove the last newline character if it exists
        data
    }
}

impl File for FileSystem {
    /// # Create a file in the current directory
    ///
    #[trace_log]
    fn create_file<T: Input + Debug>(
        &mut self,
        path: &str,
        input_source: &T,
    ) -> anyhow::Result<()> {
        let abs_path = absolutize_from(path, &self.curr_block.path);
        let (parent, name) = split_path(abs_path.clone());

        #[cfg(feature = "debug")]
        {
            debug!("Path: {}", path);
            debug!("Abs path: {}", abs_path);
            debug!("Parent: {}", parent);
            debug!("Name: {}", name);
        }

        // Controls so that the length isn´t longer than 55 chars
        if name.len() > 55 {
            return Err(FileError::FilenameTooLong.into());
        } else if name.is_empty() {
            return Err(FileError::InvalidFilename(name.to_string()).into());
        }

        let mut parent_block = self.traverse_dir(parent)?;

        // make code to check if file exists and parent exists
        for entry in parent_block.entries.iter() {
            if entry.name == name.clone().into() {
                return Err(FileError::FileAlreadyExists.into());
            }
        }

        // read data from user
        let data = input_source.read_lines();

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
        parent_block.add_entry(entry)?;
        self.update_dir(&mut parent_block, abs_path)?;

        Ok(())
    }

    fn create_file_with_content(&mut self, path: &str, content: &str) -> anyhow::Result<()> {
        let data = MockInput::new(content);
        self.create_file(path, &data)?;
        Ok(())
    }

    fn create_file_stdio(&mut self, name: &str) -> anyhow::Result<()> {
        let data = StdinInput;
        self.create_file(name, &data)?;
        Ok(())
    }

    #[trace_log]
    fn delete_file(&mut self, path: &str) -> anyhow::Result<()> {
        let abs_path = absolutize_from(&path, &self.curr_block.path);
        let (parent, name) = split_path(abs_path.clone());

        let mut parent_block = self.traverse_dir(parent.clone())?;
        let binding = parent_block.clone();
        let entry = binding.get_entry(&name.into()).ok_or(FileError::FileNotFound)?;

        self.clear_file_data(entry.blk_num)?;
        parent_block.remove_entry(&entry.name)?;

        self.fat[entry.blk_num as usize] = crate::fat::FatType::Free;

        self.write_dir_block(&parent_block)?;
        Ok(())
    }

    /// the cat function
    #[trace_log]
    fn read_file(&self, path: &str) -> anyhow::Result<()> {
        let abs_path = absolutize_from(path, &self.curr_block.path);
        let (parent, name) = split_path(abs_path);

        let parent_block = self.traverse_dir(parent.clone())?;

        #[cfg(feature = "debug")]
        {
            debug!("Path: {}", &path);
            debug!("Parent: {}", &parent);
            debug!("Name: {}", &name);
        }

        if name.is_empty() {
            return Err(FileError::InvalidFilename(name.to_string()).into());
        }

        let mut file_entry: &DirEntry = &DirEntry::default();

        for entry in parent_block.entries.iter() {
            if entry.name == name.clone().into() {
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

    /// The append function
    #[trace_log]
    fn append_file(&mut self, source: &str, dest: &str) -> anyhow::Result<()> {
        let abs_src = absolutize_from(source, &self.curr_block.path);
        let abs_dest = absolutize_from(dest, &self.curr_block.path);

        let (src_parent, src_name) = split_path(abs_src);
        let (dest_parent, dest_name) = split_path(abs_dest.clone());

        let src_block = self.traverse_dir(src_parent)?;
        let mut dest_block = self.traverse_dir(dest_parent)?;

        let new_data: FileData;

        {
            let src_entry = src_block
                .get_entry(&src_name.into())
                .ok_or(FileError::FileNotFound)?;
            let dest_entry = dest_block
                .get_entry(&dest_name.clone().into())
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

        let dest_entry = dest_block
            .get_entry_mut(&dest_name.clone().into())
            .ok_or(FileError::FileNotFound)?;

        // update size of the dest entry
        dest_entry.size = new_data.len() as u64;

        self.update_dir(&mut dest_block, abs_dest)?;

        Ok(())
    }
}
