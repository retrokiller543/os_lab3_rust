#[test]
fn test_copy() -> anyhow::Result<()> {
    let mut fs = FileSystem::new(Box::new(StdIOHandler))?;
    fs.format()?;
    fs.create_file_with_content("f1", "Hello, World!")?;
    let t1 = fs.copy_entry("f1", "f2");
    assert!(t1.is_ok());
    fs.disk.delete_disk()?;
    Ok(())
}
