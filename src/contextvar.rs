use std::{ffi::CStr, ptr};

use pyo3::{
    Bound, PyAny, PyResult, PyTypeInfo, Python,
    ffi::{self, PyContextVar_Get, PyContextVar_New, PyContextVar_Reset, PyContextVar_Set},
    types::PyList,
};

use crate::{
    any_ext::PyResultExt,
    ffi_ext::{FfiPtrExt, error_on_minusone},
};

#[repr(transparent)]
pub(crate) struct PyContextVar(PyAny);

unsafe impl PyTypeInfo for PyContextVar {
    const NAME: &'static str = "ContextVar";

    // or should I use "_contextvars"?
    const MODULE: Option<&'static str> = Some("contextvars");

    fn type_object_raw(py: Python<'_>) -> *mut pyo3::ffi::PyTypeObject {
        &raw mut ffi::PyContext_Type
    }
}

impl PyContextVar {
    fn new<'py>(py: Python<'py>, name: &CStr) -> PyResult<Bound<'py, Self>> {
        unsafe {
            PyContextVar_New(name.as_ptr(), ptr::null_mut())
                .assume_owned_or_err(py)
                .cast_into_unchecked()
        }
    }
}

pub(crate) trait PyContextVarMethods<'py> {
    fn get(&self) -> PyResult<Option<Bound<'py, PyAny>>>;
    fn set<T>(&self, value: Bound<'py, T>) -> PyResult<Bound<'py, PyContextToken>>;
    fn reset(&self, token: Bound<'py, PyContextToken>) -> PyResult<()>;
}

impl<'py> PyContextVarMethods<'py> for Bound<'py, PyContextVar> {
    fn get(&self) -> PyResult<Option<Bound<'py, PyAny>>> {
        let mut value = ptr::null_mut();
        error_on_minusone(self.py(), unsafe {
            PyContextVar_Get(self.as_ptr(), ptr::null_mut(), &mut value)
        })?;

        Ok(unsafe { value.assume_owned_or_opt(self.py()) })
    }

    fn set<T>(&self, value: Bound<'py, T>) -> PyResult<Bound<'py, PyContextToken>> {
        unsafe {
            { PyContextVar_Set(self.as_ptr(), value.as_ptr()) }
                .assume_owned_or_err(self.py())
                .cast_into_unchecked()
        }
    }

    fn reset(&self, token: Bound<'py, PyContextToken>) -> PyResult<()> {
        error_on_minusone(self.py(), unsafe {
            PyContextVar_Reset(self.as_ptr(), token.as_ptr())
        })
    }
}

#[repr(transparent)]
pub(crate) struct PyContextToken(PyAny);

unsafe impl PyTypeInfo for PyContextToken {
    const NAME: &'static str = "Token";

    const MODULE: Option<&'static str> = Some("contextvars");

    fn type_object_raw(py: Python<'_>) -> *mut pyo3::ffi::PyTypeObject {
        &raw mut ffi::PyContextToken_Type
    }
}
