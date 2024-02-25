use crate::{FatType, FAT_BLK, FileSystem, ROOT_BLK, FAT};
use crate::traits::Format;
use anyhow::Result;
use rustic_disk::Disk;
use rustic_disk::traits::BlockStorage;
use crate::dir_entry::{Block, DirEntry, FileType};

impl Format for FileSystem {
    fn format(&mut self) -> Result<()>{
        // disk should always exist since we handle making a dsk in the constructor
        if Disk::disk_exists() {
            Disk::delete_disk()?;
        }

        let mut fat = FAT::new();

        let blk = Block {
            parent_entry: DirEntry {
                name: "/".to_string(),
                file_type: FileType::Directory,
                ..Default::default()
            },
            blk_num: 0,
            entries: vec![DirEntry::default(); Self::NUM_ENTRIES],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() -> Result<()> {
        let mut fs = FileSystem::new()?;
        fs.format()?;
        assert!(Disk::disk_exists());

        // read the first block and check if it's a directory
        let block: Block = fs.read_blk(0)?;
        assert_eq!(block.parent_entry.file_type, FileType::Directory);

        Ok(())
    }
}