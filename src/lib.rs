#![feature(exact_size_is_empty)]
#![feature(trait_alias)]
#![feature(decl_macro)]
#![deny(clippy::perf)]
#![warn(clippy::trivially_copy_pass_by_ref)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::ptr_arg)]

mod cached;
mod callsite;
mod event;
mod ext;
mod formatting;
mod imports;
mod instrument;
mod introspect;
mod layer;
mod leak;
mod level;
mod py_type;
mod span;
mod template;

use pyo3::prelude::*;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[pymodule(name = "tracing")]
mod tracing {

    #[pymodule_export]
    use super::level::PyLevel;

    #[pymodule_export]
    use super::event::{py_debug, py_error, py_info, py_trace, py_warn};

    #[pymodule_export]
    use super::layer::{
        fmt::{FmtLayer, Format, NonBlocking, PyLogFile, span::PyFmtSpan},
        py_init,
    };

    #[pymodule_export]
    use super::instrument::py_instrument;
}
