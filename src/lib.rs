#![feature(adt_const_params)]
#![feature(unsized_const_params)]
#![feature(exact_size_is_empty)]

mod any_ext;
mod cached;
mod callsite;
mod event;
mod ffi_ext;
mod function_ext;
mod imports;
mod inspect;
mod instrument;
mod layer;
mod leak;
mod level;
mod span;
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
    use layer::{FmtLayer, Format, init};

    #[pymodule_export]
    use instrument::py_instrument;
}
