use pyo3::{
    ffi::{PyFunction_GetDefaults, PyFunction_GetKwDefaults},
    types::{PyDict, PyFunction, PyTuple},
    Bound,
};

// there's no PyFunctionMethods in pyo3, but ext a better name imo
//
// who thought allowing changing function defaults at runtime is a good idea?
pub(crate) trait PyFunctionMethodsExt<'py> {
    fn get_defaults(&self) -> Option<Bound<'py, PyTuple>>;
    fn get_kw_defaults(&self) -> Option<Bound<'py, PyDict>>;
}

impl<'py> PyFunctionMethodsExt<'py> for Bound<'py, PyFunction> {
    fn get_defaults(&self) -> Option<Bound<'py, PyTuple>> {
        // SAFETY: PyFunction_GetDefaults returns a borrowed tuple or null
        // https://docs.python.org/3/c-api/function.html#c.PyFunction_GetDefaults
        unsafe {
            { Bound::from_borrowed_ptr_or_opt(self.py(), PyFunction_GetDefaults(self.as_ptr())) }
                .map(|x| x.cast_into_unchecked())
        }
    }

    fn get_kw_defaults(&self) -> Option<Bound<'py, PyDict>> {
        // SAFETY: PyFunction_GetKwDefaults returns a borrowed dict or null
        // https://docs.python.org/3/c-api/function.html#c.PyFunction_GetKwDefaults
        unsafe {
            { Bound::from_borrowed_ptr_or_opt(self.py(), PyFunction_GetKwDefaults(self.as_ptr())) }
                .map(|x| x.cast_into_unchecked())
        }
    }
}
