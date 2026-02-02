use pyo3::{
    Bound, PyAny, PyResult, Python,
    exceptions::PyTypeError,
    pyfunction,
    types::{PyDict, PyDictMethods, PyTuple},
};
use tracing::{Event, Level, Metadata, Value, field::ValueSet};
use tracing_core::{Callsite, Kind, callsite::DefaultCallsite};
use valuable::Valuable;

use crate::{
    cached::{CachedDisplay, CachedValue},
    callsite::{self, CallsiteAction, Context},
    formatting::{
        percent::PercentFormatted,
        valuable::{
            NeverQuote, PyCachedValuable, QuoteStrAndTmpl, TemplateInterpolate, TemplateRepr,
        },
    },
    leak::{Leaker, VecLeaker},
};

// macro 2.0 not usable due to strict hygiene?
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

fn event(
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

enum Message<'a, 'py> {
    Any(Option<Bound<'py, PyAny>>),
    PercentFormatted(PercentFormatted<'a, 'py>),
}

struct EventAction<'a, 'py> {
    message: Message<'a, 'py>,
    kwargs: Option<&'a Bound<'py, PyDict>>,
}

pub(crate) fn leak_or_get_kwargs<'py>(
    // todo: accept a mut ref
    leaker: Option<Leaker>,
    kwargs: Option<&Bound<'py, PyDict>>,
) -> (
    Vec<&'static str>,
    Vec<PyCachedValuable<'py, QuoteStrAndTmpl, TemplateInterpolate>>,
) {
    let mut fields = vec![];
    let mut values = vec![];

    if let Some(kwargs) = kwargs {
        let mut leaker = leaker.unwrap_or(Leaker::acquire());

        for (key, value) in kwargs.iter() {
            fields.push(leaker.leak_or_get(key.to_string()));
            values.push(PyCachedValuable::<QuoteStrAndTmpl, TemplateInterpolate>::from(value));
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
                let message = PyCachedValuable::<NeverQuote, TemplateInterpolate>::from(message);
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

macro single_field_event($struct_name:ident, $fn_create_name:ident, $fn_emit_name:ident, $field:literal) {
    #[derive(Clone, Copy, Debug)]
    pub(crate) struct $struct_name(&'static DefaultCallsite);

    struct Action<'py>(Bound<'py, PyAny>);

    static FIELDS: &[&str] = &[$field];

    impl<'py> CallsiteAction for Action<'py> {
        const KIND: Kind = Kind::EVENT;

        type ReturnType = ();

        fn with_fields_and_values(
            self,
            f: impl FnOnce(
                &'static [&'static str],
                &[Option<&dyn Value>],
            ) -> Option<Self::ReturnType>,
        ) -> Option<Self::ReturnType> {
            let value = PyCachedValuable::<QuoteStrAndTmpl, TemplateRepr>::from(self.0);
            f(FIELDS, &[Some(&(&value as &dyn Valuable) as &dyn Value)])
        }

        fn do_if_enabled(metadata: &'static Metadata, values: &ValueSet) -> Self::ReturnType {
            Event::dispatch(metadata, values);
        }
    }

    pub(crate) fn $fn_create_name(context: Context, level: Level) -> $struct_name {
        $struct_name(callsite::get_or_init_callsite(
            context,
            level,
            FIELDS,
            Kind::EVENT,
        ))
    }

    pub(crate) fn $fn_emit_name(
        py: Python,
        value: Bound<'_, PyAny>,
        callsite: $struct_name,
    ) -> Option<()> {
        callsite::do_action(
            py,
            callsite.0.metadata().level().to_owned(),
            Action(value),
            Some(callsite.0),
        )
    }
}

single_field_event!(RetCallsite, ret_callsite, ret_event, "return");
single_field_event!(ErrCallsite, err_callsite, err_event, "exception");
single_field_event!(YieldCallsite, yield_callsite, yield_event, "yield");
