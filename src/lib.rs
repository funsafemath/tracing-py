mod callsite;
mod event;
mod inspect;
mod level;

use pyo3::prelude::*;

#[pyfunction]
fn init() {
    tracing_subscriber::fmt()
        // .json()
        .with_file(true)
        .with_line_number(true)
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
}
