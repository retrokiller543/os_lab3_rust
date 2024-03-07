use crate::prelude::Input;

#[cfg(test)]
mod path_tests;
#[cfg(test)]
mod task1;
mod task2;
mod task3;

#[derive(Debug)]
pub(crate) struct MockInput {
    pub(crate) input: String,
}

impl MockInput {
    pub(crate) fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
        }
    }
}

impl Input for MockInput {
    fn read_lines(&mut self) -> anyhow::Result<String> {
        Ok(self.input.clone())
    }
}

#[cfg(test)]
mod tests {
    use rustic_disk::Disk;

    use crate::dir_entry::{DirBlock, DirEntry, FileType};
    use crate::utils::fixed_str::FixedString;
    use crate::FileSystem;

    #[test]
    fn dir_entry_max_name_length_serialization_size() {
        let name = "a".repeat(56); // Generates a string with 55 'a' characters
        let dir_entry = DirEntry {
            name: FixedString::from(name),
            file_type: FileType::File,
            size: 0,
            blk_num: 0,
            access_level: 0,
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
                access_level: 0,
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

    use crate::prelude::*;
    use crate::FileSystem;

    #[test]
    fn create_file_inside_dir() -> anyhow::Result<()> {
        let mut fs = FileSystem::new(Box::new(StdIOHandler))?;
        fs.format()?;
        fs.create_dir("d1")?;
        fs.create_dir("d1/d2")?;
        let results = fs.create_file_with_content("d2/f1", "Hello, World!");
        assert!(results.is_err()); // We should not find the directory d2
        fs.disk.delete_disk()?;
        Ok(())
    }

    #[test]
    fn test_nested_append() -> anyhow::Result<()> {
        let mut fs = FileSystem::new(Box::new(StdIOHandler))?;
        fs.format()?;
        fs.create_dir("d1")?;
        fs.create_file_with_content("d1/f1", "Hello, World!")?;
        fs.create_file_with_content("f2", "Hello, World!")?;
        fs.change_dir("d1")?;
        let result = fs.append_file("f1", "../f2");
        assert!(result.is_ok());
        fs.disk.delete_disk()?;
        Ok(())
    }
}
