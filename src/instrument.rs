use std::sync::LazyLock;

use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyCFunction, PyDict, PyFunction, PyTuple},
};
use tracing::Level;
use tracing_core::Kind;

use crate::callsite::get_or_init_callsite;

// todo: use PyOnceCell
// maybe use a Py<PyAny> instead? some python implementation may have wraps as a builtin function, not as a python one
static WRAPT_DECORATOR: LazyLock<Py<PyFunction>> = LazyLock::new(|| {
    Python::attach(|py: Python<'_>| {
        let functools = py.import("wrapt").expect("failed to import wrapt");
        let wraps = functools
            .getattr("decorator")
            .expect("failed to find \"wrapper\" in wrapt module");

        wraps
            .cast_into()
            .expect("failed to cast wrapt.wrapper into a python function")
            .unbind()
    })
});

pub(super) static INSTRUMENT: LazyLock<Py<PyAny>> = LazyLock::new(|| {
    Python::attach(|py: Python<'_>| {
        // let a = wrap_pyfunction!(instrument);
        WRAPT_DECORATOR
            .call1(
                py,
                (PyCFunction::new_closure(py, None, None, instrument).unwrap(),),
            )
            .unwrap()
            .into_any()
    })
});

fn instrument(
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    // wrapt passes 4 positional arguments
    assert!(kwargs.is_none());

    let py = args.py();

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
    let wraps = WRAPT_DECORATOR.bind(py);

    let t = || {};

    // let wrapper = wraps.call1((func,))?.call1((Closure {
    //     wrapped: func.clone().unbind(),
    //     __module__: None,
    // },))?;

    // Ok(wrapper.unbind())
    // let a = (instrument);
    // Ok(Closure {
    //     wrapped: func.clone().unbind(),
    //     __module__: None,
    // }
    // .into_py_any(py)?)
    let add_one = PyCFunction::new_closure(py, None, None, |args, kwargs| {}).unwrap();
    Ok(add_one.unbind().into_any())
    // todo()
    // Ok(PyCFunction::new_closure(py, None, None, |py, a| {}).unwrap())
}
