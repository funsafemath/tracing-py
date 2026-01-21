use tracing_core::Kind;

#[derive(Hash, PartialEq, Eq, Clone)]
pub(super) enum CallsiteKind {
    Event,
    Span,
    Hint,
}

impl From<Kind> for CallsiteKind {
    fn from(value: Kind) -> Self {
        if value.is_event() {
            Self::Event
        } else if value.is_span() {
            Self::Span
        } else if value.is_hint() {
            Self::Hint
        } else {
            // yes, From<T> generally shouldn't panic, but python code can't create an invalid value
            // and call this function directly, so it basically never panics
            panic!("unknown callsite kind: {value:?}")
        }
    }
}

impl From<CallsiteKind> for Kind {
    fn from(value: CallsiteKind) -> Self {
        match value {
            CallsiteKind::Event => Kind::EVENT,
            CallsiteKind::Span => Kind::SPAN,
            CallsiteKind::Hint => Kind::HINT,
        }
    }
}
