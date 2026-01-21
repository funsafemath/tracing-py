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

pub(crate) struct Frame<'a>(Bound<'a, PyFrameObject>);

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

    pub(crate) fn ix_address(&self) -> usize {
        let code = self.code();
        // SAFETY: self.frame is a valid frame
        let last_instruction_offset =
            usize::try_from(unsafe { PyFrame_GetLasti(self.0.as_ptr() as *mut PyFrameObject) })
                .expect("16-bit computers are not supported, sorry");
        code.bytecode_addr() + last_instruction_offset
    }

    // ugly, but for some reason other attributes/functions for introspection didn't work
    //
    // it fails if someone changes __name__, but who would do this?
    //
    // maybe i'll fix it later, but anyway it's evaluated only a single time for each callsite
    pub(crate) fn module(&self) -> String {
        let globals = unsafe { PyEval_GetFrameGlobals() };
        let globals = unsafe { Py::from_owned_ptr(self.0.py(), globals) };
        let globals: &Bound<'_, PyDict> = globals.bind(self.0.py());
        let mod_name = globals
            .get_item("__name__")
            .expect("__name__ global variable must exist");

        let s = if mod_name.is_none() {
            0
        } else {
            let s: Py<PyString> = mod_name.extract().expect("__name__ was modified");
            s.to_string_lossy(self.0.py())
                .chars()
                .filter(|x| *x == '.')
                .count()
        } + 1;

        let file = self.code().filename();
        let file = file.to_string_lossy(self.0.py()).into_owned();
        let path = Path::new(&file).to_owned();

        // todo: this is a horrible idea, please change to modules[modules[__name__].__package__]...-based lookup at least
        path.components()
            .rev()
            .take(s)
            .map(|x| x.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/")
    }

    pub(crate) fn code(&'a self) -> Code<'a> {
        // SAFETY: self.frame is valid, result cannot be NULL: https://docs.python.org/3/c-api/frame.html#c.PyFrame_GetCode
        let code = unsafe { PyFrame_GetCode(self.0.as_ptr() as *mut PyFrameObject) };

        // SAFETY: code is indeed a PyCodeObject, and PyFrame_GetCode return a strong reference
        let code = unsafe { Py::from_owned_ptr(self.0.py(), code as *mut pyo3::ffi::PyObject) };
        Code::new(code.into_bound(self.0.py()))
    }
}
