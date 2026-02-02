use pyo3::{prelude::*, types::PyTuple};

use crate::{ext::any::infallible_attr, py_type::mk_imported_type};

// todo: use conditional compilation
mk_imported_type!(PyTemplate, "string.templatelib", "Template");

pub(crate) trait PyTemplateMethods<'py> {
    fn strings(&self) -> Bound<'py, PyTuple>;
    fn interpolations(&self) -> Bound<'py, PyTuple>;
}

impl<'py> PyTemplateMethods<'py> for Bound<'py, PyTemplate> {
    fn strings(&self) -> Bound<'py, PyTuple> {
        infallible_attr!(self, "strings")
    }

    fn interpolations(&self) -> Bound<'py, PyTuple> {
        infallible_attr!(self, "interpolations")
    }
}
