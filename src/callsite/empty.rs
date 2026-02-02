use std::{
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

use tracing::Metadata;
use tracing_core::{Callsite, Interest};

static NEXT_REF_ADDR: AtomicUsize = AtomicUsize::new(1);

pub(super) struct EmptyCallsite {}

impl EmptyCallsite {
    pub(super) fn new() -> &'static Self {
        let value = NEXT_REF_ADDR.fetch_add(1, Ordering::Relaxed);
        let callsite: *const Self = ptr::without_provenance(value);
        // SAFETY: value is non-null, ZSTs don't need provenance and may be located at dangling addresses
        // also miri says it's ok
        unsafe { callsite.as_ref().expect("usize overflow? really?") }
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
