use pyo3::{prelude::*, types::PyMappingProxy};

use crate::{infallible_attr, py_type::mk_imported_type};

mk_imported_type!(PySignature, "inspect", "Signature");

pub(crate) trait PySignatureMethods<'py> {
    fn parameters(&self) -> Bound<'py, PyMappingProxy>;
}

impl<'py> PySignatureMethods<'py> for Bound<'py, PySignature> {
    fn parameters(&self) -> Bound<'py, PyMappingProxy> {
        infallible_attr!(self, "parameters")
    }
}
