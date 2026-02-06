pub mod file;
pub mod rotation;
pub mod span;
pub mod time;
pub mod to_layer;

use pyo3::prelude::*;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::{
    layer::fmt::{
        file::{LogFile, NonBlocking},
        span::PyFmtSpan,
        time::timer::PyTimer,
    },
    level::PyLevel,
};

#[pyclass]
pub struct FmtLayer {
    log_level: Level,
    file: LogFile,
    format: PyFormat,
    fmt_span: FmtSpan,
    non_blocking: Option<NonBlocking>,
    log_internal_errors: Option<bool>,
    timer: Option<PyTimer>,
    with_ansi: Option<bool>,
    with_file: Option<bool>,
    with_level: Option<bool>,
    with_line_number: Option<bool>,
    with_target: Option<bool>,
    with_thread_ids: Option<bool>,
}

#[pymethods]
impl FmtLayer {
    #[expect(
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
        format = PyFormat::Full,
        fmt_span = Python::attach(|x| {Py::new(x, PyFmtSpan::NONE)}).unwrap(),
        non_blocking = None,
        log_internal_errors = None,
        timer = Some(PyTimer::SYSTEM_TIME),
        with_ansi = None,
        with_file = None,
        with_level = None,
        with_line_number = None,
        with_target = None,
        with_thread_ids = None))]
    fn new(
        py: Python,
        log_level: PyLevel,
        file: LogFile,
        format: PyFormat,
        fmt_span: Py<PyFmtSpan>,
        non_blocking: Option<NonBlocking>,
        log_internal_errors: Option<bool>,
        timer: Option<PyTimer>,
        with_ansi: Option<bool>,
        with_file: Option<bool>,
        with_level: Option<bool>,
        with_line_number: Option<bool>,
        with_target: Option<bool>,
        with_thread_ids: Option<bool>,
    ) -> Self {
        Self {
            log_level: Level::from(log_level),
            file,
            format,
            fmt_span: FmtSpan::from(&*fmt_span.borrow(py)),
            non_blocking,
            log_internal_errors,
            timer,
            with_ansi,
            with_file,
            with_level,
            with_line_number,
            with_target,
            with_thread_ids,
        }
    }
}

#[pyclass(name = "Format", rename_all = "UPPERCASE", from_py_object)]
#[derive(Clone, Copy)]
pub enum PyFormat {
    Full,
    Compact,
    Pretty,
    Json,
}
