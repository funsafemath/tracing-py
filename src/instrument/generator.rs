use pyo3::{IntoPyObjectExt, prelude::*, types::PyType};
use tracing::Span;

use crate::{any_ext::InfallibleAttr, imports::get_generator_type};

// todo: impl all generator methods, use proper inner type
#[pyclass]
pub(crate) struct InstrumentedGenerator {
    inner: Py<PyAny>,
    span: Span,
}

impl InstrumentedGenerator {
    pub(crate) fn new(inner: Py<PyAny>, span: Span) -> Self {
        Self { inner, span }
    }
}

#[pymethods]
impl InstrumentedGenerator {
    // there must be a better way to set the superclass
    // also this does not get added to asyncio coroutine classes cache for some reason? or maybe it is, not sure
    #[getter]
    fn __class__<'py>(&self, py: Python<'py>) -> &Bound<'py, PyType> {
        get_generator_type(py)
    }

    // todo: use c iter next method directly (the performance is already great actually)
    fn __next__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let _enter = self.span.enter();
        self.inner
            .bind(py)
            .infallible_attr::<"__next__", PyAny>()
            .call0()
    }

    // we can pretty much always return Self, but let's call the actual __iter__ method just to be sure
    fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let _enter = self.span.enter();
        let iterable = self
            .inner
            .bind(py)
            .infallible_attr::<"__iter__", PyAny>()
            .call0()?;
        InstrumentedGenerator {
            inner: iterable.unbind(),
            span: self.span.clone(),
        }
        .into_bound_py_any(py)
    }
}
