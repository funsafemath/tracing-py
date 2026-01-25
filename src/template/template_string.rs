use pyo3::{prelude::*, types::PyTuple, PyTypeInfo};

use crate::{any_ext::InfallibleAttr, imports::get_template_type};

#[repr(transparent)]
pub(crate) struct PyTemplate(PyAny);

// SAFETY: type_object_raw infallibly produces a valid pointer to the type object
// todo: use conditional compilation
unsafe impl PyTypeInfo for PyTemplate {
    const NAME: &'static str = "Template";

    const MODULE: Option<&'static str> = Some("string.templatelib");

    fn type_object_raw(py: Python<'_>) -> *mut pyo3::ffi::PyTypeObject {
        get_template_type(py).as_type_ptr()
    }
}

pub(crate) trait PyTemplateMethods<'py> {
    fn strings(&self) -> Bound<'py, PyTuple>;
    fn interpolations(&self) -> Bound<'py, PyTuple>;
}

impl<'py> PyTemplateMethods<'py> for Bound<'py, PyTemplate> {
    fn strings(&self) -> Bound<'py, PyTuple> {
        self.infallible_attr::<"strings", PyTuple>()
    }

    fn interpolations(&self) -> Bound<'py, PyTuple> {
        self.infallible_attr::<"interpolations", PyTuple>()
    }
}
