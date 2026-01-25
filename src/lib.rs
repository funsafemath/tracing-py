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

use pyo3::prelude::*;

#[pymodule(name = "tracing")]
mod tracing {
    use super::*;

    #[pymodule_export]
    use level::PyLevel;

    #[pymodule_export]
    use event::{py_debug, py_error, py_info, py_trace, py_warn};

    #[pymodule_export]
    use layer::{init, FmtLayer, Format};

    // #[pymodule_export]
    // use instrument::py_instrument;

    #[pymodule_init]
    fn init_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
        // module.add("instrument", instrument::INSTRUMENT.clone_ref(module.py()))?;
        Ok(())
    }
}
