use anyhow::Result;

use crate::errors::FileError;
use crate::traits::DirEntryHandling;
use crate::FileSystem;
use crate::utils::path_handler::{absolutize_from, split_path};

impl DirEntryHandling for FileSystem {
    /// The move function is used to move a file from one directory to another
    fn move_entry(&mut self, source: &str, dest: &str) -> Result<()> {
        let abs_src = absolutize_from(source, &self.curr_block.path);
        let abs_dest = absolutize_from(dest, &self.curr_block.path);

        let (src_parent, src_name) = split_path(abs_src);
        let (dest_parent, dest_name) = split_path(abs_dest);

        // NOTE: this will need to be updated to handle directories

        if let Some(entry) = self.curr_block.get_entry(&src_name.clone().into()) {
            let mut new_entry = entry.clone();
            new_entry.name = dest_name.into();

            self.curr_block.add_entry(new_entry)?;
            self.curr_block.remove_entry(&src_name.into())?;
        } else {
            return Err(FileError::FileNotFound.into());
        }

        self.write_curr_blk()?;

        Ok(())
    }

    /// The copy function is used to copy a file from one directory to another
    fn copy_entry(&mut self, source: &str, dest: &str) -> Result<()> {
        let abs_src = absolutize_from(source, &self.curr_block.path);
        let abs_dest = absolutize_from(dest, &self.curr_block.path);

        let (src_parent, src_name) = split_path(abs_src);
        let (dest_parent, dest_name) = split_path(abs_dest);

        // NOTE: this will need to be updated to handle directories

        if let Some(entry) = self.curr_block.get_entry(&src_name.into()) {
            let mut new_entry = entry.clone();
            new_entry.name = dest_name.into();
            let data = self.read_file_data(new_entry.blk_num)?;
            new_entry.blk_num = self.get_free_block()?;
            self.write_data(&data, new_entry.blk_num)?;
            self.curr_block.add_entry(new_entry)?;
        } else {
            return Err(FileError::FileNotFound.into());
        }

        self.write_curr_blk()?;

        Ok(())
    }
}
