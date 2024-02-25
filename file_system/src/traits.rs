use anyhow::Result;

pub trait Format {
    fn format(&mut self) -> Result<()>;
}

pub trait File {
    fn create_file(&mut self, name: &str) -> Result<()>;
    fn delete_file(&mut self, name: &str) -> Result<()>;
    fn read_file(&self, name: &str) -> Result<()>;
    fn write_file(&mut self, name: &str) -> Result<()>;
}

pub trait Directory {
    fn create_dir(&mut self, name: &str) -> Result<()>;
    fn delete_dir(&mut self, name: &str) -> Result<()>;
    fn list_dir(&self) -> Result<()>;
}

pub trait DirEntryHandling {
    fn move_entry(&self, source: &str, dest: &str) -> Result<()>;
    fn copy_entry(&self, source: &str, dest: &str) -> Result<()>;
}