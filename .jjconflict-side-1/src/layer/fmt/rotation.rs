use pyo3::prelude::*;
use tracing_appender::rolling::Rotation;

#[pyclass(name = "Rotation")]
#[derive(Clone, Copy)]
pub enum PyRotation {
    #[pyo3(name = "MINUTELY")]
    Minutely,
    #[pyo3(name = "HOURLY")]
    Hourly,
    #[pyo3(name = "DAILY")]
    Daily,
    #[pyo3(name = "WEEKLY")]
    Weekly,
    #[pyo3(name = "NEVER")]
    Never,
}

#[pymethods]
impl PyRotation {
    fn __repr__(&self) -> String {
        format!("{:?}", Rotation::from(*self))
    }
}

impl From<PyRotation> for Rotation {
    fn from(value: PyRotation) -> Rotation {
        match value {
            PyRotation::Minutely => Rotation::MINUTELY,
            PyRotation::Hourly => Rotation::HOURLY,
            PyRotation::Daily => Rotation::DAILY,
            PyRotation::Weekly => Rotation::WEEKLY,
            PyRotation::Never => Rotation::NEVER,
        }
    }
}
