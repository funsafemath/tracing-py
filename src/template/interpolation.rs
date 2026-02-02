use pyo3::{prelude::*, types::PyString};

use crate::{
    ext::any::{PyAnyMethodsExt, infallible_attr},
    py_type::mk_imported_type,
};

mk_imported_type!(PyInterpolation, "string.templatelib", "Interpolation");

pub(crate) enum Conversion {
    Str,
    Repr,
    Ascii,
}

impl Conversion {
    fn convert<'py, T>(&self, object: Bound<'py, T>) -> PyResult<Bound<'py, PyString>> {
        match self {
            Conversion::Str => object.as_any().str(),
            Conversion::Repr => object.as_any().repr(),
            Conversion::Ascii => object.as_any().ascii(),
        }
    }
}

pub(crate) trait PyInterpolationMethods<'py> {
    fn value(&self) -> Bound<'py, PyAny>;
    fn expression(&self) -> Bound<'py, PyString>;
    fn conversion(&self) -> Conversion;
    fn format_spec(&self) -> Bound<'py, PyString>;
}

impl<'py> PyInterpolationMethods<'py> for Bound<'py, PyInterpolation> {
    fn value(&self) -> Bound<'py, PyAny> {
        infallible_attr!(self, "value")
    }

    fn expression(&self) -> Bound<'py, PyString> {
        infallible_attr!(self, "expression")
    }

    fn conversion(&self) -> Conversion {
        let conversion_string: Bound<'_, PyString> = infallible_attr!(self, "conversion");
        match conversion_string.to_string().as_str() {
            "s" => Conversion::Str,
            "r" => Conversion::Repr,
            "a" => Conversion::Ascii,
            // unreachable as python does not allow to use anything except s, r and a
            _ => unreachable!("invalid conversion string: {conversion_string}"),
        }
    }

    fn format_spec(&self) -> Bound<'py, PyString> {
        infallible_attr!(self, "format_spec")
    }
}
