use pyo3::{PyTypeInfo, prelude::*, types::PyString};

use crate::{ext::any::PyAnyMethodsExt, imports::get_interpolation_type, infallible_attr};

#[repr(transparent)]
pub(crate) struct PyInterpolation(PyAny);

// SAFETY: type_object_raw infallibly produces a valid pointer to the type object
// todo: use conditional compilation
unsafe impl PyTypeInfo for PyInterpolation {
    const NAME: &'static str = "Interpolation";

    const MODULE: Option<&'static str> = Some("string.templatelib");

    fn type_object_raw(py: Python<'_>) -> *mut pyo3::ffi::PyTypeObject {
        get_interpolation_type(py).as_type_ptr()
    }
}

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
