use std::fmt::Debug;

use pyo3::{Bound, Py, Python, pyclass, pymethods};
use tracing_subscriber::{
    Layer, Registry,
    fmt::{self, format::FmtSpan},
};

#[pyclass]
#[derive(Clone)]
pub(crate) enum Format {
    Full,
    Compact,
    Pretty,
    Json,
}

#[pyclass]
pub(crate) struct FmtLayer {
    log_internal_errors: Option<bool>,
    with_ansi: Option<bool>,
    with_file: Option<bool>,
    with_level: Option<bool>,
    with_line_number: Option<bool>,
    with_target: Option<bool>,
    with_thread_ids: Option<bool>,
    with_max_level: Option<crate::level::PyLevel>,
    without_time: bool,
    fmt_span: FmtSpan,
    format: Format,
}

#[pymethods]
impl FmtLayer {
    #[allow(
        clippy::too_many_arguments,
        reason = "how else am I supposed to implement a python constructor?"
    )]
    #[new]
    #[pyo3(signature = (*, log_internal_errors = None, with_ansi = None, with_file = None, with_level = None,
    with_line_number = None, with_target = None, with_thread_ids = None, with_max_level = None, without_time = false,
                fmt_span=Python::attach(|x| {Py::new(x, PyFmtSpan::NONE)}).unwrap(),
format=Python::attach(|x| {Py::new(x, Format::Full)}).unwrap() ))]
    fn new(
        py: Python,
        log_internal_errors: Option<bool>,
        with_ansi: Option<bool>,
        with_file: Option<bool>,
        with_level: Option<bool>,
        with_line_number: Option<bool>,
        with_target: Option<bool>,
        with_thread_ids: Option<bool>,
        with_max_level: Option<Bound<'_, crate::level::PyLevel>>,
        without_time: bool,
        fmt_span: Py<PyFmtSpan>,
        format: Py<Format>,
    ) -> Self {
        Self {
            log_internal_errors,
            with_ansi,
            with_file,
            with_level,
            with_line_number,
            with_target,
            with_thread_ids,
            with_max_level: with_max_level.map(|x| *x.borrow()),
            without_time,
            fmt_span: fmt_span.borrow(py).0.clone(),
            format: format.borrow(py).clone(),
        }
    }
}

impl From<&FmtLayer> for Box<dyn Layer<Registry> + Send + Sync> {
    fn from(value: &FmtLayer) -> Self {
        let FmtLayer {
            log_internal_errors,
            with_ansi,
            with_file,
            with_level,
            with_line_number,
            with_target,
            with_thread_ids,
            // todo impl
            with_max_level,
            without_time,
            fmt_span,
            format,
        } = value;

        // chaining methods would be more elegant, but it requires guessing the default values
        let mut layer: fmt::Layer<Registry> = fmt::layer();

        if let Some(log_internal_errors) = log_internal_errors {
            layer = layer.log_internal_errors(*log_internal_errors);
        }

        if let Some(with_ansi) = with_ansi {
            layer = layer.with_ansi(*with_ansi);
        }

        if let Some(with_file) = with_file {
            layer = layer.with_file(*with_file);
        }

        if let Some(with_level) = with_level {
            layer = layer.with_level(*with_level);
        }

        if let Some(with_line_number) = with_line_number {
            layer = layer.with_line_number(*with_line_number);
        }

        if let Some(with_target) = with_target {
            layer = layer.with_target(*with_target);
        }

        if let Some(with_thread_ids) = with_thread_ids {
            layer = layer.with_thread_ids(*with_thread_ids);
        }

        layer = layer.with_span_events(fmt_span.clone());

        // incredibly ugly, but i didn't find a simple way to do this due to generic parameters
        // i'll think about it later
        match (format, without_time) {
            (Format::Full, true) => Box::new(layer.without_time()),
            (Format::Full, false) => Box::new(layer),
            (Format::Compact, true) => Box::new(layer.compact().without_time()),
            (Format::Compact, false) => Box::new(layer.compact()),
            (Format::Pretty, true) => Box::new(layer.pretty().without_time()),
            (Format::Pretty, false) => Box::new(layer.pretty()),
            (Format::Json, true) => Box::new(layer.json().without_time()),
            (Format::Json, false) => Box::new(layer.json()),
        }
    }
}

#[pyclass(name = "FmtSpan")]
pub(crate) struct PyFmtSpan(FmtSpan);

#[pymethods]
impl PyFmtSpan {
    #[classattr]
    const NEW: Self = Self(FmtSpan::NEW);

    #[classattr]
    const ENTER: Self = Self(FmtSpan::ENTER);

    #[classattr]
    const EXIT: Self = Self(FmtSpan::EXIT);

    #[classattr]
    const CLOSE: Self = Self(FmtSpan::CLOSE);

    #[classattr]
    const NONE: Self = Self(FmtSpan::NONE);

    #[classattr]
    const ACTIVE: Self = Self(FmtSpan::ACTIVE);

    #[classattr]
    const FULL: Self = Self(FmtSpan::FULL);

    fn __or__<'py>(&self, other: Bound<'py, Self>) -> Self {
        Self(self.0.clone() & other.borrow().0.clone())
    }

    fn __and__<'py>(&self, other: Bound<'py, Self>) -> Self {
        Self(self.0.clone() & other.borrow().0.clone())
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

impl Debug for PyFmtSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
