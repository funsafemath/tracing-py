mod default;
mod empty;
mod kind;

use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use pyo3::Python;
use tracing::{Level, Value, level_filters};
use tracing_core::{Callsite, Kind, LevelFilter, callsite::DefaultCallsite};

use crate::{
    callsite::{default::CallsiteIdentifier, empty::EmptyCallsite, kind::CallsiteKind},
    inspect::{Frame, Inspector},
};

pub(crate) fn get_or_init_callsite(
    py: Python,
    level: Level,
    fields: &'static [&'static str],
    kind: Kind,
) -> &'static DefaultCallsite {
    // no need to use rwlock/dashmap as we're forced into singlethreaded execution by GIL
    // turns out pyo3 has free-threaded python support, so it may be better to use an rwlock
    static CALLSITES: LazyLock<Mutex<HashMap<CallsiteIdentifier, &'static DefaultCallsite>>> =
        LazyLock::new(Mutex::default);

    let frame = Frame::new(py);
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
        // TODO: add fields
        .or_insert_with(|| default::new_callsite(inspector, identifier))
}

pub(crate) trait CallsiteAction {
    type ReturnType;

    fn with_fields_and_values(
        self,
        f: impl FnOnce(&'static [&'static str], &[Option<&dyn Value>]) -> Option<Self::ReturnType>,
    ) -> Option<Self::ReturnType>;

    fn do_if_enabled(
        callsite: &'static impl Callsite,
        values: &[Option<&dyn Value>],
    ) -> Self::ReturnType;
}

pub(crate) fn do_action<A: CallsiteAction>(
    py: Python,
    level: Level,
    kind: Kind,
    action: A,
) -> Option<A::ReturnType> {
    if level <= level_filters::STATIC_MAX_LEVEL && level <= LevelFilter::current() {
        action.with_fields_and_values(|fields, values| {
            // todo: maybe remove the fields from the callsite id,
            // so filtering by callsite can be done before extracting the fields
            let callsite = get_or_init_callsite(py, level, fields, kind);

            let enabled = {
                let interest = callsite.interest();
                !interest.is_never()
                // oh not, it's not a stable api
                    && tracing::__macro_support::__is_enabled(callsite.metadata(), interest)
            };

            if enabled {
                Some(A::do_if_enabled(callsite, values))
            } else {
                None
            }
        })
    } else {
        None
    }
}
