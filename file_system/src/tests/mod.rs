#[cfg(test)]
mod path_tests;

#[cfg(test)]
mod tests {
    use rustic_disk::Disk;

    use crate::dir_entry::{DirBlock, DirEntry, FileType};
    use crate::FileSystem;
    use crate::utils::fixed_str::FixedString;

    #[test]
    fn dir_entry_max_name_length_serialization_size() {
        let name = "a".repeat(56); // Generates a string with 55 'a' characters
        let dir_entry = DirEntry {
            name: FixedString::from(name),
            file_type: FileType::File,
            size: 0,
            blk_num: 0,
        };

        let serialized = bincode::serialize(&dir_entry).expect("Failed to serialize DirEntry");
        println!("Serialized DirEntry size: {}", serialized.len());

        // Adjust the expected size according to your serialization results
        assert!(
            serialized.len() <= Disk::BLOCK_SIZE,
            "DirEntry exceeds single block size"
        );
    }

    #[test]
    fn block_max_entries_fit() {
        let max_entry = DirEntry::gen_max_size_entry();
        let serialized_size = bincode::serialize(&max_entry)
            .expect("Failed to serialize")
            .len();
        let entries_fit = Disk::BLOCK_SIZE / serialized_size; // Assuming no additional overhead for simplicity

        println!("Entries that fit in a block: {}", entries_fit);

        // Verify against your calculated or expected value
        assert!(entries_fit > 0, "No entries fit in a block");
    }

    #[test]
    fn add_entries_in_block() {
        let mut block = DirBlock::default();
        let max_entry = DirEntry::gen_max_size_entry();
        block.entries = vec![DirEntry::default(); FileSystem::num_entries()];

        let mut size = block.get_size();
        assert!(size <= Disk::BLOCK_SIZE, "Block exceeds single block size");

        for _ in 0..FileSystem::num_entries() {
            block.add_entry(max_entry.clone()).unwrap();
            let new_size = block.get_size();
            assert_eq!(
                new_size, size,
                "Block size should not change after removing entries"
            );
            size = new_size;
        }

        if block
            .entries
            .iter()
            .any(|entry| *entry == DirEntry::default())
        {
            panic!("Block should be full");
        }

        assert_eq!(
            block.entries.len(),
            FileSystem::num_entries(),
            "Block should be full"
        );
    }

    #[test]
    fn add_real_entries() {
        let mut block = DirBlock::default();
        block.entries = vec![DirEntry::default(); FileSystem::num_entries()];

        let mut size = block.get_size();
        assert!(size <= Disk::BLOCK_SIZE, "Block exceeds single block size");

        for i in 1..(FileSystem::num_entries() + 1) {
            let entry = DirEntry {
                name: FixedString::from(format!("f{}", &i)),
                file_type: FileType::File,
                size: 20,
                blk_num: i as u16,
            };
            block.add_entry(entry).unwrap();
            let new_size = block.get_size();
            assert_eq!(
                new_size, size,
                "Block size should not change after adding entries"
            );
            size = new_size;
        }
    }

    #[test]
    fn remove_entries_in_block() {
        let mut block = DirBlock::default();
        let max_entry = DirEntry::gen_max_size_entry();
        block.entries = vec![max_entry.clone(); FileSystem::num_entries()];

        let mut size = block.get_size();
        for _ in 0..FileSystem::num_entries() {
            block.remove_entry(&max_entry.name).unwrap();
            let new_size = block.get_size();
            assert_eq!(
                new_size, size,
                "Block size should not change after removing entries"
            );
            size = new_size;
        }
    }
}

#[cfg(test)]
mod generic_tests {
    use rustic_disk::Disk;

    use crate::dir_entry::{DirEntry, FileType};
    use crate::FileSystem;

    #[test]
    fn test_file_system_creation() {
        let fs = FileSystem::new().unwrap();
        assert_eq!(fs.curr_block.blk_num, 0);
        Disk::delete_disk().unwrap();
    }

    #[test]
    fn test_file_system_write_curr_blk() {
        let mut fs = FileSystem::new().unwrap();
        let entry = DirEntry {
            name: "test".into(),
            file_type: FileType::File,
            size: 0,
            blk_num: 0,
        };
        fs.curr_block.entries.push(entry.clone());
        //fs.curr_block.entries[0] = entry.clone();
        fs.write_curr_blk().unwrap();
        let read_block = fs.read_blk(0).unwrap();
        assert_eq!(read_block.entries[0].name, entry.name);
        Disk::delete_disk().unwrap();
    }
}

#[cfg(test)]
mod format_tests {
    use anyhow::Result;

    use rustic_disk::Disk;

    use crate::dir_entry::{DirBlock, FileType};
    use crate::FileSystem;
    use crate::prelude::Format;

    #[test]
    fn test_format() -> Result<()> {
        let mut fs = FileSystem::new()?;
        fs.format()?;
        assert!(Disk::disk_exists());

        // read the first block and check if it's a directory
        let block: DirBlock = fs.read_blk(0)?;
        assert_eq!(block.parent_entry.file_type, FileType::Directory);

        Ok(())
    }
}
