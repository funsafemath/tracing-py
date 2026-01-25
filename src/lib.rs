#![feature(adt_const_params)]
#![feature(unsized_const_params)]
#![feature(inherent_associated_types)]

mod any_ext;
mod cached;
mod callsite;
mod event;
mod imports;
mod inspect;
// mod instrument;
mod layer;
mod leak;
mod level;
// mod span;
mod template;
mod valuable;

use ::tracing::Level;
use pyo3::prelude::*;
use tracing_core::dispatcher;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Registry};

#[pyfunction]
fn init() {
    // Registry::default().wi;
    dispatcher::set_global_default(
        tracing_subscriber::fmt()
            .json()
            .pretty()
            .with_file(true)
            .with_line_number(true)
            .without_time()
            .with_level(false)
            .with_target(false)
            .compact()
            .with_max_level(Level::ERROR)
            .finish()
            .with(tracing_subscriber::fmt::layer())
            .into(),
    )
    .unwrap();
    // .with(tracing_subscriber::fmt::layer());
    // .init();
}
#[pymodule(name = "tracing")]
mod tracing {
    use super::*;

    #[pymodule_export]
    use level::PyLevel;

    #[pymodule_export]
    use super::init;

    #[pymodule_export]
    use event::{py_debug, py_error, py_info, py_trace, py_warn};

    #[pymodule_export]
    use layer::{init_with, FmtLayer, Format};

    // #[pymodule_export]
    // use instrument::py_instrument;

    #[pymodule_init]
    fn init_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
        // module.add("instrument", instrument::INSTRUMENT.clone_ref(module.py()))?;
        Ok(())
    }
}
