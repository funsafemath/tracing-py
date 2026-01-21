mod callsite;
mod event;
mod inspect;

use pyo3::prelude::*;
use tracing::Metadata;

use crate::event::{py_debug, py_error, py_info, py_trace, py_warn};

#[pyfunction]
fn init() {
    // tracing_subscriber::fmt().compact().without_time().init();
    tracing_subscriber::fmt().with_file(true).init();
}

#[pymodule(name = "tracing")]
fn tracing_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_error, m)?)?;
    m.add_function(wrap_pyfunction!(py_warn, m)?)?;
    m.add_function(wrap_pyfunction!(py_info, m)?)?;
    m.add_function(wrap_pyfunction!(py_debug, m)?)?;
    m.add_function(wrap_pyfunction!(py_trace, m)?)?;
    m.add_function(wrap_pyfunction!(init, m)?)?;

    Ok(())
}
