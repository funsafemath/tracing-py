use pyo3::{IntoPyObjectExt, prelude::*, types::PyType};
use tracing::Span;

use crate::{
    any_ext::InfallibleAttr, imports::get_coroutine_type,
    instrument::generator::InstrumentedGenerator,
};

#[pyclass]
pub(crate) struct InstrumentedCoroutine {
    inner: Py<PyAny>,
    span: Span,
}

impl InstrumentedCoroutine {
    pub(crate) fn new(generator: Py<PyAny>, span: Span) -> Self {
        Self {
            inner: generator,
            span,
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
        self.inner
            .bind(py)
            .infallible_attr::<"send", PyAny>()
            .call1((arg,))
    }

    fn throw<'py>(&self, py: Python<'py>, arg: Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let _enter = self.span.enter();
        self.inner
            .bind(py)
            .infallible_attr::<"throw", PyAny>()
            .call1((arg,))
    }

    fn __await__<'py>(&self, py: Python<'py>) -> PyResult<Py<PyAny>> {
        let generator = self
            .inner
            .bind(py)
            .infallible_attr::<"__await__", PyAny>()
            .call0()?;
        Ok(
            InstrumentedGenerator::new(generator.unbind(), self.span.clone())
                .into_py_any(py)
                .unwrap(),
        )
    }

    #[getter]
    fn cr_await<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        self.inner.bind(py).infallible_attr::<"cr_await", PyAny>()
    }

    #[getter]
    fn cr_code<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        self.inner.bind(py).infallible_attr::<"cr_code", PyAny>()
    }

    #[getter]
    fn cr_frame<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        self.inner.bind(py).infallible_attr::<"cr_frame", PyAny>()
    }

    #[getter]
    fn cr_origin<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        self.inner.bind(py).infallible_attr::<"cr_origin", PyAny>()
    }

    #[getter]
    fn cr_running<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        self.inner.bind(py).infallible_attr::<"cr_running", PyAny>()
    }

    #[getter]
    fn cr_suspended<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        self.inner
            .bind(py)
            .infallible_attr::<"cr_suspended", PyAny>()
    }
}
