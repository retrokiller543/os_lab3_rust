mod py_filesystem;

use std::sync::Once;
use chrono::Local;
use env_logger::{Builder, Env};
use pyo3::prelude::*;
use crate::errors::{FileError, FSError, IOHandlerError};
use std::io::Write;

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

macro_rules! logger_builder {
    ($lvl:expr) => {
        {
            Builder::from_env(Env::default().default_filter_or($lvl))
                .format(|buf, record| {
                    writeln!(
                        buf,
                        "[{} {} - {}:{}] {}",
                        Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.level(),
                        record
                            .file()
                            .unwrap_or(record.module_path().unwrap_or("unknown")),
                        record.line().unwrap_or(0),
                        record.args()
                    )
                })
        }
    };
}

/// This function sets up the logger for the application.
/// It uses the `Once` type from the `std::sync` module to ensure that the logger is only set up once.
/// The logger is configured to:
/// - Use the environment variable to set the log level, defaulting to "info" if the variable is not set.
/// - Format the log messages to include the current date and time, the log level, the file and line number where the log was generated, and the log message itself.
/// - Output the log messages to the standard output.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// setup_logger();
/// ```
#[pyfunction]
fn setup_logger(lvl: &str) {
    static START: Once = Once::new();
    START.call_once(|| {
        logger_builder!(lvl)
            .target(env_logger::Target::Stdout)
            .init();
    });
}

#[pyfunction]
fn setup_file_logger(lvl: &str) -> PyResult<()> {
    // Get the current timestamp
    let now = Local::now();
    // Format the timestamp as a string in the desired format
    let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    // Create the log filename with the timestamp
    let log_filename = format!("logs/{}.log", timestamp);
    // Create the log file and directory if needed
    std::fs::create_dir_all("logs")?;

    let file = std::fs::File::create(log_filename)?;

    static START: Once = Once::new();
    START.call_once(|| {
        logger_builder!(lvl)
            .target(env_logger::Target::Pipe(Box::new(file)))
            .init();
    });

    Ok(())
}

#[pyfunction]
fn setup_pyo3_logger() {
    pyo3_log::init();
}

#[pymodule]
#[pyo3(name = "RusticFS")]
fn rustic_fs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<crate::FileSystem>()?;
    m.add_function(wrap_pyfunction!(setup_logger, m)?)?;
    m.add_function(wrap_pyfunction!(setup_file_logger, m)?)?;
    m.add_function(wrap_pyfunction!(setup_pyo3_logger, m)?)?;
    //m.add_class::<rustic_disk::Disk>()?;
    Ok(())
}
