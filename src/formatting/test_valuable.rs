use std::fmt::Display;

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

// pub struct PyValuable<'py>(Bound<'py, PyAny>);
pub(crate) type PyCachedValuable<'py> =
    CachedValue<OwnedValuable, Bound<'py, PyAny>, CachedValuable>;
// pub(crate) enum PyValuableMarker {}
pub(crate) enum OwnedValuable {
    SmallInt(i128),
    Float(f64),
    Bool(bool),
    None,
    Str(String),
}

impl Valuable for OwnedValuable {
    fn as_value(&self) -> Value<'_> {
        match self {
            OwnedValuable::SmallInt(int) => Value::I128(*int),
            OwnedValuable::Float(float) => Value::F64(*float),
            OwnedValuable::Bool(bool) => Value::Bool(*bool),
            OwnedValuable::None => Value::Unit,
            OwnedValuable::Str(str) => Value::String(str),
        }
    }

    fn visit(&self, visit: &mut dyn valuable::Visit) {
        self.as_value().visit(visit);
    }
}

// deferring any type checks until the value is required, so we don't waste time on filtered events
impl<'py> GetValue<OwnedValuable, CachedValuable> for Bound<'py, PyAny> {
    fn value(&self) -> OwnedValuable {
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
            OwnedValuable::Str(tmpl.format())
        } else {
            OwnedValuable::Str(self.to_string())
        }
    }
}

impl<'py> Display for PyCachedValuable<'py> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{:?}", self.as_value())
        match self.as_value() {
            Value::String(str) => write!(f, "{str}"),
            other => write!(f, "{other:?}"),
        }
    }
}
