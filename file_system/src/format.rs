use anyhow::Result;
use logger_macro::trace_log;

use rustic_disk::traits::BlockStorage;
use rustic_disk::Disk;

use crate::dir_entry::{DirBlock, DirEntry, FileType};
use crate::traits::Format;
use crate::{FatType, FileSystem, FAT, FAT_BLK, ROOT_BLK, READ_WRITE_EXECUTE};

impl Format for FileSystem {
    #[trace_log]
    fn format(&mut self) -> Result<()> {
        // disk should always exist since we handle making a dsk in the constructor
        if Disk::disk_exists() {
            self.disk.delete_disk()?;
        }

        let mut fat = FAT::new();

        let blk = DirBlock {
            path: "/".to_string(),
            parent_entry: DirEntry {
                name: "/".into(),
                file_type: FileType::Directory,
                access_level: READ_WRITE_EXECUTE,
                ..Default::default()
            },
            blk_num: 0,
            entries: vec![DirEntry::default(); Self::num_entries()],
        };

        self.disk = Disk::new()?;
        self.disk.write_block(ROOT_BLK as usize, &blk)?;
        self.curr_block = blk;
        fat[ROOT_BLK as usize] = FatType::EOF;
        fat[FAT_BLK as usize] = FatType::EOF;
        self.disk.write_block(FAT_BLK as usize, &fat)?;
        self.fat = fat;

        Ok(())
    }
}
