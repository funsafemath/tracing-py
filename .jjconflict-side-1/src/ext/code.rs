use pyo3::{
    Bound,
    types::{PyBytes, PyCode, PyString},
};

use crate::ext::any::infallible_attr;

pub trait PyCodeMethodsExt<'py> {
    fn filename(&self) -> Bound<'py, PyString>;

    fn qualname(&self) -> Bound<'py, PyString>;

    fn name(&self) -> Bound<'py, PyString>;

    fn bytecode(&self) -> Bound<'py, PyBytes>;
}

impl<'py> PyCodeMethodsExt<'py> for Bound<'py, PyCode> {
    fn filename(&self) -> Bound<'py, PyString> {
        infallible_attr!(self, "co_filename")
    }

    fn qualname(&self) -> Bound<'py, PyString> {
        infallible_attr!(self, "co_qualname")
    }

    fn name(&self) -> Bound<'py, PyString> {
        infallible_attr!(self, "co_name")
    }

    // todo: there's a PyCode_GetCode function, which should be a bit faster
    fn bytecode(&self) -> Bound<'py, PyBytes> {
        infallible_attr!(self, "co_code")
    }
}
