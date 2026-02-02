use pyo3::{IntoPyObjectExt, prelude::*, types::PyType};
use tracing::Span;

use crate::{
    event::{ErrCallsite, RetCallsite, YieldCallsite},
    ext::any::infallible_attr,
    imports::get_coroutine_type,
    instrument::generator::InstrumentedGenerator,
};

#[pyclass]
pub struct InstrumentedCoroutine {
    inner: Py<PyAny>,
    span: Span,
    ret_callsite: Option<RetCallsite>,
    err_callsite: Option<ErrCallsite>,
    yield_callsite: Option<YieldCallsite>,
}

impl InstrumentedCoroutine {
    pub fn new(
        coroutine: Py<PyAny>,
        span: Span,
        ret_callsite: Option<RetCallsite>,
        err_callsite: Option<ErrCallsite>,
        yield_callsite: Option<YieldCallsite>,
    ) -> Self {
        Self {
            inner: coroutine,
            span,
            ret_callsite,
            err_callsite,
            yield_callsite,
        }
    }
}

// todo: impl all coroutine methods
#[pymethods]
impl InstrumentedCoroutine {
    // there must be a better way to set the superclass
    // also this does not get added to asyncio coroutine classes cache for some reason? or maybe it is, not sure
    // failed attemps: classattr __class__: unsettable in pyo3?, classattr __bases__: coroutine can't be used as a base
    #[getter]
    fn __class__<'py>(&self, py: Python<'py>) -> &Bound<'py, PyType> {
        get_coroutine_type(py)
    }

    fn send<'py>(&self, py: Python<'py>, arg: Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let _enter = self.span.enter();
        infallible_attr!(self.inner, "send", py).call1((arg,))
    }

    fn throw<'py>(&self, py: Python<'py>, arg: Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let _enter = self.span.enter();
        infallible_attr!(self.inner, "throw", py).call1((arg,))
    }

    fn __await__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let generator = infallible_attr!(self.inner, "__await__", py).call0()?;
        InstrumentedGenerator::new(
            generator.unbind(),
            self.span.clone(),
            self.ret_callsite,
            self.err_callsite,
            self.yield_callsite,
        )
        .into_py_any(py)
    }

    #[getter]
    fn cr_await<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        infallible_attr!(self.inner, "cr_await", py)
    }

    #[getter]
    fn cr_code<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        infallible_attr!(self.inner, "cr_code", py)
    }

    #[getter]
    fn cr_frame<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        infallible_attr!(self.inner, "cr_frame", py)
    }

    #[getter]
    fn cr_origin<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        infallible_attr!(self.inner, "cr_origin", py)
    }

    #[getter]
    fn cr_running<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        infallible_attr!(self.inner, "cr_running", py)
    }

    #[getter]
    fn cr_suspended<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        infallible_attr!(self.inner, "cr_suspended", py)
    }
}
