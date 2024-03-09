use rustic_disk::Disk;

use crate::prelude::*;
use crate::FileSystem;

#[test]
fn test_create() -> anyhow::Result<()> {
    let mut fs = FileSystem::new(Box::new(StdIOHandler))?;
    fs.format()?;
    let t1 = fs.create_file_with_content("f1", "Hello, World!");
    assert!(t1.is_ok());
    fs.disk.delete_disk()?;
    Ok(())
}

#[test]
fn test_create_large_file() -> anyhow::Result<()> {
    let mut fs = FileSystem::new(Box::new(StdIOHandler))?;
    fs.format()?;
    let t2 = fs.create_file_with_content("f1111", "Hello, World!".repeat(100).as_str());
    assert!(t2.is_ok());
    fs.disk.delete_disk()?;
    Ok(())
}

#[test]
fn test_cat() -> anyhow::Result<()> {
    let mut fs = FileSystem::new(Box::new(StdIOHandler))?;
    fs.format()?;
    fs.create_file_with_content("f1", "Hello, World!")?;
    let t3 = fs.read_file("f1");
    assert!(t3.is_ok());
    let t31 = fs.read_file("f4");
    assert!(t31.is_err());
    fs.disk.delete_disk()?;
    Ok(())
}

#[test]
fn test_long_name() -> anyhow::Result<()> {
    let mut fs = FileSystem::new(Box::new(StdIOHandler))?;
    fs.format()?;
    let t4 = fs.create_file_with_content(
        "AbcdefghijAbcdefghijAbcdefghijAbcdefghijAbcdefghijAbcde",
        "Hello, World!",
    );
    assert!(t4.is_ok());
    let t41 = fs.create_file_with_content(
        "AbcdefghijAbcddefghijAbcdefghijAbcdefghijAbcdefghijAbcde",
        "Hello, World!",
    );
    assert!(t41.is_err());
    fs.disk.delete_disk()?;
    Ok(())
}

#[test]
// Max number of files is 52, adding one more gives an error
fn test_nr_of_files() -> anyhow::Result<()> {
    let mut fs = FileSystem::new(Box::new(StdIOHandler))?;
    fs.format()?;
    for i in 0..52 {
        let t = fs.create_file_with_content(format!("f{}", i).as_str(), "Hello!");
        dbg!(&i);
        assert!(t.is_ok());
    }
    fs.disk.delete_disk()?;
    Ok(())
}
