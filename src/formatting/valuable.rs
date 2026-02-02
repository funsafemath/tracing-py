use std::marker::PhantomData;

use pyo3::{
    prelude::*,
    types::{PyBool, PyFloat, PyInt, PyString},
};
use valuable::{Valuable, Value};

use crate::{
    cached::{CachedValuable, CachedValue, GetValue},
    ext::template::PyTemplateMethodsExt,
    template::template_string::PyTemplate,
};

pub trait StrFmt {
    fn make<T: TemplateFmt>(string: String) -> OwnedValuable<Self, T>
    where
        Self: Sized;
}

pub enum QuoteStrAndTmpl {}
pub enum NeverQuote {}

impl StrFmt for QuoteStrAndTmpl {
    fn make<T: TemplateFmt>(str: String) -> OwnedValuable<Self, T> {
        OwnedValuable::Str(str, PhantomData, PhantomData)
    }
}

impl StrFmt for NeverQuote {
    fn make<T: TemplateFmt>(string: String) -> OwnedValuable<Self, T> {
        OwnedValuable::UnquotedStr(string, PhantomData)
    }
}

pub trait TemplateFmt {
    fn make<S: StrFmt>(template: &Bound<'_, PyTemplate>) -> OwnedValuable<S, Self>
    where
        Self: Sized;
}

pub enum TemplateInterpolate {}
pub enum TemplateRepr {}

impl TemplateFmt for TemplateInterpolate {
    fn make<S: StrFmt>(template: &Bound<'_, PyTemplate>) -> OwnedValuable<S, Self>
    where
        Self: Sized,
    {
        S::make(template.format())
    }
}

impl TemplateFmt for TemplateRepr {
    fn make<S>(template: &Bound<'_, PyTemplate>) -> OwnedValuable<S, Self>
    where
        Self: Sized,
    {
        OwnedValuable::UnquotedStr::<S, Self>(template.to_string(), PhantomData)
    }
}

pub type PyCachedValuable<'py, S, T> =
    CachedValue<OwnedValuable<S, T>, Bound<'py, PyAny>, CachedValuable>;

// todo: add list/dict here
pub enum OwnedValuable<S, T> {
    SmallInt(i128),
    Float(f64),
    Bool(bool),
    None,
    Str(String, PhantomData<S>, PhantomData<T>),
    UnquotedStr(String, PhantomData<T>),
}

impl<S: StrFmt, T: TemplateFmt> Valuable for OwnedValuable<S, T> {
    fn as_value(&self) -> Value<'_> {
        match self {
            Self::SmallInt(int) => Value::I128(*int),
            Self::Float(float) => Value::F64(*float),
            Self::Bool(bool) => Value::Bool(*bool),
            Self::None => Value::Unit,
            Self::Str(str, _, _) => Value::String(str),
            Self::UnquotedStr(str, _) => Value::UnquotedString(str),
        }
    }

    fn visit(&self, visit: &mut dyn valuable::Visit) {
        self.as_value().visit(visit);
    }
}

// deferring any type checks until the value is required, so we don't waste time on filtered events
// should we use cast instead of cast_exact? not sure if anyone subclasses primitives
// maybe always use an UnquotedString, and use repr/str as the generic paramter instead?
impl<S: StrFmt, T: TemplateFmt> GetValue<OwnedValuable<S, T>, CachedValuable> for Bound<'_, PyAny> {
    fn value(&self) -> OwnedValuable<S, T> {
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
            T::make(tmpl)
        } else if let Ok(str) = self.cast_exact::<PyString>() {
            S::make(str.to_string())
        } else {
            OwnedValuable::<S, T>::UnquotedStr(python_format(self, self.repr()), PhantomData)
        }
    }
}

fn python_format(any: &Bound<'_, PyAny>, format_result: PyResult<Bound<'_, PyString>>) -> String {
    match format_result {
        Result::Ok(s) => return s.to_string_lossy().to_string(),
        Result::Err(err) => err.write_unraisable(any.py(), Some(any)),
    }

    any.get_type().name().map_or_else(
        |_| "<unprintable object>".to_owned(),
        |name| format!("<unprintable {name} object>"),
    )
}
