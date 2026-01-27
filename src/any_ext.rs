use std::ptr;

use pyo3::{ffi, prelude::*, types::PyString, PyTypeCheck};

// copied from pyo3 src/py_result_ext.rs
pub(crate) trait PyResultExt<'py> {
    unsafe fn cast_into_unchecked<T>(self) -> PyResult<Bound<'py, T>>;
}

impl<'py> PyResultExt<'py> for PyResult<Bound<'py, PyAny>> {
    #[inline]
    unsafe fn cast_into_unchecked<T>(self) -> PyResult<Bound<'py, T>> {
        self.map(|instance| unsafe { instance.cast_into_unchecked() })
    }
}

// copied from pyo3 src/ffi_ptr_ext.rs
pub(crate) trait FfiPtrExt {
    /// Assumes this pointer carries a Python reference which needs to be decref'd.
    ///
    /// If the pointer is NULL, this function will fetch an error.
    unsafe fn assume_owned_or_err(self, py: Python<'_>) -> PyResult<Bound<'_, PyAny>>;
}

impl FfiPtrExt for *mut ffi::PyObject {
    #[inline]
    unsafe fn assume_owned_or_err(self, py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        unsafe { Bound::from_owned_ptr_or_err(py, self) }
    }
}

pub(super) trait InfallibleAttr<'py> {
    fn infallible_attr<const ATTR: &'static str, R: PyTypeCheck>(&self) -> Bound<'py, R>;
}

impl<'py, T> InfallibleAttr<'py> for Bound<'py, T> {
    // todo: use pyo3 intern! macro
    fn infallible_attr<const ATTR: &'static str, R: PyTypeCheck>(&self) -> Bound<'py, R> {
        self.as_any().getattr(ATTR).unwrap().cast_into().unwrap()
    }
}

pub(super) trait PyAnyMethodsExt<'py> {
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
        let format_spec = match format_spec {
            Some(str) => str.as_ptr(),
            None => ptr::null_mut(),
        };

        unsafe {
            ffi::PyObject_Format(self.as_ptr(), format_spec)
                .assume_owned_or_err(self.py())
                .cast_into_unchecked()
        }
    }
}
