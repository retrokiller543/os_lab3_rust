from typing import Optional, List, Any
from dataclasses import dataclass

"""
Your documentation link or description here.
"""

class IOHandler:
    pass

@dataclass
class FileData:
    # Update with actual fields based on your Rust structure
    content: bytes

@dataclass
class DirEntry:
    # Assuming this matches the Rust definition, update with actual fields
    name: str
    blk_num: int
    # Add additional fields as necessary

@dataclass
class DirBlock:
    # Update with actual fields based on your Rust structure
    entries: List[DirEntry]

@dataclass
class FileSystem:
    """
    Docs for FileSystem can be found at https://jkdshfhjsd.com not an actual link yet!!!
    """

    _disk: Disk
    _curr_dir: DirBlock
    _fat: FAT
    _io_handler: IOHandler

    def __init__(self) -> None:
        """
        Creates a new `FileSystem` object.
        """
        ...

    def update_curr_dir(self) -> None:
        """
        Updates the current directory to reflect any changes.
        """
        ...

    def write_curr_blk(self) -> None:
        """
        Writes the current block back to the disk.
        """
        ...

    def get_free_block(self) -> int:
        """
        Finds a free block and returns its index.
        """
        ...

    def update_fat(self, blk: int, next_blk: Optional[int] = None) -> None:
        """
        Updates the File Allocation Table (FAT) entry for a given block.
        """
        ...

    def read_file_data(self, start_blk: int) -> FileData:
        """
        Reads file data starting from a specific block.
        """
        ...

    def clear_file_data(self, start_blk: int) -> None:
        """
        Clears the data of a file starting from a specific block.
        """
        ...

    def remove_dir_data(self, dir_entry: DirEntry, path: str) -> None:
        """
        Removes directory data based on the given directory entry and path.
        """
        ...

    def read_blk(self, blk: int) -> DirBlock:
        """
        Reads a directory block from a specified block number.
        """
        ...

    def read_dir_block(self, entry: DirEntry) -> DirBlock:
        """
        Reads a directory block based on the given directory entry.
        """
        ...

    def write_dir_block(self, block: DirBlock) -> None:
        """
        Writes a directory block to the disk.
        """
        ...

    def update_dir(self, entry: DirBlock, path: str) -> None:
        """
        Updates a directory based on the given directory block and path.
        """
        ...

    def traverse_dir(self, path: str) -> DirBlock:
        """
        Traverses the directory structure and returns the directory block for the specified path.
        """
        ...

    def get_all_dirs(self, path: str) -> List[DirBlock]:
        """
        Retrieves all directories under the specified path.
        """
        ...

    def change_dir(self, path: str) -> None:
        """
        Changes the current working directory to the specified path.
        """
        ...

    def print_working_dir(self) -> None:
        """
        Prints the current working directory.
        """
        ...

    def format(self) -> None:
        """
        Formats the filesystem, erasing all data and resetting the system.
        """
        ...

    def create_file(self, path: str) -> None:
        """
        Creates a new file at the specified path.
        """
        ...

    def create_file_with_content(self, path: str, content: str) -> None:
        """
        Creates a new file with the given content at the specified path.
        """
        ...

    def create_dir(self, path: str) -> None:
        """
        Creates a new directory at the specified path.
        """
        ...

    def remove_entry(self, path: str) -> None:
        """
        Removes a file or directory at the specified path.
        """
        ...

    def read_file(self, path: str) -> None:
        """
        Reads the content of a file at the specified path.
        """
        ...

    def append_file(self, source: str, dest: str) -> None:
        """
        Appends the content of the source file to the destination file.
        """
        ...

    def list_dir(self) -> None:
        """
        Lists all entries in the current directory.
        """
        ...

    def change_permissions(self, path: str, access_level: str) -> None:
        """
        Changes the permissions of the file or directory at the specified path.
        """
        ...

    def copy_entry(self, source: str, dest: str) -> None:
        """
        Copies a file or directory from the source to the destination path.
        """
        ...

    def move_entry(self, source: str, dest: str) -> None:
        """
        Moves a file or directory from the source to the destination path.
        """
        ...

    def execute_py(self, file_path: str) -> None:
        """
        :param file_path: str:
        :return:
        """


@dataclass
class Disk:
    diskfile: Any

    def __init__(self) -> None: ...

class FileType:
    File = ...
    Directory = ...

    def __init__(self) -> None: ...
    # If there are any methods related to FileType, add them here


class FatType:
    pass


class FAT:
    # Assuming a constructor and methods to manipulate the FAT vector
    def __init__(self, fat_entries: List['FatType']) -> None: ...
    # Additional methods here

def setup_logger(log_level: str) -> None:
    """
    Sets up the logger for the filesystem.
    This logger will write to the console.

    Errors:
        - Can only be called once.
    """
    ...

def setup_file_logger(log_level: str) -> None:
    """
    Sets up the logger for the filesystem.
    This logger will write to a file.

    Errors:
        - Can only be called once.
    """
    ...

def setup_pyo3_logger() -> None:
    """
    This sets up the logger to integrate with python logging via pyo3_log crate.
    """
    ...