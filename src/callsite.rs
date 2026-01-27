mod default;
mod empty;
mod kind;

use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use pyo3::Python;
use tracing::Level;
use tracing_core::{Kind, callsite::DefaultCallsite};

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
