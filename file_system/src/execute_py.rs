use anyhow::Result;

#[cfg(PyPy)]
use crate::errors::FSError;
use crate::FileSystem;

#[cfg(not(PyPy))]
use {
    pyo3::prelude::*,
    pyo3::types::PyDict,
    
    crate::{READ, EXECUTE},
    crate::dir_entry::FileType,
    crate::errors::FileError,
    crate::utils::check_access_level,
    crate::utils::path_handler::{absolutize_from, split_path},
};
use logger_macro::trace_log;

#[cfg(not(PyPy))]
#[trace_log]
pub fn run_code(code: String) -> PyResult<()> {
    Python::with_gil(|py| {
        let locals = PyDict::new(py);
        // get globals from the current python environment
        let globals = py.eval("globals()", None, Some(&locals))?;
        // convert the globals to a dictionary using PyTryFrom
        let globals = <PyDict as PyTryFrom>::try_from(globals)?;

        py.run(&code, None, Some(locals))?;
        Ok(())
    })
}

impl FileSystem {
    #[trace_log]
    pub fn execute_py(&mut self, input: &str) -> Result<()> {
        #[cfg(PyPy)]
        return Err(FSError::PythonNotSupported.into());

        #[cfg(not(PyPy))]
        {
            pyo3::prepare_freethreaded_python();

            let code_to_run: String;
            let name: String;

            // Check if input is enclosed in quotes to signal raw Python code
            if input.starts_with('"') && input.ends_with('"') {
                // It's raw Python code
                code_to_run = input.trim_matches('"').replace("\\n", "\n").to_string();

                // Check if we have execute permissions in the current directory
                let current_dir = &self.curr_block;
                if !check_access_level(current_dir.parent_entry.access_level, EXECUTE) {
                    return Err(FileError::NoPermissionToExecute(current_dir.clone().path).into());
                }
            } else {
                // It's a virtual file path, process it
                let abs_path = absolutize_from(input, &self.curr_block.path);
                let (parent, name_extracted) = split_path(abs_path.clone());
                name = name_extracted; // Save the name for permission checks

                let parent_block = self.traverse_dir(parent)?;

                if !check_access_level(parent_block.parent_entry.access_level, READ) {
                    return Err(FileError::NoPermissionToWrite(name.clone()).into());
                }

                let entry = parent_block.get_entry(&name.clone().into()).ok_or(FileError::FileNotFound)?;

                if entry.file_type != FileType::File {
                    return Err(FileError::FileIsDirectory.into());
                }

                if !check_access_level(entry.access_level, EXECUTE) {
                    return Err(FileError::NoPermissionToExecute(name.clone()).into());
                }

                code_to_run = self.read_file_data(entry.blk_num)?.into();
            }

            // Execute the Python code
            run_code(code_to_run)?;

            Ok(())
        }
    }
}