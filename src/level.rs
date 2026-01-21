use pyo3::prelude::*;

#[pyclass]
pub(crate) struct Level(tracing::Level);

#[pymethods]
impl Level {
    #[classattr]
    const TRACE: Self = Self(tracing::Level::TRACE);

    #[classattr]
    const DEBUG: Self = Self(tracing::Level::DEBUG);

    #[classattr]
    const INFO: Self = Self(tracing::Level::INFO);

    #[classattr]
    const WARN: Self = Self(tracing::Level::WARN);

    #[classattr]
    const ERROR: Self = Self(tracing::Level::ERROR);

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

impl From<Level> for tracing::Level {
    fn from(value: Level) -> Self {
        value.0
    }
}
