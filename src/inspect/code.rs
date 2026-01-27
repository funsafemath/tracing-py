use pyo3::{
    Bound, Py,
    ffi::{self, PyBytesObject, PyCodeObject},
    types::{PyAnyMethods, PyString},
};

use crate::any_ext::InfallibleAttr;

unsafe extern "C" {
    // https://docs.python.org/3/c-api/code.html#c.PyCode_GetCode
    // added in 3.11, this crate targets 3.14+
    //
    // this may create a bytes object according to the docs, so maybe I should use another way to uniquely identify a code location
    //
    // returns a strong ref
    pub fn PyCode_GetCode(f: *mut PyCodeObject) -> *mut ffi::PyObject;
}

// todo: impl it as a transparent-repr struct with PyTypeInfo
pub(crate) struct Code<'a>(pub(super) Bound<'a, PyCodeObject>);

impl<'a> Code<'a> {
    pub(super) fn new(code: Bound<'a, PyCodeObject>) -> Self {
        Self(code)
    }

    pub(crate) fn filename(&self) -> Py<PyString> {
        self.0
            .as_any()
            .infallible_attr::<"co_filename", PyString>()
            .unbind()
    }

    pub(crate) fn target(&self) -> Py<PyString> {
        self.0
            .as_any()
            .infallible_attr::<"co_qualname", PyString>()
            .unbind()
    }

    pub(super) fn bytecode_addr(&'a self) -> usize {
        // SAFETY: self.code is a valid & bound PyCodeObject
        let addr = unsafe { PyCode_GetCode(self.0.as_ptr().cast::<PyCodeObject>()) };

        if addr.is_null() {
            // same as new(), maybe just ignore & return, not panic?
            panic!("frame bytecode is null");
        }

        // let's hope the interpreter is smart enough not to drop the bytecode after the request
        //
        // we can store the ref somewhere, but i trust the python interpreter (peak rust safety)
        //
        // i really should find another way to uniquely identify the code

        drop(unsafe { Py::<PyBytesObject>::from_owned_ptr(self.0.py(), addr) });

        addr as usize
    }
}
