use std::{fmt::Display, marker::PhantomData};

use pyo3::{
    prelude::*,
    types::{PyBool, PyFloat, PyInt},
};
use valuable::{Valuable, Value};

use crate::{
    cached::{CachedValuable, CachedValue, GetValue},
    ext::template::PyTemplateMethodsExt,
    template::template_string::PyTemplate,
};

pub(crate) trait StrFmt {
    fn make(str: &str) -> Value<'_>;
}

pub(crate) enum QuotedString {}
pub(crate) enum UnquotedString {}

impl StrFmt for QuotedString {
    fn make(str: &str) -> Value<'_> {
        Value::String(str)
    }
}

impl StrFmt for UnquotedString {
    fn make(str: &str) -> Value<'_> {
        Value::UnquotedString(str)
    }
}

pub(crate) type PyCachedValuable<'py, S> =
    CachedValue<OwnedValuable<S>, Bound<'py, PyAny>, CachedValuable>;

pub(crate) enum OwnedValuable<S: StrFmt> {
    SmallInt(i128),
    Float(f64),
    Bool(bool),
    None,
    Str(String, PhantomData<S>),
}

impl<S: StrFmt> Valuable for OwnedValuable<S> {
    fn as_value(&self) -> Value<'_> {
        match self {
            Self::SmallInt(int) => Value::I128(*int),
            Self::Float(float) => Value::F64(*float),
            Self::Bool(bool) => Value::Bool(*bool),
            Self::None => Value::Unit,
            Self::Str(str, _) => S::make(str),
        }
    }

    fn visit(&self, visit: &mut dyn valuable::Visit) {
        self.as_value().visit(visit);
    }
}

// deferring any type checks until the value is required, so we don't waste time on filtered events
impl<'py, S: StrFmt> GetValue<OwnedValuable<S>, CachedValuable> for Bound<'py, PyAny> {
    fn value(&self) -> OwnedValuable<S> {
        if self.is_none() {
            OwnedValuable::None
        } else if let Ok(int) = self.cast_exact::<PyInt>()
            && let Ok(int) = int.extract::<i128>()
        {
            // it's possible to improve bigint conversion speed,
            // even extracting a num-bigint and converting it to a string is faster than python's string conversion function,
            // and rug (gmp) is even faster
            OwnedValuable::SmallInt(int)
        } else if let Ok(float) = self.cast_exact::<PyFloat>() {
            OwnedValuable::Float(PyFloatMethods::value(float))
        } else if let Ok(bool) = self.cast_exact::<PyBool>() {
            OwnedValuable::Bool(bool.is_true())
        } else if let Ok(tmpl) = self.cast_exact::<PyTemplate>() {
            OwnedValuable::<S>::Str(tmpl.format(), PhantomData)
        } else {
            OwnedValuable::<S>::Str(self.to_string(), PhantomData)
        }
    }
}
