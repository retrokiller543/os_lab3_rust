use pyo3::prelude::*;

use crate::prelude::*;

use std::sync::{Arc, Mutex};

#[pyclass]
#[pyo3(name = "FileSystem")]
pub struct PyFileSystem {
    inner: Arc<Mutex<FileSystem>>,
}

#[pymethods]
impl PyFileSystem {
    #[new]
    pub fn new() -> Self {
        // Instantiate your FileSystem with a default IOHandler
        let fs = FileSystem::new(Box::new(StdIOHandler {}));
        PyFileSystem {
            inner: Arc::new(Mutex::new(match fs {
                Ok(fs) => fs,
                Err(e) => panic!("Error creating FileSystem: {:?}", e),
            })),
        }
    }

    pub fn set_std_io_handler(&mut self) {
        let mut fs = self.inner.lock().unwrap();
        fs.io_handler = Box::new(StdIOHandler {});
    }

    // Example method to invoke write operation
    pub fn write(&self, content: String) -> PyResult<()> {
        let mut fs = self.inner.lock().unwrap();
        fs.io_handler
            .write(content)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))
    }

    // format
    pub fn format(&self) -> PyResult<()> {
        let mut fs = self.inner.lock().unwrap();
        fs.format()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))
    }

    // create_file and create_dir
    pub fn create_file(&self, name: String) -> PyResult<()> {
        let mut fs = self.inner.lock().unwrap();
        println!("Enter data to write to file (end with an empty line): ");
        fs.create_file_stdio(&name)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))
    }

    pub fn create_dir(&self, name: String) -> PyResult<()> {
        let mut fs = self.inner.lock().unwrap();
        fs.create_dir(&name)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))
    }

    // list_dir
    pub fn list_dir(&self) -> PyResult<()> {
        let mut fs = self.inner.lock().unwrap();
        fs.list_dir()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))
    }

    // read_file
    pub fn read_file(&self, name: String) -> PyResult<()> {
        let mut fs = self.inner.lock().unwrap();
        fs.read_file(&name)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))
    }

    // delete_file and delete_dir
    pub fn remove_entry(&self, name: String) -> PyResult<()> {
        let mut fs = self.inner.lock().unwrap();
        fs.remove_entry(&name)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))
    }

    // move_entry and copy_entry
    pub fn move_entry(&self, source: String, dest: String) -> PyResult<()> {
        let mut fs = self.inner.lock().unwrap();
        fs.move_entry(&source, &dest)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))
    }

    pub fn copy_entry(&self, source: String, dest: String) -> PyResult<()> {
        let mut fs = self.inner.lock().unwrap();
        fs.copy_entry(&source, &dest)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))
    }

    // change_permissions
    pub fn change_permissions(&self, path: String, permissions: String) -> PyResult<()> {
        let mut fs = self.inner.lock().unwrap();
        fs.change_permissions(&path, &permissions)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))
    }

    // append_file
    pub fn append_file(&self, source: String, dest: String) -> PyResult<()> {
        let mut fs = self.inner.lock().unwrap();
        fs.append_file(&source, &dest)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))
    }

    // change directory
    pub fn change_dir(&self, path: String) -> PyResult<()> {
        let mut fs = self.inner.lock().unwrap();
        fs.change_dir(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))
    }

    // print working directory
    pub fn print_working_dir(&self) -> PyResult<()> {
        let mut fs = self.inner.lock().unwrap();
        fs.print_working_dir()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:?}", e)))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{:#?}", self.inner))
    }
}
