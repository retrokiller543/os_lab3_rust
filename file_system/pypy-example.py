from RusticFS import FileSystem

fs = FileSystem()
fs.format()

fs.create_file_with_content("file1", "print('Hello, World!')")
fs.change_permissions("file1", "7")
fs.execute_py("file1") # execute from a file
fs.change_permissions("/", "7")
fs.list_dir()

