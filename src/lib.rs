#![feature(adt_const_params)]
#![feature(unsized_const_params)]

mod any_ext;
mod cached;
mod callsite;
mod event;
mod imports;
mod inspect;
// mod instrument;
mod leak;
mod level;
// mod span;
mod template;
mod valuable;

use pyo3::prelude::*;

#[pyfunction]
fn init() {
    tracing_subscriber::fmt()
        .json()
        .with_file(true)
        .with_line_number(true)
        .without_time()
        .with_level(false)
        .with_target(false)
        .init();
}
#[pymodule(name = "tracing")]
mod tracing {
    use super::*;

    #[pymodule_export]
    use level::Level;

    #[pymodule_export]
    use super::init;

    #[pymodule_export]
    use event::{py_debug, py_error, py_info, py_trace, py_warn};

    // #[pymodule_export]
    // use instrument::py_instrument;

    #[pymodule_init]
    fn init_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
        // module.add("instrument", instrument::INSTRUMENT.clone_ref(module.py()))?;
        Ok(())
    }
}
