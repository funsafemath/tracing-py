mod default;
mod empty;
mod kind;

use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use pyo3::Python;
use tracing::Level;
use tracing_core::{callsite::DefaultCallsite, Kind};

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

// fn emit(py: Python, level: Level, message: Py<PyString>, kind: Kind) {
//     let callsite = get_or_init_callsite(py, level, kind);

//     // that's a part of the event! macro expansion with the "log" feature off (it's pointless for python)
//     let enabled = level <= level_filters::STATIC_MAX_LEVEL && level <= LevelFilter::current() && {
//         let interest = callsite.interest();
//         !interest.is_never()
//             && tracing::__macro_support::__is_enabled(callsite.metadata(), interest)
//     };

//     if enabled {
//         Event::dispatch(
//             callsite.metadata(),
//             &callsite
//                 .metadata()
//                 .fields()
//                 .value_set_all(&[(Some(&format_args!("{message}") as &dyn Value))]),
//         );
//     }
// }
