use pyo3::{pyfunction, types::PyString, Py, Python};
use tracing::{level_filters, Event, Level, Value};
use tracing_core::{Callsite, Kind, LevelFilter};

use crate::callsite::get_or_init_callsite;

fn event(py: Python, level: Level, message: Py<PyString>) {
    let callsite = get_or_init_callsite(py, level, Kind::EVENT);

    // that's a part of the event! macro expansion with the "log" feature off (it's pointless for python)
    let enabled = level <= level_filters::STATIC_MAX_LEVEL && level <= LevelFilter::current() && {
        let interest = callsite.interest();
        !interest.is_never()
            && tracing::__macro_support::__is_enabled(callsite.metadata(), interest)
    };

    if enabled {
        Event::dispatch(
            callsite.metadata(),
            &callsite
                .metadata()
                .fields()
                .value_set_all(&[(Some(&format_args!("{message}") as &dyn Value))]),
        );
    }
}

#[pyfunction(name = "error")]
pub(super) fn py_error(py: Python, message: Py<PyString>) {
    event(py, Level::ERROR, message);
}

#[pyfunction(name = "warn")]
pub(super) fn py_warn(py: Python, message: Py<PyString>) {
    event(py, Level::WARN, message);
}

#[pyfunction(name = "info")]
pub(super) fn py_info(py: Python, message: Py<PyString>) {
    event(py, Level::INFO, message);
}

#[pyfunction(name = "debug")]
pub(super) fn py_debug(py: Python, message: Py<PyString>) {
    event(py, Level::DEBUG, message);
}

#[pyfunction(name = "trace")]
pub(super) fn py_trace(py: Python, message: Py<PyString>) {
    event(py, Level::TRACE, message);
}
