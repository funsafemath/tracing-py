use pyo3::{IntoPyObjectExt, exceptions::PyStopIteration, prelude::*, types::PyType};
use tracing::Span;

use crate::{
    event::{self, ErrCallsite, RetCallsite, YieldCallsite},
    ext::any::infallible_attr,
    imports::get_generator_type,
};

// todo: impl all generator methods, use proper inner type
#[pyclass]
pub(crate) struct InstrumentedGenerator {
    inner: Py<PyAny>,
    span: Span,
    ret_callsite: Option<RetCallsite>,
    err_callsite: Option<ErrCallsite>,
    yield_callsite: Option<YieldCallsite>,
}

impl InstrumentedGenerator {
    pub(crate) fn new(
        inner: Py<PyAny>,
        span: Span,
        ret_callsite: Option<RetCallsite>,
        err_callsite: Option<ErrCallsite>,
        yield_callsite: Option<YieldCallsite>,
    ) -> Self {
        Self {
            inner,
            span,
            ret_callsite,
            err_callsite,
            yield_callsite,
        }
    }
}

#[pymethods]
impl InstrumentedGenerator {
    // see coroutine comment
    #[getter]
    fn __class__<'py>(&self, py: Python<'py>) -> &Bound<'py, PyType> {
        get_generator_type(py)
    }

    // todo: use c iter next method directly (the performance is already great actually)
    fn __next__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let _enter = self.span.enter();
        let ret_val = infallible_attr!(self.inner, "__next__", py).call0();
        match ret_val {
            Ok(ret) => {
                if let Some(yield_callsite) = self.yield_callsite {
                    event::yield_event(py, ret.clone(), yield_callsite);
                }
                Ok(ret)
            }
            Err(err) => {
                let err = err.into_bound_py_any(py)?;
                if let Some(err_callsite) = self.err_callsite {
                    // todo: as i've already written in instrument.rs,
                    // events should use &Bound, not Bound, so clones aren't needed
                    let err = err.clone();
                    match err.cast_into::<PyStopIteration>() {
                        Ok(stop_iteration) => {
                            if let Some(ret_callsite) = self.ret_callsite {
                                event::ret_event(
                                    py,
                                    infallible_attr!(stop_iteration, "value"),
                                    ret_callsite,
                                );
                            }
                        }
                        Err(err) => {
                            event::err_event(py, err.into_inner(), err_callsite);
                        }
                    }
                }
                Err(PyErr::from_value(err))
            }
        }
    }

    // we can pretty much always return Self, but let's call the actual __iter__ method just to be sure
    fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let iterable = infallible_attr!(self.inner, "__iter__", py).call0()?;
        InstrumentedGenerator {
            inner: iterable.unbind(),
            span: self.span.clone(),
            ret_callsite: self.ret_callsite,
            err_callsite: self.err_callsite,
            yield_callsite: self.yield_callsite,
        }
        .into_bound_py_any(py)
    }
}
