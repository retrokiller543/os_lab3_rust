use pyo3::prelude::*;
use crate::dir_entry::{DirBlock, DirEntry};
use crate::file_data::FileData;
use crate::prelude::*;

macro_rules! py_wrap {
    ($call:expr, $ret:ty) => {{
        match $call {
            Ok(val) => Ok(val),
            Err(e) => {
                let py_err = pyo3::PyErr::new::<pyo3::exceptions::PyException, _>(format!("{}", e));
                Err(py_err)
            }
        }
    }};
    ($call:expr) => {{
        match $call {
            Ok(_) => Ok(()),
            Err(e) => {
                let py_err = pyo3::PyErr::new::<pyo3::exceptions::PyException, _>(format!("{}", e));
                Err(py_err)
            }
        }
    }};
}

#[pymethods]
impl FileSystem {
    #[new]
    pub fn py_new() -> PyResult<Self> {
        py_wrap!(Self::new(Box::new(StdIOHandler)), Self)
    }

    #[pyo3(name = "update_curr_dir")]
    pub fn py_update_curr_dir(&mut self) -> PyResult<()> {
        py_wrap!(self.update_curr_dir())
    }

    #[pyo3(name = "write_curr_blk")]
    pub fn py_write_curr_blk(&self) -> PyResult<()> {
        py_wrap!(self.write_curr_blk())
    }

    #[pyo3(name = "get_free_block")]
    pub fn py_get_free_block(&mut self) -> PyResult<u16> {
        py_wrap!(self.get_free_block(), u16)
    }

    #[pyo3(name = "update_fat", signature = (blk=0, next_blk=None))]
    pub fn py_update_fat(&mut self, blk: u16, next_blk: Option<u16>) -> PyResult<()> {
        py_wrap!(self.update_fat(blk, next_blk))
    }

    #[pyo3(name = "read_file_data")]
    pub fn py_read_file_data(&self, start_blk: u16) -> PyResult<FileData> {
        py_wrap!(self.read_file_data(start_blk), FileData)
    }

    #[pyo3(name = "clear_file_data")]
    pub fn py_clear_file_data(&mut self, start_blk: u16) -> PyResult<()> {
        py_wrap!(self.clear_file_data(start_blk))
    }

    #[pyo3(name = "remove_dir_data")]
    pub fn py_remove_dir_data(&mut self, dir_entry: &DirEntry, path: &str) -> PyResult<()> {
        py_wrap!(self.remove_dir_data(dir_entry, path))
    }

    #[pyo3(name = "read_blk")]
    pub fn py_read_blk(&self, blk: u64) -> PyResult<DirBlock> {
        py_wrap!(self.read_blk(blk), DirBlock)
    }

    #[pyo3(name = "read_dir_block")]
    pub fn py_read_dir_block(&self, entry: &DirEntry) -> PyResult<DirBlock> {
        py_wrap!(self.read_dir_block(entry), DirBlock)
    }

    #[pyo3(name = "write_dir_block")]
    pub fn py_write_dir_block(&self, block: &DirBlock) -> PyResult<()> {
        py_wrap!(self.write_dir_block(block))
    }

    #[pyo3(name = "update_dir")]
    pub fn py_update_dir(&mut self, entry: &mut DirBlock, path: String) -> PyResult<()> {
        py_wrap!(self.update_dir(entry, path))
    }

    #[pyo3(name = "traverse_dir")]
    pub fn py_traverse_dir(&self, path: String) -> PyResult<DirBlock> {
        py_wrap!(self.traverse_dir(path), DirBlock)
    }

    #[pyo3(name = "get_all_dirs")]
    pub fn py_get_all_dirs(&self, path: String) -> PyResult<Vec<DirBlock>> {
        py_wrap!(self.get_all_dirs(path), Vec<DirBlock>)
    }

    // =============================FileSystem Commands=========================
    #[pyo3(name = "change_dir")]
    pub fn py_change_dir(&mut self, path: &str) -> PyResult<()> {
        py_wrap!(self.change_dir(path))
    }

    #[pyo3(name = "print_working_dir")]
    pub fn py_print_working_dir(&mut self) -> PyResult<()> {
        py_wrap!(self.print_working_dir())
    }

    #[pyo3(name = "format")]
    pub fn py_format(&mut self) -> PyResult<()> {
        py_wrap!(self.format())
    }

    #[pyo3(name = "create_file")]
    pub fn py_create_file(&mut self, path: &str) -> PyResult<()> {
        println!("Enter data for file (end with an empty line): {}", path);
        py_wrap!(self.create_file_stdio(path))
    }

    #[pyo3(name = "create_file_with_content")]
    pub fn py_create_file_with_content(&mut self, path: &str, content: &str) -> PyResult<()> {
        py_wrap!(self.create_file_with_content(path, content))
    }

    #[pyo3(name = "create_dir")]
    pub fn py_create_dir(&mut self, path: &str) -> PyResult<()> {
        py_wrap!(self.create_dir(path))
    }

    // removes a file or directory
    #[pyo3(name = "remove_entry")]
    pub fn py_remove_entry(&mut self, path: &str) -> PyResult<()> {
        py_wrap!(self.remove_entry(path))
    }

    #[pyo3(name = "read_file")]
    pub fn py_read_file(&mut self, path: &str) -> PyResult<()> {
        py_wrap!(self.read_file(path))
    }

    #[pyo3(name = "append_file")]
    pub fn py_append_file(&mut self, source: &str, dest: &str) -> PyResult<()> {
        py_wrap!(self.append_file(source, dest))
    }

    #[pyo3(name = "list_dir")]
    pub fn py_list_dir(&mut self) -> PyResult<()> {
        py_wrap!(self.list_dir())
    }

    #[pyo3(name = "change_permissions")]
    pub fn py_change_permissions(&mut self, path: &str, access_level: &str) -> PyResult<()> {
        py_wrap!(self.change_permissions(path, access_level))
    }

    #[pyo3(name = "copy_entry")]
    pub fn py_copy_entry(&mut self, source: &str, dest: &str) -> PyResult<()> {
        py_wrap!(self.copy_entry(source, dest))
    }

    #[pyo3(name = "move_entry")]
    pub fn py_move_entry(&mut self, source: &str, dest: &str) -> PyResult<()> {
        py_wrap!(self.move_entry(source, dest))
    }
    
    //#[cfg(not(PyPy))]
    #[pyo3(name = "execute_py")]
    pub fn py_execute_py(&mut self, file_path: &str) -> PyResult<()> {
        py_wrap!(self.execute_py(file_path))
    }

    // =========================================================================

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:#?}", self))
    }
}