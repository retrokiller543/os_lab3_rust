use crate::FileSystem;
use crate::traits::Directory;
use anyhow::Result;
use crate::dir_entry::FileType;

impl Directory for FileSystem {
    fn create_dir(&mut self, name: &str) -> Result<()> {
        unimplemented!()
    }

    fn delete_dir(&mut self, name: &str) -> Result<()> {
        unimplemented!()
    }

    fn list_dir(&self) -> Result<()> {
        // Print column headers
        println!("{:20} {:10} {:15} {:10}", "Name", "Type", "Size (Bytes)", "Block Number");

        for entry in &self.curr_block.entries {
            if !entry.name.is_empty() {
                let entry_type = match entry.file_type {
                    FileType::File => "File",
                    FileType::Directory => "Directory",
                };
                // Format and print each entry according to the column widths
                println!("{:20} {:10} {:15} {:10}", entry.name, entry_type, entry.size, entry.blk_num);
            }
        }

        Ok(())
    }
}