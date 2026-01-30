pub(crate) mod span;
pub(crate) mod to_layer;


use pyo3::{FromPyObject, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use tracing_subscriber::fmt::format::FmtSpan;

use crate::{layer::fmt::span::PyFmtSpan, level::PyLevel};

#[pyclass]
pub(crate) struct FmtLayer {
    log_internal_errors: Option<bool>,
    with_ansi: Option<bool>,
    with_file: Option<bool>,
    with_level: Option<bool>,
    with_line_number: Option<bool>,
    with_target: Option<bool>,
    with_thread_ids: Option<bool>,
    with_max_level: crate::level::PyLevel,
    without_time: bool,
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
    #[pyo3(signature = (*, 
           log_internal_errors = None,
           with_ansi = None, 
           with_file = None, 
           with_level = None,
           with_line_number = None,
           with_target = None,
           with_thread_ids = None, 
           with_max_level = Python::attach(|x| {Py::new(x, PyLevel::INFO)}).unwrap(), 
           without_time = false,
           fmt_span = Python::attach(|x| {Py::new(x, PyFmtSpan::NONE)}).unwrap(),
           format = Python::attach(|x| {Py::new(x, Format::Full)}).unwrap(), 
           // tracing uses stdout by default, not sure why
           // https://github.com/tokio-rs/tracing/issues/2492
           file = LogFile::Stdout,
           non_blocking = None ))]
    fn new(
        py: Python,
        log_internal_errors: Option<bool>,
        with_ansi: Option<bool>,
        with_file: Option<bool>,
        with_level: Option<bool>,
        with_line_number: Option<bool>,
        with_target: Option<bool>,
        with_thread_ids: Option<bool>,
        with_max_level: Py<crate::level::PyLevel>,
        without_time: bool,
        fmt_span: Py<PyFmtSpan>,
        format: Py<Format>,
        file: LogFile,
        non_blocking: Option<NonBlocking>
    ) -> PyResult<Self> {
       Ok( Self {
            log_internal_errors,
            with_ansi,
            with_file,
            with_level,
            with_line_number,
            with_target,
            with_thread_ids,
            with_max_level: *with_max_level.borrow(py),
            without_time,
            fmt_span: FmtSpan::from(&*fmt_span.borrow(py)),
            format: *format.borrow(py),
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


