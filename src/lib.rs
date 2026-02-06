#![feature(exact_size_is_empty)]
#![feature(trait_alias)]
#![feature(decl_macro)]
#![warn(clippy::allow_attributes)]

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
        fmt::{
            FmtLayer, PyFormat,
            file::{NonBlocking, PyLogFile, PyRollingLog},
            rotation::PyRotation,
            span::PyFmtSpan,
            time::format::PyTimeFormat,
            time::timer::PyTimer,
            time::timer::Time,
        },
        py_init,
    };

    #[pymodule_export]
    use super::leak::debug::leak_info;

    #[pymodule_export]
    use super::instrument::py_instrument;
}
