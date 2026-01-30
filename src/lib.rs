#![feature(exact_size_is_empty)]
#![feature(trait_alias)]

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
mod span;
mod template;

use pyo3::prelude::*;

#[pymodule(name = "tracing")]
mod tracing {
    use super::*;

    #[pymodule_export]
    use level::PyLevel;

    #[pymodule_export]
    use event::{py_debug, py_error, py_info, py_trace, py_warn};

    #[pymodule_export]
    use layer::{
        FmtLayer, Format,
        fmt::{OutFile, span::PyFmtSpan},
        py_init,
    };

    #[pymodule_export]
    use instrument::py_instrument;
}
