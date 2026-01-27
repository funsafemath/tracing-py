mod fmt;

pub(crate) use fmt::{FmtLayer, Format};

use pyo3::{Bound, PyAny, PyResult, exceptions::PyRuntimeError, pyfunction, types::PyAnyMethods};
use tracing_subscriber::{
    Layer, Registry, layer::SubscriberExt, registry, util::SubscriberInitExt,
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
        None => vec![Box::new(tracing_subscriber::fmt::layer())],
    };

    registry()
        .with(layers)
        .try_init()
        .map_err(|x| PyRuntimeError::new_err(x.to_string()))?;

    Ok(())
}
