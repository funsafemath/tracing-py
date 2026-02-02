use pyo3::prelude::*;
use tracing::{Level, Metadata, Span, Value, field::ValueSet};
use tracing_core::{Kind, callsite::DefaultCallsite};
use valuable::Valuable;

use crate::{
    callsite::{self, CallsiteAction},
    formatting::valuable::{PyCachedValuable, QuoteStrAndTmpl, TemplateRepr},
};

struct SpanAction<'py> {
    fields: &'static [&'static str],
    values: Vec<Bound<'py, PyAny>>,
}

impl CallsiteAction for SpanAction<'_> {
    const KIND: Kind = Kind::SPAN;
    type ReturnType = Span;

    // leaks the data for no good reason, see the comment above [crate::event::EventAction::with_fields_and_values]
    fn with_fields_and_values(
        self,
        f: impl FnOnce(&'static [&'static str], &[Option<&dyn Value>]) -> Option<Span>,
    ) -> Option<Span> {
        let values = self
            .values
            .into_iter()
            .map(PyCachedValuable::<QuoteStrAndTmpl, TemplateRepr>::from)
            .collect::<Vec<_>>();

        let values = values
            .iter()
            .map(|x| x as &dyn Valuable)
            .collect::<Vec<_>>();

        let values = values
            .iter()
            .map(|x| Some(x as &dyn Value))
            .collect::<Vec<_>>();

        f(self.fields, &values)
    }

    fn do_if_enabled(metadata: &'static Metadata, values: &ValueSet) -> Self::ReturnType {
        Span::new(metadata, values)
    }
}

pub(crate) fn span(
    py: Python,
    level: Level,
    fields: &'static [&'static str],
    values: Vec<Bound<'_, PyAny>>,
    callsite: &'static DefaultCallsite,
) -> Option<Span> {
    callsite::do_action(py, level, SpanAction { fields, values }, Some(callsite))
}
