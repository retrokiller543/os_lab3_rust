use rustic_disk::Disk;

use crate::prelude::*;
use crate::FileSystem;

#[test]
fn test_copy() -> anyhow::Result<()> {
    let mut fs = FileSystem::new()?;
    fs.format()?;
    fs.create_file_with_content("f1", "Hello, World!")?;
    let t1 = fs.copy_entry("f1", "f2");
    assert!(t1.is_ok());
    Disk::delete_disk()?;
    Ok(())
}

#[test]
// TODO: Make this test pass, it should fail to copy a file to an existing file
fn test_copy_axisting_file() -> anyhow::Result<()> {
    let mut fs = FileSystem::new()?;
    fs.format()?;
    fs.create_file_with_content("f1", "Hello, World!")?;
    fs.create_file_with_content("f2", "Hllllello, World!")?;
    let t5 = fs.copy_entry("f1", "f2");
    assert!(t5.is_err());
    Disk::delete_disk()?;
    Ok(())
}

#[test]
fn test_copy_not_exosting() -> anyhow::Result<()> {
    let mut fs = FileSystem::new()?;
    fs.format()?;
    fs.create_file_with_content("f1", "Hello, World!")?;
    let t1 = fs.copy_entry("f6", "f1");
    assert!(t1.is_err());
    Disk::delete_disk()?;
    Ok(())
}

#[test]
fn test_move() -> anyhow::Result<()> {
    let mut fs = FileSystem::new()?;
    fs.format()?;
    fs.create_file_with_content("f1", "Hello, World!")?;
    let t1 = fs.move_entry("f1", "f2");
    assert!(t1.is_ok());
    Disk::delete_disk()?;
    Ok(())
}

#[test]
// TODO: Make this test pass, it should fail to move a file to an existing file
fn test_move_to_existing_file() -> anyhow::Result<()> {
    let mut fs = FileSystem::new()?;
    fs.format()?;
    fs.create_file_with_content("f1", "Hello, World!")?;
    fs.create_file_with_content("f2", "Hello, World!")?;
    let t1 = fs.move_entry("f1", "f2");
    assert!(t1.is_err());
    Disk::delete_disk()?;
    Ok(())
}

#[test]
fn test_move_not_existing() -> anyhow::Result<()> {
    let mut fs = FileSystem::new()?;
    fs.format()?;
    fs.create_file_with_content("f1", "Hello, World!")?;
    let t1 = fs.move_entry("f6", "f1");
    assert!(t1.is_err());
    Disk::delete_disk()?;
    Ok(())
}

#[test]
fn test_remove() -> anyhow::Result<()> {
    let mut fs = FileSystem::new()?;
    fs.format()?;
    fs.create_file_with_content("f1", "Hello, World!")?;
    let t1 = fs.delete_file("f1");
    assert!(t1.is_ok());
    Disk::delete_disk()?;
    Ok(())
}

#[test]
fn test_remove_not_existing() -> anyhow::Result<()> {
    let mut fs = FileSystem::new()?;
    fs.format()?;
    let t1 = fs.delete_file("f1");
    assert!(t1.is_err());
    Disk::delete_disk()?;
    Ok(())
}

#[test]
fn test_append() -> anyhow::Result<()> {
    let mut fs = FileSystem::new()?;
    fs.format()?;
    fs.create_file_with_content("f1", "Hello, World!")?;
    fs.create_file_with_content("f2", "Hehgeh")?;
    let t1 = fs.append_file("f1", "f2");
    assert!(t1.is_ok());
    Disk::delete_disk()?;
    Ok(())
}