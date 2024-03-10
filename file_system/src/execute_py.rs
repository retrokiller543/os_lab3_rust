use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::{FileSystem, READ, EXECUTE};
use crate::dir_entry::FileType;
use crate::errors::FileError;
use crate::utils::check_access_level;
use crate::utils::path_handler::{absolutize_from, split_path};

pub fn run_code(code: String) -> PyResult<()> {
    Python::with_gil(|py| {
        let locals = PyDict::new(py);
        // get globals from the current python environment
        let globals = py.eval("globals()", None, Some(&locals))?;
        // convert the globals to a dictionary using PyTryFrom
        let globals = <PyDict as PyTryFrom>::try_from(globals)?;

        py.run(&code, Some(globals), Some(locals))?;
        Ok(())
    })
}

impl FileSystem {
    pub fn execute_py(&mut self, file_path: &str) -> Result<()> {
        #[cfg(not(PyPy))]
        pyo3::prepare_freethreaded_python();
        #[cfg(PyPy)]
        return Err(FSError::PythonNotSupported.into());
        
        let abs_path = absolutize_from(file_path, &self.curr_block.path);
        let (parent, name) = split_path(abs_path.clone());
        let parent_block = self.traverse_dir(parent)?;

        if !check_access_level(parent_block.parent_entry.access_level, READ) {
            return Err(FileError::NoPermissionToWrite(name).into());
        }

        let entry = parent_block.get_entry(&name.clone().into()).ok_or(FileError::FileNotFound)?;

        if entry.file_type != FileType::File {
            return Err(FileError::FileIsDirectory.into());
        }

        if !check_access_level(entry.access_level, EXECUTE) {
            return Err(FileError::NoPermissionToExecute(name).into());
        }
        
        let data = self.read_file_data(entry.blk_num)?;
        
        run_code(data.into())?;

        Ok(())
    }
}