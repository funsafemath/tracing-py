use pyo3::{
    Bound, PyAny, Python, pyfunction,
    types::{PyDict, PyDictMethods},
};
use tracing::{Event, Level, Value};
use tracing_core::{Callsite, Kind};
use valuable::Valuable;

use crate::{
    cached::{CachedDisplay, CachedValue},
    callsite::{self, CallsiteAction},
    leak::{Leaker, VecLeaker},
    valuable::PyCachedValuable,
};

#[pyfunction(name = "error")]
#[pyo3(signature = (message = None, **kwargs))]
pub(super) fn py_error(
    py: Python<'_>,
    message: Option<Bound<'_, PyAny>>,
    kwargs: Option<&Bound<'_, PyDict>>,
) {
    event(py, Level::ERROR, message, kwargs);
}

#[pyfunction(name = "warn")]
#[pyo3(signature = (message = None, **kwargs))]
pub(super) fn py_warn(
    py: Python<'_>,
    message: Option<Bound<'_, PyAny>>,
    kwargs: Option<&Bound<'_, PyDict>>,
) {
    event(py, Level::WARN, message, kwargs);
}

#[pyfunction(name = "info")]
#[pyo3(signature = (message = None, **kwargs))]
pub(super) fn py_info(
    py: Python<'_>,
    message: Option<Bound<'_, PyAny>>,
    kwargs: Option<&Bound<'_, PyDict>>,
) {
    event(py, Level::INFO, message, kwargs);
}

#[pyfunction(name = "debug")]
#[pyo3(signature = (message = None, **kwargs))]
pub(super) fn py_debug(
    py: Python<'_>,
    message: Option<Bound<'_, PyAny>>,
    kwargs: Option<&Bound<'_, PyDict>>,
) {
    event(py, Level::DEBUG, message, kwargs);
}

#[pyfunction(name = "trace")]
#[pyo3(signature = (message = None, **kwargs))]
pub(super) fn py_trace(
    py: Python<'_>,
    message: Option<Bound<'_, PyAny>>,
    kwargs: Option<&Bound<'_, PyDict>>,
) {
    event(py, Level::TRACE, message, kwargs);
}

fn event(
    py: Python,
    level: Level,
    message: Option<Bound<'_, PyAny>>,
    kwargs: Option<&Bound<'_, PyDict>>,
) {
    callsite::do_action(py, level, Kind::EVENT, EventAction { message, kwargs });
}

struct EventAction<'a, 'py> {
    message: Option<Bound<'a, PyAny>>,
    kwargs: Option<&'py Bound<'py, PyDict>>,
}
impl<'a, 'py> CallsiteAction for EventAction<'a, 'py> {
    type ReturnType = ();

    // yes, it's incredibly inefficient and leaks (if used correctly, a fixed amount of) memory for no good reason,
    // but fixing it requires giving up on fmt subscriber's pretty format of kwargs, patching the subscriber,
    // or patching the tracing-core
    fn with_fields_and_values(
        self,
        f: impl FnOnce(&'static [&'static str], &[Option<&dyn Value>]),
    ) {
        let mut fields = vec![];
        let mut values = vec![];

        if self.message.is_some() {
            fields.push("message");
        }

        {
            let mut leaker = Leaker::acquire();

            if let Some(kwargs) = self.kwargs {
                for (key, value) in kwargs.iter() {
                    fields.push(leaker.leak_or_get(key.to_string()));
                    values.push(PyCachedValuable::from(value));
                }
            }
        }

        let fields: &'static [&'static str] = VecLeaker::leak_or_get_once(fields);

        // this vector seems unnecessary
        let values = values
            .iter()
            .map(|x| x as &dyn Valuable)
            .collect::<Vec<_>>();

        let mut values = values
            .iter()
            .map(|x| Some(x as &dyn Value))
            .collect::<Vec<_>>();

        if let Some(message) = self.message {
            // format subscriber formats message=Value::String("text") as "text" instead of text, so fmt::Arguments is used
            // todo: use valuable if message is a list/dict/bool/int/float/null
            // also should I cache the Display? not sure if the performance boost has more impact than the memory allocation overhead
            // though it's probably possible to cached the value only if there's more than 1 active layer,
            // that's more efficient
            let message: CachedValue<_, _, CachedDisplay> =
                CachedValue::from(PyCachedValuable::from(message));
            let args = format_args!("{message}");
            // todo: do not use insert
            values.insert(0, Some(&args as &dyn Value));
            f(fields, &values);
        } else {
            f(fields, &values);
        }
    }

    fn do_if_enabled(
        callsite: &'static impl Callsite,
        values: &[Option<&dyn Value>],
    ) -> Option<()> {
        Event::dispatch(
            callsite.metadata(),
            &callsite.metadata().fields().value_set_all(values),
        );
        Some(())
    }
}
