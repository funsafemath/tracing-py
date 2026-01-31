pub(crate) mod span;
pub(crate) mod to_layer;


use pyo3::{FromPyObject, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::{layer::fmt::span::PyFmtSpan, level::PyLevel};

#[pyclass]
pub(crate) struct FmtLayer {
    with_ansi: Option<bool>,
    with_file: Option<bool>,
    with_level: Option<bool>,
    with_line_number: Option<bool>,
    with_target: Option<bool>,
    with_thread_ids: Option<bool>,
    without_time: bool,
    log_internal_errors: Option<bool>,
    log_level: Level,
    fmt_span: FmtSpan,
    format: Format,
    file: LogFile,
    non_blocking: Option<NonBlocking>
}

#[pymethods]
impl FmtLayer {
    #[allow(
        clippy::too_many_arguments,
        reason = "how else am I supposed to implement a python constructor?"
    )]
    #[new]
    #[pyo3(
        signature = (*, 
        log_level = PyLevel::Info, 
        // tracing uses stdout by default, not sure why
        // https://github.com/tokio-rs/tracing/issues/2492
        file = LogFile::Stdout,
        format = Format::Full, 
        fmt_span = Python::attach(|x| {Py::new(x, PyFmtSpan::NONE)}).unwrap(),
        non_blocking = None,
        log_internal_errors = None,
        with_ansi = None, 
        with_file = None, 
        with_level = None,
        with_line_number = None,
        with_target = None,
        with_thread_ids = None, 
        without_time = false))]
    fn new(
        py: Python,
        log_level: PyLevel,
        file: LogFile,
        format: Format,
        fmt_span: Py<PyFmtSpan>,
        non_blocking: Option<NonBlocking>,
        log_internal_errors: Option<bool>,
        with_ansi: Option<bool>,
        with_file: Option<bool>,
        with_level: Option<bool>,
        with_line_number: Option<bool>,
        with_target: Option<bool>,
        with_thread_ids: Option<bool>,
        without_time: bool,
    ) -> PyResult<Self> {
       Ok( Self {
            log_internal_errors,
            with_ansi,
            with_file,
            with_level,
            with_line_number,
            with_target,
            with_thread_ids,
            log_level: Level::from(log_level),
            without_time,
            fmt_span: FmtSpan::from(&*fmt_span.borrow(py)),
            format,
            file,
            non_blocking
        })
    }
}

#[pyclass]
#[derive(Clone, Copy)]
pub(crate) enum Format {
    #[pyo3(name = "FULL")]
    Full,
    #[pyo3(name = "COMPACT")]
    Compact,
    #[pyo3(name = "PRETTY")]
    Pretty,
    #[pyo3(name = "JSON")]
    Json,
}


#[pyclass]
#[derive(Clone, Copy)]
pub(crate) enum NonBlocking {
    #[pyo3(name = "LOSSY")]
    Lossy,
    #[pyo3(name = "COMPLETE")]
    Complete
}

enum LogFile {
    Stdout,
    Stderr,
    Path(String),
}

impl<'a, 'py> FromPyObject<'a, 'py> for LogFile {
    type Error = PyErr;

    fn extract(obj: pyo3::Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        Ok(if let Ok(log_file) = obj.cast::<PyLogFile>() {
            match *log_file.borrow() {
                PyLogFile::Stdout => Self::Stdout,
                PyLogFile::Stderr => Self::Stderr,
            }
        }else {
          let log_file = obj.extract::<String>()?;
          Self::Path(log_file)  
        } )
    }
}


#[pyclass(name = "File")]
#[derive(Clone)]
pub(crate) enum PyLogFile {
    #[pyo3(name = "STDOUT")]
    Stdout,
    #[pyo3(name = "STDERR")]
    Stderr,
}


