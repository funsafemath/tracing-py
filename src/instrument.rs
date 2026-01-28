pub(super) mod coroutine;
mod generator;
mod parameter;
mod py_signature;
mod signature;

use std::collections::HashSet;

use pyo3::{
    IntoPyObjectExt,
    exceptions::PyValueError,
    prelude::*,
    types::{PyCFunction, PyDict, PyFrame, PyFunction, PyTuple},
};
use tracing::{Level, error};
use tracing_core::Kind;

use crate::{
    callsite::{self, Context},
    function_ext::PyFunctionMethodsExt,
    imports::{get_coroutine_type, get_generator_type},
    inspect::frame::UnboundPyFrameMethodsExt,
    instrument::{
        coroutine::InstrumentedCoroutine, generator::InstrumentedGenerator,
        signature::extract_signature,
    },
    level::PyLevel,
    span::span,
};

fn instrument<'py>(
    py: Python<'py>,
    function: Bound<'py, PyFunction>,
    options: InstrumentOptions,
) -> PyResult<Bound<'py, PyAny>> {
    let signature = extract_signature(&function)?;
    let frame = PyFrame::from_thread_state(py).expect("must be called from python context");
    let callsite = callsite::get_or_init_callsite(
        Context::FrameAndCode {
            code: function.code(),
            frame,
        },
        options.level,
        signature.param_names(),
        Kind::SPAN,
    );

    let function = function.unbind();

    Ok(PyCFunction::new_closure(
        py,
        None,
        None,
        move |args: &Bound<'_, PyTuple>, kwargs: Option<&Bound<'_, PyDict>>| {
            let py = args.py();

            let function = function.bind(args.py());

            let bound: Vec<Bound<'_, PyAny>> = match signature.bind(
                py,
                args,
                kwargs,
                function.get_defaults(),
                function.get_kw_defaults(),
            ) {
                Ok(sign) => sign,
                Err(e) => {
                    error!("failed to bind arguments to {function:?} parameters: {e}");
                    return function.call(args, kwargs).map(|x| x.unbind());
                }
            };

            let span = span(py, Level::INFO, signature.param_names(), bound, callsite);

            let res = {
                let _entered = span.as_ref().map(|x| x.enter());
                function.call(args, kwargs)?
            };

            let Some(span) = span else {
                return Ok(res.unbind());
            };

            // todo: log errors & propagate return value
            if res.is_instance(get_generator_type(py))? {
                Ok(InstrumentedGenerator::new(res.unbind(), span).into_py_any(py)?)
            } else if res.is_instance(get_coroutine_type(py))? {
                Ok(InstrumentedCoroutine::new(res.unbind(), span).into_py_any(py)?)
            } else {
                Ok(res.unbind())
            }
        },
    )?
    .into_any())
}

#[pyclass]
#[derive(Clone)]
struct InstrumentOptions {
    level: Level,
    skip: HashSet<String>,
    skip_all: bool,
}

impl InstrumentOptions {
    const DEFAULT_LEVEL: Level = Level::INFO;
}

#[pymethods]
impl InstrumentOptions {
    fn __call__<'py>(
        &self,
        py: Python<'py>,
        func: Bound<'py, PyFunction>,
    ) -> PyResult<Bound<'py, PyAny>> {
        instrument(py, func, self.clone())
    }
}

impl Default for InstrumentOptions {
    fn default() -> Self {
        Self {
            level: Self::DEFAULT_LEVEL,
            skip: Default::default(),
            skip_all: Default::default(),
        }
    }
}

// todo: set function name/signature (functools.wraps doesn't work on native functions, wrapt is too slow)
//
// overhead is predominantly caused by Span::new call, so optimizing this function is not a priority
#[pyfunction(name = "instrument")]
#[pyo3(signature = (func = None, /, *, level = None, skip = None, skip_all = None))]
pub(crate) fn py_instrument<'py>(
    py: Python<'py>,
    func: Option<Bound<'py, PyFunction>>,
    level: Option<PyLevel>,
    skip: Option<Vec<String>>,
    skip_all: Option<bool>,
) -> PyResult<Bound<'py, PyAny>> {
    if let Some(func) = func {
        if skip.is_some() || level.is_some() || skip_all.is_some() {
            Err(PyValueError::new_err(
                "instrument() does not accept keyword arguments when the positional argument is a function",
            ))
        } else {
            instrument(py, func, InstrumentOptions::default())
        }
    } else {
        let mut options = InstrumentOptions::default();
        if let Some(level) = level {
            options.level = Level::from(level);
        }
        if let Some(skip) = skip {
            options.skip = skip.into_iter().collect();
        }
        if let Some(skip_all) = skip_all {
            options.skip_all = skip_all;
        }

        options.into_bound_py_any(py)
    }
}
