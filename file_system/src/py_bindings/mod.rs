mod definitions;

use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "RusticFS")]
fn rustic_fs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<crate::py_bindings::definitions::PyFileSystem>()?;
    m.add_class::<rustic_disk::Disk>()?;
    Ok(())
}
