use std::ffi::c_int;

use pyo3::{
    ffi::{PyFrameObject, PyFrame_GetCode, PyFrame_GetLineNumber},
    Py, Python,
};

use crate::inspect::Code;

unsafe extern "C" {
    // https://docs.python.org/3/c-api/frame.html#c.PyFrame_GetLasti
    // added in 3.11, this crate targets 3.14+
    pub fn PyFrame_GetLasti(f: *mut PyFrameObject) -> c_int;
}

pub(crate) struct Frame<'a> {
    frame: *mut PyFrameObject,

    py: Python<'a>,
}

impl<'a> Frame<'a> {
    pub(crate) fn new(py: Python<'a>) -> Self {
        // SAFETY: uhh... safe? the lifetime ensures that object can't be used anywhere
        // after yielding the control back to interpreter (PyEval_GetFrame returns a borrowed reference)
        let frame = unsafe { pyo3::ffi::PyEval_GetFrame() };
        if frame.is_null() {
            // maybe just ignore & return, not panic?
            panic!("interpreter frame is null");
        }

        Self { frame, py }
    }

    pub(crate) fn line_number(&self) -> c_int {
        // SAFETY: self.frame is a valid frame
        unsafe { PyFrame_GetLineNumber(self.frame) }
    }

    pub(crate) fn ix_address(&self) -> usize {
        let code = self.code();
        // SAFETY: self.frame is a valid frame
        let last_instruction_offset = usize::try_from(unsafe { PyFrame_GetLasti(self.frame) })
            .expect("16-bit computers are not supported, sorry");
        code.bytecode_addr() + last_instruction_offset
    }

    pub(crate) fn code(&'a self) -> Code<'a> {
        // SAFETY: self.frame is valid, result cannot be NULL: https://docs.python.org/3/c-api/frame.html#c.PyFrame_GetCode
        let code = unsafe { PyFrame_GetCode(self.frame) };

        // SAFETY: code is a valid object, and it obviously lives at least as long as the frame, which lives as long as
        // we don't return to the callsite, and that's exactly the lifetime of py
        let code = unsafe { Py::from_owned_ptr(self.py, code as *mut pyo3::ffi::PyObject) };
        Code::new(code.into_bound(self.py))
    }
}
