use pyo3::{exceptions::PyTypeError, prelude::*};
use tracing_appender::rolling::Rotation;

use crate::layer::fmt::rotation::PyRotation;
pub enum LogFile {
    Stdout,
    Stderr,
    Path(String),
    Rolling(PyRollingLog),
}

impl<'a, 'py> FromPyObject<'a, 'py> for LogFile {
    type Error = PyErr;

    fn extract(obj: pyo3::Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(log_file) = obj.cast::<PyLogFile>() {
            Ok(match *log_file.borrow() {
                PyLogFile::Stdout => Self::Stdout,
                PyLogFile::Stderr => Self::Stderr,
            })
        } else if let Ok(rolling) = obj.extract::<PyRollingLog>() {
            Ok(Self::Rolling(rolling))
        } else if let Ok(path) = obj.extract::<String>() {
            Ok(Self::Path(path))
        } else {
            Err(PyTypeError::new_err(
                "expected a File, a RollingLog, or a string",
            ))
        }
    }
}

#[pyclass(name = "File")]
#[derive(Clone)]
pub enum PyLogFile {
    #[pyo3(name = "STDOUT")]
    Stdout,
    #[pyo3(name = "STDERR")]
    Stderr,
}

#[pyclass(name = "RollingLog")]
#[derive(Clone)]
pub struct PyRollingLog {
    pub dir: String,
    pub prefix: String,
    pub rotation: Rotation,
}

#[pymethods]
impl PyRollingLog {
    #[new]
    fn new(dir: String, prefix: String, rotation: PyRotation) -> Self {
        Self {
            dir,
            prefix,
            rotation: rotation.into(),
        }
    }
}

#[pyclass]
#[derive(Clone, Copy)]
pub enum NonBlocking {
    #[pyo3(name = "LOSSY")]
    Lossy,
    #[pyo3(name = "COMPLETE")]
    Complete,
}
