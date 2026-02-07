use pyo3::prelude::*;
use tracing_appender::rolling::Rotation;

#[pyclass(name = "Rotation", rename_all = "UPPERCASE", from_py_object)]
#[derive(Clone, Copy)]
pub enum PyRotation {
    Minutely,
    Hourly,
    Daily,
    Weekly,
    Never,
}

#[pymethods]
impl PyRotation {
    fn __repr__(&self) -> String {
        format!("{:?}", Rotation::from(*self))
    }
}

impl From<PyRotation> for Rotation {
    fn from(value: PyRotation) -> Self {
        match value {
            PyRotation::Minutely => Self::MINUTELY,
            PyRotation::Hourly => Self::HOURLY,
            PyRotation::Daily => Self::DAILY,
            PyRotation::Weekly => Self::WEEKLY,
            PyRotation::Never => Self::NEVER,
        }
    }
}
