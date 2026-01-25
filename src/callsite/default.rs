use tracing::{field::FieldSet, Level, Metadata};
use tracing_core::{
    callsite::{DefaultCallsite, Identifier},
    Kind,
};

use crate::{
    callsite::{CallsiteKind, EmptyCallsite},
    inspect::Inspector,
    leak::{leak, Leaker},
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
    inspector: Inspector,
    CallsiteIdentifier {
        level,
        fields,
        kind,
        ..
    }: CallsiteIdentifier,
) -> &'static DefaultCallsite {
    let frame = &inspector.frame;
    let code = &inspector.code;
    let py = inspector.py;

    let line = u32::try_from(frame.line_number()).expect("negative line number?");

    let mut leaker = Leaker::acquire();

    let filename = leaker.leak_or_get(code.filename().to_string_lossy(py).into_owned());

    let target = code.target();
    let target = leaker.leak_or_get(target.to_string_lossy(py).into_owned());

    let empty_callsite = EmptyCallsite::new();

    let meta = leak(Metadata::new(
        leaker.leak_or_get(format!("event {}", filename)),
        target,
        level,
        Some(leaker.leak_or_get(inspector.module())),
        Some(line),
        Some(leaker.leak_or_get(inspector.module())),
        FieldSet::new(fields, Identifier(empty_callsite)),
        Kind::from(kind),
    ));
    leak(DefaultCallsite::new(meta))
}
