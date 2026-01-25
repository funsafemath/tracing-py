use pyo3::{
    pyfunction,
    types::{PyDict, PyDictMethods},
    Bound, PyAny, Python,
};
use tracing::{level_filters, Event, Level, Value};
use tracing_core::{Callsite, Kind, LevelFilter};
use valuable::Valuable;

use crate::{
    callsite::get_or_init_callsite,
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
    with_fields_and_values(message, kwargs, |fields, values| {
        // todo: filter by level before calling with_fields_and_values
        // also maybe remove the fields from the callsite id,
        // so filtering by callsite can be done before fields extraction
        let callsite = get_or_init_callsite(py, level, fields, Kind::EVENT);

        // that's a part of the event! macro expansion with the "log" feature off (it's pointless for python)
        let enabled =
            level <= level_filters::STATIC_MAX_LEVEL && level <= LevelFilter::current() && {
                let interest = callsite.interest();
                !interest.is_never()
                // oh not, it's not a stable api
                    && tracing::__macro_support::__is_enabled(callsite.metadata(), interest)
            };

        if enabled {
            Event::dispatch(
                callsite.metadata(),
                &callsite.metadata().fields().value_set_all(values),
            )
        }
    });
}

// yes, it's incredibly inefficient and leaks (if used correctly, fixed amount of) memory for no good reason,
// but fixing it requires giving up on fmt subscriber's pretty format of kwargs, patching the subscriber,
// or patching the tracing-core
fn with_fields_and_values(
    message: Option<Bound<'_, PyAny>>,
    kwargs: Option<&Bound<'_, PyDict>>,
    f: impl FnOnce(&'static [&'static str], &[Option<&dyn Value>]),
) {
    let mut fields = vec![];
    let mut values = vec![];

    if message.is_some() {
        fields.push("message");
    }

    {
        let mut leaker = Leaker::acquire();

        if let Some(kwargs) = kwargs {
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

    if let Some(message) = message {
        // format subscriber formats message=Value::String("text") as "text" instead of text, so fmt::Arguments is used
        // todo: use valuable if message is a list/dict/bool/int/float/null
        let args = format_args!("{}", PyCachedValuable::from(message));
        // todo: do not use insert
        values.insert(0, Some(&args as &dyn Value));
        f(fields, &values);
    } else {
        f(fields, &values);
    }
}
