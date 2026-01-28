use pyo3::{
    Bound,
    types::{PyBytes, PyCode, PyString},
};

use crate::any_ext::InfallibleAttr;

pub(crate) trait PyCodeMethodsExt<'py> {
    fn filename(&self) -> Bound<'py, PyString>;

    fn qualname(&self) -> Bound<'py, PyString>;

    fn name(&self) -> Bound<'py, PyString>;

    fn bytecode(&self) -> Bound<'py, PyBytes>;
}

impl<'py> PyCodeMethodsExt<'py> for Bound<'py, PyCode> {
    fn filename(&self) -> Bound<'py, PyString> {
        self.infallible_attr::<"co_filename", PyString>()
    }

    fn qualname(&self) -> Bound<'py, PyString> {
        self.infallible_attr::<"co_qualname", PyString>()
    }

    fn name(&self) -> Bound<'py, PyString> {
        self.infallible_attr::<"co_name", PyString>()
    }

    // todo: there's a PyCode_GetCode function, which should be a bit faster
    fn bytecode(&self) -> Bound<'py, PyBytes> {
        self.infallible_attr::<"co_code", PyBytes>()
    }
}
