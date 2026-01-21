use tracing::Metadata;
use tracing_core::{Callsite, Interest};

use crate::callsite::leak;

#[derive(Debug)]
pub(super) struct EmptyCallsite {
    // Identifiers depend on address, and identifiers must be unique, so we can't use zero-sized types,
    // as addresses may be reused after leaking
    //
    // we also can use addr of a leaked EmptyCallsite for the first DefaultCallsite,
    // and then use addr of previous callsite, reducing memory leaks, but this does not really matter
    //
    // todo: patch tracing-core or something, i don't like this unholy abomination
    _byte: u8,
}

impl EmptyCallsite {
    pub(super) fn new() -> &'static Self {
        leak(EmptyCallsite { _byte: 0 })
    }
}

impl Callsite for EmptyCallsite {
    fn set_interest(&self, _: Interest) {
        panic!("can't register empty callsite")
    }

    fn metadata(&self) -> &Metadata<'_> {
        panic!("can't access empty callsite")
    }
}
