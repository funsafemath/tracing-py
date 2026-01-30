use pyo3::prelude::*;

#[pyclass(name = "Level")]
#[derive(Clone, Copy)]
pub(crate) struct PyLevel(tracing::Level);

#[pymethods]
impl PyLevel {
    #[classattr]
    pub(crate) const TRACE: Self = Self(tracing::Level::TRACE);

    #[classattr]
    pub(crate) const DEBUG: Self = Self(tracing::Level::DEBUG);

    #[classattr]
    pub(crate) const INFO: Self = Self(tracing::Level::INFO);

    #[classattr]
    pub(crate) const WARN: Self = Self(tracing::Level::WARN);

    #[classattr]
    pub(crate) const ERROR: Self = Self(tracing::Level::ERROR);

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

impl From<PyLevel> for tracing::Level {
    fn from(value: PyLevel) -> Self {
        value.0
    }
}
