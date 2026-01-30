use pyo3::prelude::*;
use tracing::Level;

#[pyclass(name = "Level")]
#[derive(Clone, Copy)]
pub(crate) enum PyLevel {
    #[pyo3(name = "TRACE")]
    Trace,
    #[pyo3(name = "DEBUG")]
    Debug,
    #[pyo3(name = "INFO")]
    Info,
    #[pyo3(name = "WARN")]
    Warn,
    #[pyo3(name = "ERROR")]
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
            PyLevel::Trace => Level::TRACE,
            PyLevel::Debug => Level::DEBUG,
            PyLevel::Info => Level::INFO,
            PyLevel::Warn => Level::WARN,
            PyLevel::Error => Level::ERROR,
        }
    }
}
