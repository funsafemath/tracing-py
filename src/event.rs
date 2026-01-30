use std::fmt::Display;

use pyo3::{
    Bound, PyAny, PyResult, Python,
    exceptions::PyTypeError,
    pyfunction,
    types::{PyAnyMethods, PyDict, PyDictMethods, PyString, PyTuple},
};
use tracing::{Event, Level, Metadata, Value, field::ValueSet};
use tracing_core::Kind;
use valuable::Valuable;

use crate::{
    cached::{CachedDisplay, CachedValue},
    callsite::{self, CallsiteAction},
    formatting::valuable::PyCachedValuable,
    leak::{Leaker, VecLeaker},
};

macro_rules! py_event {
    ($fn_name:ident, $py_name:literal, $lvl:expr) => {
        #[pyfunction(name = $py_name)]
        #[pyo3(signature = (message = None, fmt_args = None, **kwargs))]
        pub(super) fn $fn_name(
            py: Python<'_>,
            message: Option<Bound<'_, PyAny>>,
            fmt_args: Option<Bound<'_, PyTuple>>,
            kwargs: Option<&Bound<'_, PyDict>>,
        ) -> PyResult<()> {
            event(py, $lvl, message, fmt_args, kwargs)
        }
    };
}

py_event!(py_error, "error", Level::ERROR);
py_event!(py_warn, "warn", Level::WARN);
py_event!(py_info, "info", Level::INFO);
py_event!(py_debug, "debug", Level::ERROR);
py_event!(py_trace, "trace", Level::TRACE);

fn event(
    py: Python,
    level: Level,
    message: Option<Bound<'_, PyAny>>,
    fmt_args: Option<Bound<'_, PyTuple>>,
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
            Message::PercentFormatted(PercentFormatted {
                message: message.cast_into()?,
                args: fmt_args,
            })
        }
        None => Message::Any(message),
    };
    callsite::do_action(py, level, EventAction { message, kwargs }, None);
    Ok(())
}

// Args are intentionally PyTuple, not a PyAny, even though you can pass a single non-tuple argument to str.__mod__
// IMO accepting PyAny is a bad design on the Python side, as it's inconsistent and makes a bad function signature;
// also if you try to use a list instead of a tuple, you'll get a runtime error
//
// Accepting a tuple here also makes the users much less likely to make mistakes like using info(some string object, obj2),
// as type-checker will complain that obj2 is not a tuple.
// TODO: should percent formatting be moved to a submodule and be accessible like tracing.legacy.function?
// That'll reduce the number possible mistakes and also allow to use *args to log multiple arbitrary objects
struct PercentFormatted<'py> {
    message: Bound<'py, PyString>,
    args: Bound<'py, PyTuple>,
}

// PyUnicode_Format may be a bit faster, rewrite this function using it if you have nothing better to do
impl<'py> Display for PercentFormatted<'py> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Is there any way to gracefully handle the error?
        // returning a fmt error panics,
        // and I can't check if arguments are valid for a given string before fmt() is called
        // (that's the point of lazy formatting)
        write!(
            f,
            "{}",
            self.message
                .rem(&self.args,)
                .expect("failed to format a string")
        )
    }
}
enum Message<'py> {
    Any(Option<Bound<'py, PyAny>>),
    PercentFormatted(PercentFormatted<'py>),
}

struct EventAction<'a, 'py> {
    message: Message<'py>,
    kwargs: Option<&'a Bound<'py, PyDict>>,
}

pub(crate) fn leak_or_get_kwargs<'py>(
    // todo: accept a mut ref
    leaker: Option<Leaker>,
    kwargs: Option<&Bound<'py, PyDict>>,
) -> (Vec<&'static str>, Vec<PyCachedValuable<'py>>) {
    let mut fields = vec![];
    let mut values = vec![];

    if let Some(kwargs) = kwargs {
        let mut leaker = leaker.unwrap_or(Leaker::acquire());

        for (key, value) in kwargs.iter() {
            fields.push(leaker.leak_or_get(key.to_string()));
            values.push(PyCachedValuable::from(value));
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

        // todo: do not use insert
        match self.message {
            Message::Any(Some(_)) | Message::PercentFormatted(_) => {
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
            Message::Any(bound) => match bound {
                Some(message) => {
                    // format subscriber formats message=Value::String("text") as "text" instead of text, so fmt::Arguments is used
                    // todo: use valuable if message is a list/dict/bool/int/float/null
                    // also should I cache the Display? not sure if the performance boost has more impact than the memory allocation overhead
                    // though it's probably possible to cached the value only if there's more than 1 active layer,
                    // that's more efficient

                    // on a second look, it looks like that subscribers never call fmt repeatedly?
                    let message: CachedValue<_, _, CachedDisplay> =
                        CachedValue::from(PyCachedValuable::from(message));
                    let args = format_args!("{message}");
                    // todo: do not use insert
                    values.insert(0, Some(&args as &dyn Value));
                    f(fields, &values)
                }
                None => f(fields, &values),
            },
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
