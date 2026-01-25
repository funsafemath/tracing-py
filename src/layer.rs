use pyo3::{pyclass, pyfunction, pymethods, Bound, Py, Python};
use tracing_subscriber::{
    fmt::{self},
    layer::SubscriberExt,
    registry,
    util::SubscriberInitExt,
    Layer, Registry,
};
#[pyclass]
pub(super) struct FmtLayer {
    log_internal_errors: Option<bool>,
    with_ansi: Option<bool>,
    with_file: Option<bool>,
    with_level: Option<bool>,
    with_line_number: Option<bool>,
    with_target: Option<bool>,
    with_thread_ids: Option<bool>,
    with_max_level: Option<crate::level::PyLevel>,
    without_time: bool,
    format: Format,
}

#[pymethods]
impl FmtLayer {
    #[allow(
        clippy::too_many_arguments,
        reason = "how else am I supposed to implement a python constructor?"
    )]
    #[new]
    #[pyo3(signature = (log_internal_errors = None, with_ansi = None, with_file = None, with_level = None,
    with_line_number = None, with_target = None, with_thread_ids = None, with_max_level = None, without_time = false,
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
            with_max_level: with_max_level.map(|x| x.borrow().clone()),
            without_time,
            format: format.bind_borrowed(py).borrow().clone(),
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
            with_max_level,
            without_time,
            format,
        } = value;

        // chaining methods would be more elegant, it requires guessing the default values
        // let layer = tracing_subscriber::fmt::Layer::new();
        let mut layer: fmt::Layer<Registry> = fmt::layer();
        // let mut layer = &;

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

        // incredibly ugly, but i didn't find a simple way to do this use to generic parameters
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

#[pyclass]
#[derive(Clone)]
pub(super) enum Format {
    Full,
    Compact,
    Pretty,
    Json,
}

#[pyfunction]
pub(super) fn init_with(layers: Vec<Bound<'_, FmtLayer>>) -> eyre::Result<()> {
    let layers: Vec<Box<dyn Layer<Registry> + Send + Sync>> =
        layers.into_iter().map(|x| (&*x.borrow()).into()).collect();

    registry().with(layers).try_init()?;

    Ok(())
}
