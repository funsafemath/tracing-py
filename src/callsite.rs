use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use pyo3::Python;
use tracing::{field::FieldSet, Level, Metadata};
use tracing_core::{
    callsite::{DefaultCallsite, Identifier},
    Callsite, Interest, Kind,
};

use crate::inspect::Frame;

fn leak<T>(x: T) -> &'static T {
    Box::leak(Box::new(x))
}

#[derive(Debug)]
struct EmptyCallsite {
    // Identifiers depend on address, and identifiers must be unique, so we can't use zero-sized types,
    // as addresses may be reused after leaking
    //
    // also we can use addr of a leaked EmptyCallsite for the first DefaultCallsite,
    //  and then use addr of previous callsite, reducing memory leaks, but this does not really matter
    //
    // todo: patch tracing-core or something, i don't like this unholy abomination
    _byte: u8,
}

impl EmptyCallsite {
    fn new() -> &'static Self {
        leak(EmptyCallsite { _byte: 0 })
    }
}

impl Callsite for EmptyCallsite {
    fn set_interest(&self, _: Interest) {
        panic!("can't register empty callsite")
    }

    fn metadata(&self) -> &crate::Metadata<'_> {
        panic!("can't access empty callsite")
    }
}
fn new_callsite(
    py: Python,
    level: Level,
    fields: &'static [&'static str],
    kind: Kind,
) -> &'static DefaultCallsite {
    let frame = Frame::new(py);

    let line = u32::try_from(frame.line_number()).expect("negative line number?");

    let name = frame.code().filename();
    let name = leak(format!("event {}", name.to_string_lossy(py)));

    let empty_callsite = EmptyCallsite::new();

    // todo: get relative paths somehow?
    let meta = leak(Metadata::new(
        name,
        "TODO",
        level,
        Some(name),
        Some(line),
        Some("TODO"),
        FieldSet::new(fields, Identifier(empty_callsite)),
        kind,
    ));
    leak(DefaultCallsite::new(meta))
}

// no need to use rwlock/dashmap as we're forced into singlethreaded execution by GIL
static CALLSITES: LazyLock<Mutex<HashMap<CallsiteIdentifier, &'static DefaultCallsite>>> =
    LazyLock::new(Mutex::default);

#[derive(Hash, PartialEq, Eq)]
enum CallsiteKind {
    Event,
    Span,
    Hint,
}

impl CallsiteKind {
    fn from_kind(kind: Kind) -> Self {
        if kind.is_event() {
            Self::Event
        } else if kind.is_span() {
            Self::Span
        } else if kind.is_hint() {
            Self::Hint
        } else {
            panic!("unknown callsite kind: {kind:?}")
        }
    }
}

pub(crate) fn get_or_init_callsite(
    py: Python,
    level: Level,
    kind: Kind,
) -> &'static DefaultCallsite {
    let frame = Frame::new(py);

    let identifier = CallsiteIdentifier {
        address: frame.ix_address(),
        level,
        // TODO:  add fields
        fields: &["message"],
        kind: CallsiteKind::from_kind(kind.clone()),
    };

    CALLSITES
        .lock()
        .unwrap()
        .entry(identifier)
        // TODO: add fields
        .or_insert_with(|| new_callsite(py, level, &["message"], kind))
}

// a single address can contain multiple callsites,
// since i can't make python code use only a single event level or constast fields
// filename, module name and line number hopefully stay constant
#[derive(Hash, PartialEq, Eq)]
pub(crate) struct CallsiteIdentifier {
    address: usize,
    level: Level,
    fields: &'static [&'static str],
    kind: CallsiteKind,
}
