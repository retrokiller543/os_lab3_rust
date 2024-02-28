use anyhow::Result;
use crate::dir_entry::DirEntry;

pub trait Format {
    fn format(&mut self) -> Result<()>;
}

pub trait File {
    fn create_file(&mut self, name: &str) -> Result<()>;
    fn delete_file(&mut self, entry: &DirEntry) -> Result<()>;
    fn read_file(&self, name: &str) -> Result<()>;
    fn append_file(&mut self, source: &str, dest: &str) -> Result<()>;
}

pub trait Directory {
    fn create_dir(&mut self, name: &str) -> Result<()>;
    fn delete_dir(&mut self, name: &str) -> Result<()>;
    fn list_dir(&self) -> Result<()>;
}

pub trait DirEntryHandling {
    fn move_entry(&mut self, source: &str, dest: &str) -> Result<()>;
    fn copy_entry(&mut self, source: &str, dest: &str) -> Result<()>;
}
