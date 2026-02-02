use std::ptr;

use pyo3::{ffi, prelude::*, types::PyString};

use crate::ext::ffi::FfiPtrExt;

pub macro infallible_attr {
    ($obj:expr, $attr:expr) => {
        ::pyo3::prelude::PyAnyMethods::getattr($obj.as_any(), ::pyo3::intern!($obj.py(), $attr))
            .unwrap()
            .cast_into()
            .unwrap()
    },
    ($obj:expr, $attr:expr, $py:expr) => {
        ::pyo3::prelude::PyAnyMethods::getattr($obj.bind($py).as_any(), ::pyo3::intern!($py, $attr))
            .unwrap()
            .cast_into()
            .unwrap()
    }
}

// copied from pyo3 src/py_result_ext.rs
pub trait PyResultExt<'py> {
    unsafe fn cast_into_unchecked<T>(self) -> PyResult<Bound<'py, T>>;
}

impl<'py> PyResultExt<'py> for PyResult<Bound<'py, PyAny>> {
    #[inline]
    unsafe fn cast_into_unchecked<T>(self) -> PyResult<Bound<'py, T>> {
        self.map(|instance| unsafe { instance.cast_into_unchecked() })
    }
}

pub trait PyAnyMethodsExt<'py> {
    fn ascii(&self) -> PyResult<Bound<'py, PyString>>;

    fn format(&self, format_spec: Option<Bound<'py, PyString>>) -> PyResult<Bound<'py, PyString>>;
}

impl<'py> PyAnyMethodsExt<'py> for Bound<'py, PyAny> {
    // same impl as fn str/repr(&self) from pyo3 src/types/any.rs, should be as safe as str method
    fn ascii(&self) -> PyResult<Bound<'py, PyString>> {
        unsafe {
            ffi::PyObject_ASCII(self.as_ptr())
                .assume_owned_or_err(self.py())
                .cast_into_unchecked()
        }
    }

    fn format(&self, format_spec: Option<Bound<'py, PyString>>) -> PyResult<Bound<'py, PyString>> {
        // https://docs.python.org/3/c-api/object.html#c.PyObject_Format:
        // "format_spec may be NULL. In this case the call is equivalent to format(obj).
        // Returns the formatted string on success, NULL on failure.""
        let format_spec = format_spec.map_or_else(ptr::null_mut, |str| str.as_ptr());

        unsafe {
            ffi::PyObject_Format(self.as_ptr(), format_spec)
                .assume_owned_or_err(self.py())
                .cast_into_unchecked()
        }
    }
}
