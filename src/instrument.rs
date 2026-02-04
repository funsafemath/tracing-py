mod fn_types;
mod log_parameters;
mod parameter;
mod py_signature;
mod signature;

use std::ops::Index;

use pyo3::{
    IntoPyObjectExt,
    prelude::*,
    types::{PyCFunction, PyDict, PyFrame, PyFunction, PyTuple},
};
use rapidhash::{RapidHashMap, RapidHashSet};
use tracing::{Level, error};
use tracing_core::Kind;

use crate::{
    callsite::{self, Context},
    event::{self, ret_event},
    ext::{frame::UnboundPyFrameMethodsExt, function::PyFunctionMethodsExt},
    instrument::{
        fn_types::{
            FunctionType,
            async_generator::InstrumentedAsyncGenerator,
            coroutine::InstrumentedCoroutine,
            generator::{GeneratorType, InstrumentedGenerator},
        },
        log_parameters::RetLog,
        signature::extract_signature,
    },
    leak::VecLeaker,
    level::PyLevel,
    span::span,
};

// todo: set function name/signature (functools.wraps doesn't work on native functions, wrapt is too slow)
// todo: allow instrumenting native functions
// todo: skip arg extraction if span is not enabled
// todo: warn/throw an error if attempting to skip a non-existent parameter
// todo: warn/throw an error if trying to use log_yield on non-generator function
//
// overhead is predominantly caused by Span::new call, so optimizing this function is not a priority
fn instrument<'py>(
    py: Python<'py>,
    function: Bound<'py, PyFunction>,
    options: InstrumentOptions,
) -> PyResult<Bound<'py, PyAny>> {
    let signature = extract_signature(&function)?;
    let frame = PyFrame::from_thread_state(py).expect("must be called from python context");

    let param_names = signature
        .param_names()
        .iter()
        .copied()
        .filter(|x| !options.skip.contains(*x))
        .collect();

    let param_names = VecLeaker::leak_or_get_once(param_names);

    let code = function.code();

    let ctx = Context::FrameAndCode { code, frame };

    let (ret_callsite, yield_callsite, err_callsite) =
        options.ret_log.callsites(ctx.clone(), options.level);

    let callsite = callsite::get_or_init_callsite(ctx, options.level, param_names, Kind::SPAN);

    let retain_indices = if options.skip_all {
        vec![]
    } else {
        let mut param_to_index = RapidHashMap::default();
        for (i, param) in signature.param_names().iter().enumerate() {
            param_to_index.insert(*param, i);
        }

        let mut indices = param_names
            .iter()
            .map(|x| *param_to_index.index(x))
            .collect::<Vec<_>>();
        indices.sort_unstable();
        indices
    };

    let function = function.unbind();

    Ok(PyCFunction::new_closure(
        py,
        None,
        None,
        move |args: &Bound<'_, PyTuple>, kwargs: Option<&Bound<'_, PyDict>>| {
            let py = args.py();

            let function = function.bind(args.py());

            let bound = if retain_indices.is_empty() {
                vec![]
            } else {
                let mut bound: Vec<Bound<'_, PyAny>> = match signature.bind(
                    py,
                    args,
                    kwargs,
                    function.get_defaults(),
                    function.get_kw_defaults(),
                ) {
                    Ok(sign) => sign,
                    Err(e) => {
                        error!("failed to bind arguments to {function:?} parameters: {e}");
                        return function.call(args, kwargs).map(Bound::unbind);
                    }
                };

                // that's quite a bad algorithm; it's possible to do signature binding with skipping unneeded parameters in
                // O(number of not skipped parameters) time
                let mut bound_skipped = Vec::with_capacity(retain_indices.len());
                for idx in retain_indices.iter().rev() {
                    bound_skipped.push(bound.swap_remove(*idx));
                }
                bound_skipped.reverse();
                bound_skipped
            };

            let span = span(py, options.level, signature.param_names(), bound, callsite);

            let (res, fn_type) = {
                let _entered = span.as_ref().map(|x| x.enter());
                let ret_val = match function.call(args, kwargs) {
                    Ok(ret_val) => ret_val,
                    Err(err) => {
                        let err = err.into_bound_py_any(py)?;
                        if let Some(err_callsite) = err_callsite {
                            event::err_event(py, err.clone().into_bound_py_any(py)?, err_callsite);
                        }
                        return Err(PyErr::from_value(err));
                    }
                };

                // todo: cache it, maybe?
                let fn_type = FunctionType::guess_from_return_value(&ret_val)?;
                if let Some(ret_callsite) = ret_callsite
                    && matches!(fn_type, FunctionType::Normal)
                {
                    // todo: use &Bound in PyCachedValuable and get rid of this clone
                    ret_event(py, ret_val.clone(), ret_callsite);
                }
                (ret_val, fn_type)
            };

            let Some(span) = span else {
                return Ok(res.unbind());
            };

            // todo: find out how can into_py_any fail and log errors & propagate return value
            // if there's a real situation where is does fail
            Ok(match fn_type {
                FunctionType::Normal => res.unbind(),
                FunctionType::Generator => InstrumentedGenerator::new(
                    res.unbind(),
                    span,
                    ret_callsite,
                    err_callsite,
                    yield_callsite,
                    GeneratorType::Normal,
                )
                .into_py_any(py)?,
                FunctionType::Async => {
                    // yield_callsite in this case will just log <Future Pending> on (some) await points, which isn't useful
                    InstrumentedCoroutine::new(
                        res.unbind(),
                        span,
                        ret_callsite,
                        err_callsite,
                        None,
                        GeneratorType::Normal,
                    )
                    .into_py_any(py)?
                }
                FunctionType::AsyncGenerator => InstrumentedAsyncGenerator::new(
                    res.unbind(),
                    span,
                    err_callsite,
                    yield_callsite,
                )
                .into_py_any(py)?,
            })
        },
    )?
    .into_any())
}

#[pyclass]
#[derive(Clone)]
struct InstrumentOptions {
    level: Level,
    skip: RapidHashSet<String>,
    skip_all: bool,
    ret_log: RetLog,
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
            skip: RapidHashSet::default(),
            skip_all: Default::default(),
            ret_log: RetLog::default(),
        }
    }
}

#[pyfunction(name = "instrument")]
#[pyo3(signature = (func = None, /, *, level = PyLevel::Info, skip = Vec::new(), skip_all = false, ret = false, ret_err_only = false, no_yield = false))]
#[expect(
    clippy::too_many_arguments,
    reason = "no it's not enough for an average python function"
)]
#[expect(
    clippy::fn_params_excessive_bools,
    reason = "it's python and the arguments are kw-only, so `you can't remember argument order` does not apply here"
)]
pub fn py_instrument<'py>(
    py: Python<'py>,
    func: Option<Bound<'py, PyFunction>>,
    level: PyLevel,
    skip: Vec<String>,
    skip_all: bool,
    ret: bool,
    ret_err_only: bool,
    no_yield: bool,
) -> PyResult<Bound<'py, PyAny>> {
    let options = InstrumentOptions {
        level: Level::from(level),
        skip: skip.into_iter().collect(),
        skip_all,
        ret_log: RetLog::from_opts(ret, no_yield, ret_err_only)?,
    };

    match func {
        Some(func) => instrument(py, func, options),
        None => options.into_bound_py_any(py),
    }
}
