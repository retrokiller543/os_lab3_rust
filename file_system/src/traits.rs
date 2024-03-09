use anyhow::Result;
use std::fmt::Debug;

pub trait Format {
    fn format(&mut self) -> Result<()>;
}

pub trait InputConstructor {
    fn new(io: Box<dyn IOHandler<Input = String, Output = String>>) -> Self;
}

pub trait Input {
    fn read_lines(&mut self) -> Result<String>;
}

pub trait File {
    fn create_file<T>(&mut self, name: &str, input_source: &mut T) -> Result<()>
    where
        T: Input + Debug;
    // this is only since we need to test the file system, and we need to create a file with content
    fn create_file_with_content(&mut self, name: &str, content: &str) -> Result<()>;
    fn create_file_stdio(&mut self, name: &str) -> Result<()>;
    fn delete_file(&mut self, path: &str) -> Result<()>;
    fn read_file(&mut self, name: &str) -> Result<()>;
    fn append_file(&mut self, source: &str, dest: &str) -> Result<()>;
}

pub trait Directory {
    fn create_dir(&mut self, name: &str) -> Result<()>;
    fn delete_dir(&mut self, path: &str) -> Result<()>;
    fn list_dir(&mut self) -> Result<()>;
}

pub trait DirEntryHandling {
    fn move_entry(&mut self, source: &str, dest: &str) -> Result<()>;
    fn copy_entry(&mut self, source: &str, dest: &str) -> Result<()>;
}

pub trait Permissions {
    fn change_permissions(&mut self, path: &str, permissions: &str) -> Result<()>;
}

pub trait IOHandlerClone {
    fn clone_box(&self) -> Box<dyn IOHandler<Input = String, Output = String> + Send + Sync>;
}

impl<T> IOHandlerClone for T
where
    T: 'static + IOHandler<Input = String, Output = String> + Clone + Send + Sync,
{
    fn clone_box(&self) -> Box<dyn IOHandler<Input = String, Output = String> + Send + Sync> {
        Box::new(self.clone())
    }
}

pub trait IOHandler
where
    Self: Debug + IOHandlerClone,
{
    type Input;
    type Output;

    fn read(&mut self) -> Result<Self::Input>;
    fn write(&mut self, content: Self::Output) -> Result<()>;
}
