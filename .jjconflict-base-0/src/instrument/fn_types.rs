pub mod async_generator;
pub mod coroutine;
pub mod generator;

use pyo3::{IntoPyObjectExt, prelude::*};
use tracing::Span;

use crate::{
    event::{ErrCallsite, RetCallsite, YieldCallsite},
    imports,
    instrument::fn_types::{
        async_generator::InstrumentedAsyncGenerator,
        coroutine::InstrumentedCoroutine,
        generator::{GeneratorType, InstrumentedGenerator},
    },
};

#[derive(Clone, Copy)]
pub enum FunctionType {
    Normal,
    Generator,
    Async,
    AsyncGenerator,
}

impl FunctionType {
    pub fn guess_from_return_value(ret_val: &Bound<'_, PyAny>) -> PyResult<Self> {
        let py = ret_val.py();

        Ok(if ret_val.is_instance(imports::get_coroutine_type(py))? {
            Self::Async
        } else if ret_val.is_instance(imports::get_generator_type(py))? {
            Self::Generator
        } else if ret_val.is_instance(imports::get_async_generator_type(py))? {
            Self::AsyncGenerator
        } else {
            Self::Normal
        })
    }

    pub fn instrument_ret_val(
        self,
        ret_val: Bound<'_, PyAny>,
        span: Span,
        ret_callsite: Option<RetCallsite>,
        err_callsite: Option<ErrCallsite>,
        yield_callsite: Option<YieldCallsite>,
    ) -> PyResult<Py<PyAny>> {
        let py = ret_val.py();
        match self {
            Self::Normal => Ok(ret_val.unbind()),
            Self::Generator => InstrumentedGenerator::new(
                ret_val.unbind(),
                span,
                ret_callsite,
                err_callsite,
                yield_callsite,
                GeneratorType::Normal,
            )
            .into_py_any(py),
            Self::Async => InstrumentedCoroutine::new(
                ret_val.unbind(),
                span,
                ret_callsite,
                err_callsite,
                GeneratorType::Normal,
            )
            .into_py_any(py),
            Self::AsyncGenerator => InstrumentedAsyncGenerator::new(
                ret_val.unbind(),
                span,
                err_callsite,
                yield_callsite,
            )
            .into_py_any(py),
        }
    }
}
