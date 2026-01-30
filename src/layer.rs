pub(crate) mod fmt;

use std::io::stdout;

pub(crate) use fmt::{FmtLayer, Format};

use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyCFunction};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    FmtSubscriber, Layer, Registry, layer::SubscriberExt, registry, util::SubscriberInitExt,
};

use crate::{imports::get_atexit_register, layer::fmt::to_layer::ToDynLayer};

trait ThreadSafeLayer = Layer<Registry> + Send + Sync;

// todo: accept *args instead of a Sequence
#[pyfunction(name = "init")]
#[pyo3(signature = (layers = None))]
pub(crate) fn py_init(py: Python<'_>, layers: Option<Bound<'_, PyAny>>) -> PyResult<()> {
    let layers_with_guards = match layers {
        Some(layers) => {
            if let Ok(layers) = layers.extract::<Vec<Bound<'_, FmtLayer>>>() {
                layers
                    .into_iter()
                    .map(|x: Bound<'_, FmtLayer>| x.dyn_layer())
                    .collect::<PyResult<Vec<(_, _)>>>()?
            } else {
                vec![layers.cast::<FmtLayer>()?.dyn_layer()?]
            }
        }
        None => {
            // todo: ensure that this default layer is equal to the default FmtLayer()
            let (writer, guard) = tracing_appender::non_blocking(stdout());
            let layer = tracing_subscriber::fmt::layer()
                .with_writer(writer)
                .with_filter(FmtSubscriber::DEFAULT_MAX_LEVEL);
            let dyn_layer: Box<dyn ThreadSafeLayer> = Box::new(layer);

            vec![(dyn_layer, Some(guard))]
        }
    };

    let (layers, guards): (Vec<_>, Vec<_>) = layers_with_guards.into_iter().unzip();

    registry()
        .with(layers)
        .try_init()
        .map_err(|x| PyRuntimeError::new_err(x.to_string()))?;

    let guard_vec = PyWorkerGuardVec::new(guards.into_iter().flatten().collect());
    guard_vec.into_pyobject(py)?.drop_at_exit()
}

#[pyclass]
struct PyWorkerGuardVec {
    guards: Option<Vec<WorkerGuard>>,
}

impl PyWorkerGuardVec {
    fn new(guards: Vec<WorkerGuard>) -> Self {
        Self {
            guards: Some(guards),
        }
    }

    fn drop_guards(&mut self) {
        assert!(self.guards.is_some());
        self.guards = None;
    }
}

trait PyGuardsMethods {
    fn drop_at_exit(self) -> PyResult<()>;
}

impl<'py> PyGuardsMethods for Bound<'py, PyWorkerGuardVec> {
    // todo: we can check if it's empty, and skip atexit setup if it is
    fn drop_at_exit(self) -> PyResult<()> {
        let closure = match PyCFunction::new_closure(self.py(), None, None, {
            let guard_vec = self.clone().unbind();

            move |args, _| {
                let py = args.py();
                guard_vec.borrow_mut(py).drop_guards();
            }
        }) {
            Ok(closure) => closure,
            Err(err) => {
                // python should do it itself, but why not be even more sure that worker threads will be stopped?
                self.borrow_mut().drop_guards();
                return Err(err);
            }
        };
        match get_atexit_register(self.py()).call1((&closure,)) {
            Ok(_) => Ok(()),
            Err(err) => {
                // same
                self.borrow_mut().drop_guards();
                Err(err)
            }
        }
    }
}
