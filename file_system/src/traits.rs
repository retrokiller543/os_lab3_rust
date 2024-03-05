use anyhow::Result;
use std::fmt::Debug;

use crate::dir_entry::DirEntry;

pub trait Format {
    fn format(&mut self) -> Result<()>;
}

pub trait Input {
    fn read_lines(&self) -> String;
}

pub trait File {
    fn create_file<T: Input + Debug>(&mut self, name: &str, input_source: &T) -> Result<()>;
    // this is only since we need to test the file system, and we need to create a file with content
    fn create_file_with_content(&mut self, name: &str, content: &str) -> Result<()>;
    fn create_file_stdio(&mut self, name: &str) -> Result<()>;
    fn delete_file(&mut self, path: &str) -> Result<()>;
    fn read_file(&self, name: &str) -> Result<()>;
    fn append_file(&mut self, source: &str, dest: &str) -> Result<()>;
}

pub trait Directory {
    fn create_dir(&mut self, name: &str) -> Result<()>;
    fn delete_dir(&mut self, path: &str) -> Result<()>;
    fn list_dir(&self) -> Result<()>;
}

pub trait DirEntryHandling {
    fn move_entry(&mut self, source: &str, dest: &str) -> Result<()>;
    fn copy_entry(&mut self, source: &str, dest: &str) -> Result<()>;
}
