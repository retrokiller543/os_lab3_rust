from typing import List


"""
dksjlhksjdfhghjkfd
"""
class FileSystem:
    """
    Doc str for FileSystem class from pyi file
    """

    disk: Disk  # Type might need refinement based on actual implementation
    curr_block: DirBlock
    fat: FAT

    def __init__(self) -> None: ...

    def __str__(self) -> str: ...

    @staticmethod
    def new() -> "FileSystem": ...

    def set_std_io_handler(self) -> None: ...

    def write(self, content: str) -> bool: ...

    def format(self) -> bool: ...

    def create_file(self, name: str) -> bool: ...

    def create_dir(self, name: str) -> bool: ...

    def list_dir(self) -> bool: ...

    def read_file(self, name: str) -> bool: ...

    def remove_entry(self, name: str) -> bool: ...

    def change_dir(self, name: str) -> bool: ...

    def move_entry(self, source: str, dest: str) -> bool: ...

    def copy_entry(self, source: str, dest: str) -> bool: ...

    def change_permissions(self, path: str, permissions: str) -> bool: ...

    def append_file(self, source: str, dest: str) -> bool: ...

    def print_working_dir(self) -> bool: ...

class Disk:
    # Assuming getters and setters for diskfile are provided if needed
    def __init__(self) -> None: ...

class DirBlock:
    path: str
    parent_entry: DirEntry
    blk_num: int
    entries: List[DirEntry]

    def __init__(self, path: str, parent_entry: DirEntry, blk_num: int, entries: List[DirEntry]) -> None: ...
    # Additional methods here

class DirEntry:
    name: str
    file_type: FileType
    size: int
    blk_num: int
    access_level: int

    def __init__(self, name: str, file_type: FileType, size: int, blk_num: int, access_level: int) -> None: ...
    # Additional methods here

class FileType:
    File = ...
    Directory = ...

    def __init__(self) -> None: ...
    # If there are any methods related to FileType, add them here

class FAT:
    # Assuming a constructor and methods to manipulate the FAT vector
    def __init__(self, fat_entries: List['FatType']) -> None: ...
    # Additional methods here