mod py_filesystem;

use pyo3::prelude::*;
use crate::errors::{FileError, FSError, IOHandlerError};

impl From<FSError> for PyErr {
    fn from(err: FSError) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyException, _>(format!("{}", err))
    }
}

impl From<PyErr> for FSError {
    fn from(err: PyErr) -> FSError {
        FSError::PyError(err.to_string())
    }
}

impl From<FileError> for PyErr {
    fn from(err: FileError) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyException, _>(format!("{}", err))
    }
}

impl From<PyErr> for FileError {
    fn from(err: PyErr) -> FileError {
        FileError::PyError(err.to_string())
    }
}

impl From<IOHandlerError> for PyErr {
    fn from(err: IOHandlerError) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyException, _>(format!("{}", err))
    }
}

impl From<PyErr> for IOHandlerError {
    fn from(err: PyErr) -> IOHandlerError {
        IOHandlerError::PyError(err.to_string())
    }
}

#[pymodule]
#[pyo3(name = "RusticFS")]
fn rustic_fs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<crate::FileSystem>()?;
    //m.add_class::<rustic_disk::Disk>()?;
    Ok(())
}
