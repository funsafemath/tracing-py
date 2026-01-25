use std::sync::LazyLock;

use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyCFunction, PyDict, PyFunction, PyTuple},
};
use tracing::Level;
use tracing_core::Kind;

use crate::{callsite::get_or_init_callsite, imports::get_wrapt_decorator};

pub(super) static INSTRUMENT: LazyLock<Py<PyAny>> = LazyLock::new(|| {
    Python::attach(|py: Python<'_>| {
        // let a = wrap_pyfunction!(instrument);
        get_wrapt_decorator(py)
            .call1((PyCFunction::new_closure(py, None, None, instrument).unwrap(),))
            .unwrap()
            .into_any()
            .unbind()
    })
});

fn instrument(
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    // wrapt passes 4 positional arguments
    assert!(kwargs.is_none());

    // py_error(PyString::new(py, "instrumented").into_any());

    let wrapped = args.get_borrowed_item(0)?;
    // let instance = fn_args.get_borrowed_item(1)?;
    let wrapped_args = args.get_borrowed_item(2)?.cast()?;
    let wrapped_kwargs = args.get_borrowed_item(3)?.cast()?;

    Ok(wrapped.call(wrapped_args, Some(&wrapped_kwargs))?.unbind())
}

#[pyfunction(name = "instrument")]
pub(super) fn py_instrument(py: Python<'_>, func: &Bound<'_, PyAny>) -> eyre::Result<Py<PyAny>> {
    static DEFAULT_FIELDS: &[&str] = &["message"];
    if !func.is_callable() {
        Err(PyValueError::new_err("expected a callable object"))?;
    }

    let callsite = get_or_init_callsite(py, Level::ERROR, DEFAULT_FIELDS, Kind::SPAN);
    dbg!(callsite);

    // functools.wraps is wrapped -> wrapper -> fn
    let wraps = get_wrapt_decorator(py);

    let t = || {};

    let add_one = PyCFunction::new_closure(py, None, None, |args, kwargs| {}).unwrap();
    Ok(add_one.unbind().into_any())
}
