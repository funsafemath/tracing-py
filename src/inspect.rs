use std::ffi::c_int;

use pyo3::{
    ffi::{
        PyBytesObject, PyCodeObject, PyFrameObject, PyFrame_GetCode, PyFrame_GetLineNumber,
        PyObject_GetAttrString,
    },
    types::PyString,
    Py, Python,
};

unsafe extern "C" {
    // https://docs.python.org/3/c-api/frame.html#c.PyFrame_GetLasti
    // added in 3.11, this crate targets 3.14+
    pub fn PyFrame_GetLasti(f: *mut PyFrameObject) -> c_int;

    // https://docs.python.org/3/c-api/code.html#c.PyCode_GetCode
    // same, added in 3.11
    //
    // this may create a bytes object according to the docs, so maybe I should use another way to uniquely identify a code location
    // returns a strong ref
    pub fn PyCode_GetCode(f: *mut PyCodeObject) -> *mut pyo3::ffi::PyObject;
}

pub(super) struct Frame<'a> {
    frame: *mut PyFrameObject,

    py: Python<'a>,
}

impl<'a> Frame<'a> {
    pub(super) fn new(py: Python<'a>) -> Self {
        // SAFETY: uhh... safe? the lifetime ensures that object can't be used anywhere
        // after yielding the control back to interpreter (PyEval_GetFrame returns a borrowed reference)
        let frame = unsafe { pyo3::ffi::PyEval_GetFrame() };
        if frame.is_null() {
            // maybe just ignore & return, not panic?
            panic!("interpreter frame is null");
        }

        Self { frame, py }
    }

    pub(super) fn line_number(&self) -> c_int {
        // SAFETY: self.frame is a valid frame
        unsafe { PyFrame_GetLineNumber(self.frame) }
    }

    pub(super) fn ix_address(&self) -> usize {
        let code = self.code();
        // SAFETY: self.frame is a valid frame
        let last_instruction_offset = usize::try_from(unsafe { PyFrame_GetLasti(self.frame) })
            .expect("16-bit computers are not supported, sorry");
        code.bytecode_addr() + last_instruction_offset
    }

    pub(super) fn code(&'a self) -> Code<'a> {
        Code {
            code: unsafe { PyFrame_GetCode(self.frame) },
            py: self.py,
        }
    }
}

pub(super) struct Code<'a> {
    code: *mut PyCodeObject,

    py: Python<'a>,
}

impl<'a> Code<'a> {
    pub(super) fn filename(&self) -> Py<PyString> {
        // Neither PyO3 not Python C API provide a function to get co_filename

        // SAFETY: self.code is a valid PyObject
        let a = unsafe { PyObject_GetAttrString(self.code as _, c"co_filename".as_ptr()) };

        // SAFETY: PyObject_GetAttrString returns an owned pointer or null, in the second case the function panics.
        unsafe { Py::from_owned_ptr(self.py, a) }
    }

    fn bytecode_addr(&'a self) -> usize {
        // SAFETY: self.code is a valid PyObject
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
