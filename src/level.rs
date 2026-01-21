use pyo3::pyclass;

#[derive(Clone, Debug)]
#[pyclass]
pub(crate) enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
