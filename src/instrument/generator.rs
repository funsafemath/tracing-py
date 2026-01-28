use pyo3::{prelude::*, types::PyIterator};
use tracing::{Span, error_span};

use crate::any_ext::InfallibleAttr;

#[pyclass]
pub(crate) struct InstrumentedGenerator {
    inner: Py<PyAny>,
    span: Span,
}

#[pymethods]
impl InstrumentedGenerator {
    #[new]
    fn a(generator: Py<PyAny>) -> Self {
        Self {
            inner: generator,
            span: error_span!("fsdf"),
        }
    }

    fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .bind(py)
            .infallible_attr::<"__iter__", PyAny>()
            .call0()
    }

    fn __next__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.inner
            .bind(py)
            .infallible_attr::<"__next__", PyAny>()
            .call0()
    }
}
