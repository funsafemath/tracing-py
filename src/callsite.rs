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

fn leak<T>(x: T) -> &'static T {
    Box::leak(Box::new(x))
}

pub(crate) fn get_or_init_callsite(
    py: Python,
    level: Level,
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
        &["message"],
        CallsiteKind::from(kind.clone()),
    );

    // copying 40 bytes is cheap, but since it's done in the happy path, it may be worth to rewrite this function
    // to save a billionth of a µs (and maybe not if compiler optimizes it)
    CALLSITES
        .lock()
        .unwrap()
        .entry(identifier.clone())
        // TODO: add fields
        .or_insert_with(|| default::new_callsite(inspector, identifier))
}
