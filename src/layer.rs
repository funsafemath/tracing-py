mod fmt;

pub(crate) use fmt::{FmtLayer, Format};

use pyo3::{Bound, PyAny, PyResult, exceptions::PyRuntimeError, pyfunction, types::PyAnyMethods};
use tracing_subscriber::{
    Layer, Registry, fmt::format::FmtSpan, layer::SubscriberExt, registry, util::SubscriberInitExt,
};

// todo: accept *args instead of a Sequence
#[pyfunction(name = "init")]
#[pyo3(signature = (layers = None))]
pub(crate) fn py_init(layers: Option<Bound<'_, PyAny>>) -> PyResult<()> {
    let layers: Vec<Box<dyn Layer<Registry> + Send + Sync>> = match layers {
        Some(layers) => {
            if let Ok(layers) = layers.extract::<Vec<Bound<'_, FmtLayer>>>() {
                layers
                    .into_iter()
                    .map(|x: Bound<'_, FmtLayer>| (&*x.borrow()).into())
                    .collect()
            } else {
                let layer = layers.cast::<FmtLayer>()?;
                vec![(&*layer.borrow()).into()]
            }
        }
        None => vec![Box::new(
            // TODO: add span_events to options, rn it's here only for development, i hope i won't accidentally commit it
            tracing_subscriber::fmt::layer()
                .with_line_number(true)
                .with_file(true)
                .with_target(true)
                .with_ansi(true)
                .with_level(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_span_events(FmtSpan::FULL),
        )],
    };

    registry()
        .with(layers)
        .try_init()
        .map_err(|x| PyRuntimeError::new_err(x.to_string()))?;

    Ok(())
}
