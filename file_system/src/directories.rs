use crate::FileSystem;
use crate::traits::Directory;
use anyhow::Result;

impl Directory for FileSystem {
    fn create_dir(&mut self, name: &str) -> Result<()> {
        unimplemented!()
    }

    fn delete_dir(&mut self, name: &str) -> Result<()> {
        unimplemented!()
    }

    fn list_dir(&self) -> Result<()> {
        for entry in &self.curr_block.entries {
            if entry.name != "" {
                println!("{}", entry.name);
            }
        }

        Ok(())
    }
}