from RusticFS import FileSystem
import random

fs = FileSystem()
fs.format()

fs.create_dir("dir1")
fs.change_dir("dir1")

# create 51 files in dir1
for i in range(51):
    fs.create_file_with_content(f"file{i}", f"print('Hello from file{i}')")

fs.change_dir("..")
fs.change_permissions("dir1", "7")
fs.list_dir()
fs.change_dir("dir1")
fs.list_dir()

r = random.randint(0, 50)

# read the file with the random number
print(f"Reading file{r}")
fs.read_file(f"file{r}")
print(f"Running file{r}")
fs.execute_py(f"file{r}")

fs.change_dir("..")

# create 51 files in dir2 with content of the size 4096
fs.create_dir("dir2")
fs.change_dir("dir2")
for i in range(51):
    fs.create_file_with_content(f"file{i}", "a" * 8176)

fs.list_dir()

