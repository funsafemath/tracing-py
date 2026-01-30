pub(crate) mod span;
pub(crate) mod to_layer;


use pyo3::{Py, Python, pyclass, pymethods};
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
    file: Py<OutFile>,
}

#[pymethods]
impl FmtLayer {
    #[allow(
        clippy::too_many_arguments,
        reason = "how else am I supposed to implement a python constructor?"
    )]
    #[new]
    #[pyo3(signature = (*, log_internal_errors = None, with_ansi = None, with_file = None, with_level = None,
with_line_number = None, with_target = None, with_thread_ids = None, 
with_max_level = Python::attach(|x| {Py::new(x, PyLevel::INFO)}).unwrap(), 
without_time = false,
fmt_span = Python::attach(|x| {Py::new(x, PyFmtSpan::NONE)}).unwrap(),
format = Python::attach(|x| {Py::new(x, Format::Full)}).unwrap(), 
// tracing uses stdout by default, not sure why
// https://github.com/tokio-rs/tracing/issues/2492
file = Python::attach(|x| {Py::new(x, OutFile::Stdout())}).unwrap() ))]
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
        file: Py<OutFile>,
    ) -> Self {
        Self {
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
        }
    }
}

#[pyclass]
#[derive(Clone, Copy)]
pub(crate) enum Format {
    Full,
    Compact,
    Pretty,
    Json,
}

#[pyclass(name = "File")]
#[derive(Clone)]
pub(crate) enum OutFile {
    Stdout(),
    Stderr(),
    Path(String),
}

#[pymethods]
impl OutFile {
    #[classattr]
    const STDOUT: Self = Self::Stdout();

    #[classattr]
    const STDERR: Self = Self::Stderr();

    #[new]
    fn new(path: String) -> Self {
        Self::Path(path)
    }
}

