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

    let code = frame.code();

    let filename = leak(code.filename().to_string_lossy(py).into_owned());

    let target = code.target();
    let target = leak(target.to_string_lossy(py).into_owned());

    // frame.module();

    let empty_callsite = EmptyCallsite::new();

    // todo: get relative paths somehow?
    let meta = leak(Metadata::new(
        leak(format!("event {}", filename)),
        target,
        level,
        Some(leak(frame.module())),
        Some(line),
        Some(leak(frame.module())),
        FieldSet::new(fields, Identifier(empty_callsite)),
        Kind::from(kind),
    ));
    leak(DefaultCallsite::new(meta))
}
