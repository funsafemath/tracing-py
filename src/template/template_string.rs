use pyo3::{prelude::*, types::PyTuple};

use crate::{infallible_attr, py_type::mk_imported_type};

// pub fn is_available() {
//     get_template_type(py)
// }

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
