use std::{ffi::c_int, path::Path};

use pyo3::{
    ffi::{self, PyFrameObject, PyFrame_GetCode, PyFrame_GetLineNumber},
    types::{PyAnyMethods, PyDict, PyString},
    Bound, Py, Python,
};

use crate::inspect::Code;

unsafe extern "C" {
    // https://docs.python.org/3/c-api/frame.html#c.PyFrame_GetLasti
    // added in 3.11, this crate targets 3.14+
    pub fn PyFrame_GetLasti(f: *mut PyFrameObject) -> c_int;

    pub fn PyEval_GetFrameGlobals() -> *mut ffi::PyObject;
}

pub(crate) struct Frame<'a>(pub(super) Bound<'a, PyFrameObject>);

impl<'a> Frame<'a> {
    pub(crate) fn new(py: Python<'a>) -> Self {
        // SAFETY: Safe.
        let frame = unsafe { ffi::PyEval_GetFrame() };

        // SAFETY: PyEval_GetFrame return null or borrowed reference to PyFrameObject; from_borrowed_ptr checks for null
        let frame = unsafe { Py::from_borrowed_ptr(py, frame as *mut pyo3::ffi::PyObject) };
        Self(frame.into_bound(py))
    }

    pub(crate) fn line_number(&self) -> c_int {
        // SAFETY: self.frame is a valid frame
        unsafe { PyFrame_GetLineNumber(self.0.as_ptr() as *mut PyFrameObject) }
    }

    pub(crate) fn code(&'a self) -> Code<'a> {
        // SAFETY: self.frame is valid, result cannot be NULL: https://docs.python.org/3/c-api/frame.html#c.PyFrame_GetCode
        let code = unsafe { PyFrame_GetCode(self.0.as_ptr() as *mut PyFrameObject) };

        // SAFETY: code is indeed a PyCodeObject, and PyFrame_GetCode return a strong reference
        let code = unsafe { Py::from_owned_ptr(self.0.py(), code as *mut pyo3::ffi::PyObject) };
        Code::new(code.into_bound(self.0.py()))
    }
}
