pub(super) mod coroutine;
mod generator;
mod parameter;
mod py_signature;
mod signature;

use pyo3::{
    IntoPyObjectExt,
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
    span::span,
};

// todo: set function name/signature (functools.wraps doesn't work on native functions, wrapt is too slow)
#[pyfunction(name = "instrument")]
pub(crate) fn py_instrument<'py>(
    py: Python<'py>,
    function: Bound<'py, PyFunction>,
) -> PyResult<Bound<'py, PyAny>> {
    let signature = extract_signature(&function)?;
    let frame = PyFrame::from_thread_state(py).expect("must be called from python context");
    let callsite = callsite::get_or_init_callsite(
        Context::FrameAndCode {
            code: function.code(),
            frame,
        },
        Level::INFO,
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
    )
    .unwrap()
    .into_any())
}
