use pyo3::Python;
use tracing::{field::FieldSet, Level, Metadata};
use tracing_core::{
    callsite::{DefaultCallsite, Identifier},
    Kind,
};

use crate::{
    callsite::{leak, CallsiteKind, EmptyCallsite},
    inspect::Frame,
};

// a single address can contain multiple callsites,
// since i can't make python code use only a single event level or constast fields
// filename, module name and line number hopefully stay constant
#[derive(Hash, PartialEq, Eq, Clone)]
pub(crate) struct CallsiteIdentifier {
    address: usize,
    level: Level,
    fields: &'static [&'static str],
    kind: CallsiteKind,
}

impl CallsiteIdentifier {
    pub(super) fn new(
        address: usize,
        level: Level,
        fields: &'static [&'static str],
        kind: CallsiteKind,
    ) -> Self {
        Self {
            address,
            level,
            fields,
            kind,
        }
    }
}

pub(super) fn new_callsite(
    py: Python,
    CallsiteIdentifier {
        level,
        fields,
        kind,
        ..
    }: CallsiteIdentifier,
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
        Kind::from(kind),
    ));
    leak(DefaultCallsite::new(meta))
}
