use pyo3::{prelude::*, sync::PyOnceLock, types::PyTuple};

use crate::{ext::any::infallible_attr, py_type::mk_imported_type};

mk_imported_type!(PyTemplate, "string.templatelib", "Template");

impl PyTemplate {
    // todo: conditional compilation would be much better
    pub fn is_supported(py: Python<'_>) -> bool {
        static TEMPALTES_SUPPORTED: PyOnceLock<bool> = PyOnceLock::new();
        *TEMPALTES_SUPPORTED.get_or_init(py, || py.version_info() >= (3, 14))
    }
}

pub trait PyTemplateMethods<'py> {
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
