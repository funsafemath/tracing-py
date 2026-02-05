use std::path::Path;

use pyo3::{
    ffi::{self, PyEval_GetGlobals},
    prelude::*,
    types::{PyCode, PyFrame, PyString},
};

use crate::ext::{code::PyCodeMethodsExt, frame::PyFrameMethodsExt};

#[non_exhaustive]
pub struct Inspector<'a, 'py> {
    pub frame: &'a Bound<'py, PyFrame>,
    pub code: Bound<'py, PyCode>,
    pub py: Python<'py>,
}

impl<'a, 'py> Inspector<'a, 'py> {
    pub fn new(frame: &'a Bound<'py, PyFrame>) -> Self {
        Self {
            code: frame.code(),
            py: frame.py(),
            frame,
        }
    }

    pub fn ix_address(&self) -> usize {
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
    pub fn module(&self) -> String {
        // SAFETY: safe to call, null check is at Py::from_owned_ptr
        let globals = unsafe { PyEval_GetGlobals() };

        // SAFETY: safe, PyEval_GetFrameGlobals returns an borrowed ref
        let globals = unsafe { Bound::from_borrowed_ptr(self.py, globals) };
        let mod_name = PyAnyMethods::get_item(&globals, "__name__")
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
