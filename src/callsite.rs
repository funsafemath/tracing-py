mod default;
mod empty;
mod kind;

use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use pyo3::{
    Bound, Python,
    types::{PyCode, PyFrame},
};
use tracing::{Level, Metadata, Value, field::ValueSet, level_filters};
use tracing_core::{Callsite, Kind, LevelFilter, callsite::DefaultCallsite};

use crate::{
    callsite::{default::CallsiteIdentifier, empty::EmptyCallsite, kind::CallsiteKind},
    inspect::{
        Inspector,
        frame::{PyFrameMethodsExt, UnboundPyFrameMethodsExt},
    },
};

pub(crate) enum Context<'py> {
    FromThreadState(Python<'py>),
    FrameAndCode {
        frame: Bound<'py, PyFrame>,
        code: Bound<'py, PyCode>,
    },
}

impl<'py> Context<'py> {
    fn frame_and_code(self) -> (Bound<'py, PyFrame>, Bound<'py, PyCode>) {
        match self {
            Context::FromThreadState(py) => {
                let frame =
                    PyFrame::from_thread_state(py).expect("must be called from python context");
                let code = frame.code();
                (frame, code)
            }
            Context::FrameAndCode { frame, code } => (frame, code),
        }
    }
}

pub(crate) fn get_or_init_callsite(
    ctx: Context<'_>,
    level: Level,
    fields: &'static [&'static str],
    kind: Kind,
) -> &'static DefaultCallsite {
    // no need to use rwlock/dashmap as we're forced into singlethreaded execution by GIL
    // turns out pyo3 has free-threaded python support, so it may be better to use an rwlock
    static CALLSITES: LazyLock<Mutex<HashMap<CallsiteIdentifier, &'static DefaultCallsite>>> =
        LazyLock::new(Mutex::default);

    let (frame, code) = ctx.frame_and_code();

    let inspector = Inspector::new(&frame);

    let identifier = CallsiteIdentifier::new(
        inspector.ix_address(),
        level,
        fields,
        CallsiteKind::from(kind.clone()),
    );

    CALLSITES
        .lock()
        .unwrap()
        .entry(identifier.clone())
        .or_insert_with(|| default::new_callsite((frame, code), identifier))
}
pub(crate) trait CallsiteAction {
    const KIND: Kind;
    type ReturnType;

    fn with_fields_and_values(
        self,
        f: impl FnOnce(&'static [&'static str], &[Option<&dyn Value>]) -> Option<Self::ReturnType>,
    ) -> Option<Self::ReturnType>;

    fn do_if_enabled(metadata: &'static Metadata, values: &ValueSet) -> Self::ReturnType;
}

fn is_callsite_enabled(callsite: &'static DefaultCallsite) -> bool {
    let interest = callsite.interest();
    !interest.is_never()
                // oh not, it's not a stable api
                    && tracing::__macro_support::__is_enabled(callsite.metadata(), interest)
}

pub(crate) fn do_action<A: CallsiteAction>(
    py: Python,
    level: Level,
    action: A,
    callsite: Option<&'static DefaultCallsite>,
) -> Option<A::ReturnType> {
    if level <= level_filters::STATIC_MAX_LEVEL && level <= LevelFilter::current() {
        if callsite.map(is_callsite_enabled) == Some(false) {
            return None;
        }

        action.with_fields_and_values(|fields, values| {
            // todo: maybe remove the fields from the callsite id,
            // so filtering by callsite can be done before extracting the fields
            let callsite: &DefaultCallsite = match callsite {
                Some(callsite) => callsite,
                None => get_or_init_callsite(Context::FromThreadState(py), level, fields, A::KIND),
            };

            // todo: don't check again if it was already checked
            if is_callsite_enabled(callsite) {
                Some(A::do_if_enabled(
                    callsite.metadata(),
                    &callsite.metadata().fields().value_set_all(values),
                ))
            } else {
                None
            }
        })
    } else {
        None
    }
}
