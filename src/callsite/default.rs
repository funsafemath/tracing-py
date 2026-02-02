use pyo3::{
    Bound,
    types::{PyCode, PyFrame, PyStringMethods},
};
use tracing::{Level, Metadata, field::FieldSet};
use tracing_core::{
    Kind,
    callsite::{DefaultCallsite, Identifier},
};

use crate::{
    callsite::{CallsiteKind, EmptyCallsite},
    ext::{code::PyCodeMethodsExt, frame::PyFrameMethodsExt},
    introspect::Inspector,
    leak::{Leaker, leak},
};

// a single address can contain multiple callsites,
// since i can't make python code use only a single event level or constast fields
// filename, module name and line number hopefully stay constant
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct CallsiteIdentifier {
    address: usize,
    level: Level,
    fields: &'static [&'static str],
    kind: CallsiteKind,
}

impl CallsiteIdentifier {
    pub fn new(
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

pub fn new_callsite(
    (frame, code): (Bound<'_, PyFrame>, Bound<'_, PyCode>),
    CallsiteIdentifier {
        level,
        fields,
        kind,
        ..
    }: CallsiteIdentifier,
) -> &'static DefaultCallsite {
    // for instrument() we use invocation line number as the line number (not function line number),
    // which is consistent with the tracing crate
    let line = u32::try_from(frame.line_number()).expect("negative line number?");

    let meta = {
        let mut leaker = Leaker::acquire();

        let filename = leaker.leak_or_get(code.filename().to_string_lossy().into_owned());

        let target = code.qualname();
        let target = leaker.leak_or_get(target.to_string_lossy().into_owned());

        let empty_callsite = EmptyCallsite::new();

        let name = leaker.leak_or_get(match kind {
            CallsiteKind::Event => format!("event {filename}"),
            CallsiteKind::Span => code.name().to_string(),
            CallsiteKind::Hint => unimplemented!(),
        });

        let inspector = Inspector::new(&frame);

        leak(Metadata::new(
            name,
            target,
            level,
            Some(leaker.leak_or_get(inspector.module())),
            Some(line),
            Some(leaker.leak_or_get(inspector.module())),
            FieldSet::new(fields, Identifier(empty_callsite)),
            Kind::from(kind),
        ))
    };

    leak(DefaultCallsite::new(meta))
}
