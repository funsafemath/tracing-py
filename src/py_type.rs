use pyo3::{PyTypeInfo, prelude::*, types::PyType};

use crate::imports::mk_import;

pub(crate) macro mk_imported_type($type:ident, $module:expr, $item:expr) {
    #[repr(transparent)]
    pub(crate) struct $type(PyAny);

    mk_import!(get_type, $module, $item, PyType);

    // SAFETY: type_object_raw infallibly produces a valid pointer to the type object
    unsafe impl PyTypeInfo for $type {
        const NAME: &'static str = $item;

        const MODULE: Option<&'static str> = Some($module);

        fn type_object_raw(py: Python<'_>) -> *mut pyo3::ffi::PyTypeObject {
            get_type(py).as_type_ptr()
        }
    }
}
