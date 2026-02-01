use pyo3::{
    Bound, PyAny, PyResult, Python,
    exceptions::PyTypeError,
    pyfunction,
    types::{PyDict, PyDictMethods, PyTuple},
};
use tracing::{Event, Level, Metadata, Value, field::ValueSet};
use tracing_core::{Kind, callsite::DefaultCallsite};
use valuable::Valuable;

use crate::{
    cached::{CachedDisplay, CachedValue},
    callsite::{self, CallsiteAction},
    formatting::{
        percent::PercentFormatted,
        valuable::{PyCachedValuable, QuotedString, UnquotedString},
    },
    leak::{Leaker, VecLeaker},
};

macro_rules! py_event {
    ($fn_name:ident, $py_name:literal, $lvl:expr) => {
        #[pyfunction(name = $py_name)]
        #[pyo3(signature = (message = None, fmt_args = None, **kwargs))]
        pub(super) fn $fn_name(
            py: Python<'_>,
            message: Option<Bound<'_, PyAny>>,
            fmt_args: Option<&Bound<'_, PyTuple>>,
            kwargs: Option<&Bound<'_, PyDict>>,
        ) -> PyResult<()> {
            event(py, $lvl, message, fmt_args, kwargs)
        }
    };
}

py_event!(py_trace, "trace", Level::TRACE);
py_event!(py_debug, "debug", Level::DEBUG);
py_event!(py_info, "info", Level::INFO);
py_event!(py_warn, "warn", Level::WARN);
py_event!(py_error, "error", Level::ERROR);

pub(crate) fn event(
    py: Python,
    level: Level,
    message: Option<Bound<'_, PyAny>>,
    fmt_args: Option<&Bound<'_, PyTuple>>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    let message = match fmt_args {
        Some(fmt_args) => {
            let Some(message) = message else {
                // not using positional-only arguments to forbid passing "message" kwarg accidentally
                return Err(PyTypeError::new_err(
                    "a message string must be passed when using fmt_args",
                ));
            };
            Message::PercentFormatted(PercentFormatted::new(
                message.cast_into().map_err(|x| {
                    PyTypeError::new_err(format!(
                        "if fmt_args is passed, the first argument must be a string: {x}"
                    ))
                })?,
                fmt_args,
            ))
        }
        None => Message::Any(message),
    };
    callsite::do_action(py, level, EventAction { message, kwargs }, None);
    Ok(())
}

pub enum Message<'a, 'py> {
    Any(Option<Bound<'py, PyAny>>),
    PercentFormatted(PercentFormatted<'a, 'py>),
}

pub(crate) struct EventAction<'a, 'py> {
    message: Message<'a, 'py>,
    kwargs: Option<&'a Bound<'py, PyDict>>,
}

pub(crate) fn leak_or_get_kwargs<'py>(
    // todo: accept a mut ref
    leaker: Option<Leaker>,
    kwargs: Option<&Bound<'py, PyDict>>,
) -> (Vec<&'static str>, Vec<PyCachedValuable<'py, QuotedString>>) {
    let mut fields = vec![];
    let mut values = vec![];

    if let Some(kwargs) = kwargs {
        let mut leaker = leaker.unwrap_or(Leaker::acquire());

        for (key, value) in kwargs.iter() {
            fields.push(leaker.leak_or_get(key.to_string()));
            values.push(PyCachedValuable::<QuotedString>::from(value));
        }
    }

    (fields, values)
}

impl<'a, 'py> CallsiteAction for EventAction<'a, 'py> {
    const KIND: Kind = Kind::EVENT;
    type ReturnType = ();

    // yes, it's incredibly inefficient and leaks (if used correctly, a fixed amount of) memory for no good reason,
    // but fixing it requires giving up on fmt subscriber's pretty format of kwargs, patching the subscriber,
    // or patching the tracing-core
    fn with_fields_and_values(
        self,
        f: impl FnOnce(&'static [&'static str], &[Option<&dyn Value>]) -> Option<()>,
    ) -> Option<()> {
        let (mut fields, values) = leak_or_get_kwargs(None, self.kwargs);

        match self.message {
            Message::Any(Some(_)) | Message::PercentFormatted(_) => {
                // todo: do not use insert
                fields.insert(0, "message");
            }
            Message::Any(None) => {}
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

        match self.message {
            Message::Any(None) => f(fields, &values),
            Message::Any(Some(message)) => {
                let message = PyCachedValuable::<UnquotedString>::from(message);
                let message = &message as &dyn Valuable;
                values.insert(0, Some(&message as &dyn Value));
                f(fields, &values)
            }
            Message::PercentFormatted(percent_formatted) => {
                let message: CachedValue<_, _, CachedDisplay> =
                    CachedValue::from(percent_formatted);
                let args = format_args!("{message}");
                // todo: do not use insert
                values.insert(0, Some(&args as &dyn Value));
                f(fields, &values)
            }
        }
    }

    fn do_if_enabled(metadata: &'static Metadata, values: &ValueSet) -> Self::ReturnType {
        Event::dispatch(metadata, values);
    }
}

pub(crate) fn ret_eve(
    py: Python,
    level: Level,
    message: Message<'_, '_>,
    kwargs: Option<&Bound<'_, PyDict>>,
    callsite: Option<&'static DefaultCallsite>,
) -> Option<()> {
    callsite::do_action(py, level, EventAction { message, kwargs }, callsite)
}
