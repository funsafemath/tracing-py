use pyo3::{
    ffi::{PyBytesObject, PyCodeObject, PyObject_GetAttrString},
    types::PyString,
    Py, Python,
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

pub(crate) struct Code<'a> {
    code: *mut PyCodeObject,

    py: Python<'a>,
}

impl<'a> Code<'a> {
    // SAFETY: code must be a valid PyCodeObject and must live at least as long as py
    pub(super) unsafe fn new(code: *mut PyCodeObject, py: Python<'a>) -> Self {
        Self { code, py }
    }

    pub(crate) fn filename(&self) -> Py<PyString> {
        // Neither PyO3 not Python C API provide a function to get co_filename

        // SAFETY: self.code is a valid PyObject, c"co_filename".as_ptr() is a valid C string
        let name = unsafe { PyObject_GetAttrString(self.code as _, c"co_filename".as_ptr()) };

        // SAFETY: PyObject_GetAttrString returns an owned pointer or null, in the second case the function panics.
        unsafe { Py::from_owned_ptr(self.py, name) }
    }

    pub(super) fn bytecode_addr(&'a self) -> usize {
        // SAFETY: self.code is a valid PyCodeObject
        let addr = unsafe { PyCode_GetCode(self.code) };

        if addr.is_null() {
            // same as new(), maybe just ignore & return, not panic?
            panic!("frame bytecode is null");
        }

        // let's hope the interpreter is smart enough not to drop the bytecode after the request
        //
        // we can store the ref somewhere, but i trust the python interpreter (peak rust safety)
        //
        // i really should find another way to uniquely identify the code
        drop(unsafe { Py::<PyBytesObject>::from_owned_ptr(self.py, addr) });

        addr as usize
    }
}
