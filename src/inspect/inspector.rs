use std::path::Path;

use pyo3::{
    ffi::{self},
    prelude::*,
    types::{PyCode, PyFrame, PyString},
};

use crate::inspect::{code::PyCodeMethodsExt, frame::PyFrameMethodsExt};

unsafe extern "C" {
    pub fn PyEval_GetFrameGlobals() -> *mut ffi::PyObject;
}

#[non_exhaustive]
pub(crate) struct Inspector<'a, 'py> {
    pub(crate) frame: &'a Bound<'py, PyFrame>,
    pub(crate) code: Bound<'py, PyCode>,
    pub(crate) py: Python<'py>,
}

impl<'a, 'py> Inspector<'a, 'py> {
    pub(crate) fn new(frame: &'a Bound<'py, PyFrame>) -> Self {
        Self {
            code: frame.code(),
            py: frame.py(),
            frame,
        }
    }

    pub(crate) fn ix_address(&self) -> usize {
        // SAFETY: self.frame is a valid frame
        let last_instruction_offset = self
            .frame
            .last_ix_index()
            .expect("frame has no instruction index, is the function called from python context?");
        self.code.bytecode().as_ptr() as usize + last_instruction_offset
    }

    // ugly, but for some reason other attributes/functions for introspection didn't work
    //
    // it fails if someone changes __name__, but who would do this?
    //
    // maybe i'll fix it later, but anyway it's evaluated only a single time for each callsite
    pub(crate) fn module(&self) -> String {
        // SAFETY: safe to call, null check is at Py::from_owned_ptr
        let globals = unsafe { PyEval_GetFrameGlobals() };

        // SAFETY: safe, PyEval_GetFrameGlobals returns an owned ref
        let globals = unsafe { Py::from_owned_ptr(self.py, globals) };
        let globals: &Bound<'_, PyAny> = globals.bind(self.py);
        let mod_name = PyAnyMethods::get_item(globals, "__name__")
            .expect("__name__ global variable must exist");

        let path_length = if mod_name.is_none() {
            0
        } else {
            let s: Py<PyString> = mod_name
                .extract()
                .expect("__name__ type must be str or None");
            s.to_string_lossy(self.py)
                .chars()
                .filter(|x| *x == '.')
                .count()
        } + 1;

        let file = self.code.filename();
        let file = file.to_string_lossy().into_owned();
        let path = Path::new(&file).to_owned();

        // todo: this is a horrible idea, it should be modules[modules[__name__].__package__]...-based lookup at least
        // (i haven't slept for a long time, so i'll do it later)
        path.components()
            .rev()
            .take(path_length)
            .map(|x| x.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("/")
    }
}
