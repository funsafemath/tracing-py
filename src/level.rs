use pyo3::prelude::*;
use tracing::Level;

#[pyclass(name = "Level", rename_all = "UPPERCASE", from_py_object)]
#[derive(Clone, Copy)]
pub enum PyLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[pymethods]
impl PyLevel {
    fn __repr__(&self) -> String {
        format!("{:?}", Level::from(*self))
    }

    fn __str__(&self) -> String {
        format!("{}", Level::from(*self))
    }
}

impl From<PyLevel> for tracing::Level {
    fn from(value: PyLevel) -> Self {
        match value {
            PyLevel::Trace => Self::TRACE,
            PyLevel::Debug => Self::DEBUG,
            PyLevel::Info => Self::INFO,
            PyLevel::Warn => Self::WARN,
            PyLevel::Error => Self::ERROR,
        }
    }
}
