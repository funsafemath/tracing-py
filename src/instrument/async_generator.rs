use pyo3::{IntoPyObjectExt, prelude::*, types::PyType};
use tracing::Span;

use crate::{
    event::{self, ErrCallsite, RetCallsite, YieldCallsite},
    ext::any::infallible_attr,
    imports::get_async_generator_type,
    instrument::{coroutine::InstrumentedCoroutine, generator::GeneratorType},
};

// todo: impl all async generator methods, use proper inner type
#[pyclass]
pub struct InstrumentedAsyncGenerator {
    inner: Py<PyAny>,
    span: Span,
    err_callsite: Option<ErrCallsite>,
    yield_callsite: Option<YieldCallsite>,
}

impl InstrumentedAsyncGenerator {
    pub fn new(
        inner: Py<PyAny>,
        span: Span,
        err_callsite: Option<ErrCallsite>,
        yield_callsite: Option<YieldCallsite>,
    ) -> Self {
        Self {
            inner,
            span,
            err_callsite,
            yield_callsite,
        }
    }
}

#[pymethods]
impl InstrumentedAsyncGenerator {
    // see coroutine comment
    #[getter]
    fn __class__<'py>(&self, py: Python<'py>) -> &Bound<'py, PyType> {
        get_async_generator_type(py)
    }

    // todo: use c anext method directly
    fn __anext__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        // it's an async function, so the first invocation should never fail, so we don't need to log an error
        //
        // technically it may be a normal function that returns a coroutine that actually does something
        // and maybe even logs something, and in that case we'd want to enter the span before calling __anext__,
        // but I'm not sure if it worth implementing, as this will also generate an enter/exit event in case of false positives
        let coro_next = infallible_attr!(self.inner, "__anext__", py).call0()?;

        InstrumentedCoroutine::new(
            coro_next.into(),
            self.span.clone(),
            // normal coroutine return <=> async generator yield
            self.yield_callsite
                .map(event::CallsiteCast::cast::<RetCallsite>),
            self.err_callsite,
            None,
            GeneratorType::AsyncGeneratorCoroutine,
        )
        .into_bound_py_any(py)
    }

    // we can pretty much always return Self, but let's call the actual __aiter__ method just to be sure
    fn __aiter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let iterable = infallible_attr!(self.inner, "__aiter__", py).call0()?;
        Self {
            inner: iterable.unbind(),
            span: self.span.clone(),
            err_callsite: self.err_callsite,
            yield_callsite: self.yield_callsite,
        }
        .into_bound_py_any(py)
    }
}
