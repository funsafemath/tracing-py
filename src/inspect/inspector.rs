use std::{ffi::c_int, path::Path};

use pyo3::{
    ffi::{self, PyFrameObject},
    prelude::*,
    types::PyString,
};

use crate::inspect::{code::Code, Frame};

unsafe extern "C" {
    // https://docs.python.org/3/c-api/frame.html#c.PyFrame_GetLasti
    // added in 3.11, this crate targets 3.14+
    pub fn PyFrame_GetLasti(f: *mut PyFrameObject) -> c_int;

    pub fn PyEval_GetFrameGlobals() -> *mut ffi::PyObject;
}

#[non_exhaustive]
pub(crate) struct Inspector<'a> {
    pub(crate) frame: &'a Frame<'a>,
    pub(crate) code: Code<'a>,
    pub(crate) py: Python<'a>,
}

impl<'a> Inspector<'a> {
    pub(crate) fn new(frame: &'a Frame) -> Self {
        Self {
            frame,
            code: frame.code(),
            py: frame.0.py(),
        }
    }

    pub(crate) fn ix_address(&self) -> usize {
        // SAFETY: self.frame is a valid frame
        let last_instruction_offset = usize::try_from(unsafe {
            PyFrame_GetLasti(self.frame.0.as_ptr() as *mut PyFrameObject)
        })
        .expect("16-bit computers are not supported, sorry");
        self.code.bytecode_addr() + last_instruction_offset
    }

    // ugly, but for some reason other attributes/functions for introspection didn't work
    //
    // it fails if someone changes __name__, but who would do this?
    //
    // maybe i'll fix it later, but anyway it's evaluated only a single time for each callsite
    pub(crate) fn module(&self) -> String {
        let globals = unsafe { PyEval_GetFrameGlobals() };
        let globals = unsafe { Py::from_owned_ptr(self.py, globals) };
        let globals: &Bound<'_, PyAny> = globals.bind(self.py);
        let mod_name = PyAnyMethods::get_item(globals, "__name__")
            .expect("__name__ global variable must exist");

        let s = if mod_name.is_none() {
            0
        } else {
            let s: Py<PyString> = mod_name.extract().expect("__name__ was modified");
            s.to_string_lossy(self.py)
                .chars()
                .filter(|x| *x == '.')
                .count()
        } + 1;

        let file = self.code.filename();
        let file = file.to_string_lossy(self.py).into_owned();
        let path = Path::new(&file).to_owned();

        // todo: this is a horrible idea, it should be modules[modules[__name__].__package__]...-based lookup at least
        // (i haven't slept for a long time, so i'll do it later)
        path.components()
            .rev()
            .take(s)
            .map(|x| x.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("/")
    }
}
