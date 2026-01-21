use pyo3::{
    ffi::{PyBytesObject, PyCodeObject},
    types::{PyAnyMethods, PyString},
    Bound, Py,
};

unsafe extern "C" {
    // https://docs.python.org/3/c-api/code.html#c.PyCode_GetCode
    // added in 3.11, this crate targets 3.14+
    //
    // this may create a bytes object according to the docs, so maybe I should use another way to uniquely identify a code location
    //
    // returns a strong ref
    pub fn PyCode_GetCode(f: *mut PyCodeObject) -> *mut pyo3::ffi::PyObject;
}

pub(crate) struct Code<'a>(Bound<'a, PyCodeObject>);

impl<'a> Code<'a> {
    pub(super) fn new(code: Bound<'a, PyCodeObject>) -> Self {
        Self(code)
    }

    pub(crate) fn filename(&self) -> Py<PyString> {
        // Neither PyO3 not Python C API provide a function to get co_filename from a struct directly

        // PyObject_GetAttrString is probably faster, but it's unsafe, and I didn't find a function that gets an attr by string
        // idk maybe it already uses it
        let name = self
            .0
            .as_any()
            .getattr("co_filename")
            .expect("code object must have \"co_filename\" property");

        name.extract()
            .expect("\"co_filename\" of a code object must be a string")
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
