use pyo3::{PyTypeInfo, prelude::*, types::PyMappingProxy};

use crate::{imports::get_inspect_signature_type, infallible_attr};

#[repr(transparent)]
pub(crate) struct PySignature(PyAny);

// SAFETY: type_object_raw infallibly produces a valid pointer to the type object
unsafe impl PyTypeInfo for PySignature {
    const NAME: &'static str = "Signature";

    const MODULE: Option<&'static str> = Some("inspect");

    fn type_object_raw(py: Python<'_>) -> *mut pyo3::ffi::PyTypeObject {
        get_inspect_signature_type(py).as_type_ptr()
    }
}

pub(crate) trait PySignatureMethods<'py> {
    fn parameters(&self) -> Bound<'py, PyMappingProxy>;
}

impl<'py> PySignatureMethods<'py> for Bound<'py, PySignature> {
    fn parameters(&self) -> Bound<'py, PyMappingProxy> {
        infallible_attr!(self, "parameters")
    }
}
