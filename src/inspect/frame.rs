use std::ffi::c_int;

use pyo3::{
    Bound, Python,
    ffi::{self, PyFrame_GetCode, PyFrame_GetLasti, PyFrame_GetLineNumber},
    types::{PyCode, PyFrame},
};

pub(crate) trait UnboundPyFrameMethodsExt {
    fn from_thread_state(py: Python<'_>) -> Option<Bound<'_, Self>>
    where
        Self: Sized;
}

impl UnboundPyFrameMethodsExt for PyFrame {
    fn from_thread_state(py: Python<'_>) -> Option<Bound<'_, Self>> {
        // returns NULL/Borrowed: https://docs.python.org/3/c-api/reflection.html#c.PyEval_GetFrame
        unsafe {
            Bound::from_borrowed_ptr_or_opt(py, ffi::PyEval_GetFrame() as *mut ffi::PyObject)
                .map(|x| x.cast_into_unchecked())
        }
    }
}

// frame should be !Send/!Sync actually, but that requires modifying PyO3 source
// https://docs.python.org/3/howto/free-threading-python.html#frame-objects
pub(crate) trait PyFrameMethodsExt<'py> {
    fn line_number(&self) -> c_int;

    fn code(&self) -> Bound<'py, PyCode>;

    fn last_ix_index(&self) -> Option<usize>;
}

impl<'py> PyFrameMethodsExt<'py> for Bound<'py, PyFrame> {
    fn line_number(&self) -> c_int {
        {
            // SAFETY: self.frame is a valid frame
            unsafe { PyFrame_GetLineNumber(self.as_ptr() as *mut ffi::PyFrameObject) }
        }
    }

    fn code(&self) -> Bound<'py, PyCode> {
        let code = unsafe { PyFrame_GetCode(self.as_ptr() as *mut ffi::PyFrameObject) };

        // PyFrame_GetCode returns a strong reference, https://docs.python.org/3/c-api/frame.html#c.PyFrame_GetCode
        unsafe {
            Bound::from_owned_ptr(self.py(), code as *mut ffi::PyObject).cast_into_unchecked()
        }
    }

    fn last_ix_index(&self) -> Option<usize> {
        let code = unsafe { PyFrame_GetLasti(self.as_ptr() as *mut ffi::PyFrameObject) };

        match code {
            -1 => None,
            // should not panic, as it's an index, which already means it's less or equal than usize::MAX
            // (and it could panic only on 16-bit systems, can these even run python?)
            0.. => Some(usize::try_from(code).unwrap()),
            _ => unreachable!(),
        }
    }
}
