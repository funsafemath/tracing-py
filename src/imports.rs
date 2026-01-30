use pyo3::{
    PyTypeCheck,
    prelude::*,
    sync::PyOnceLock,
    types::{PyCFunction, PyFunction, PyType},
};

macro_rules! mk_import {
    ($fn_name:ident, $module:expr, $item:expr, $type:ty) => {
        pub(super) fn $fn_name<'py>(py: Python<'py>) -> &'py Bound<'py, $type> {
            static LOCK: PyOnceLock<Py<$type>> = PyOnceLock::new();

            get_or_import(py, &LOCK, $module, $item)
        }
    };
}

mk_import!(get_generator_type, "types", "GeneratorType", PyType);
mk_import!(get_coroutine_type, "types", "CoroutineType", PyType);

mk_import!(get_template_type, "string.templatelib", "Template", PyType);
mk_import!(
    get_interpolation_type,
    "string.templatelib",
    "Interpolation",
    PyType
);

mk_import!(get_inspect_signature, "inspect", "signature", PyFunction);
mk_import!(get_inspect_signature_type, "inspect", "Signature", PyType);
mk_import!(get_inspect_parameter_type, "inspect", "Parameter", PyType);

mk_import!(get_atexit_register, "atexit", "register", PyCFunction);

fn get_or_import<'py, 'a, T: PyTypeCheck>(
    py: Python<'py>,
    lock: &'a PyOnceLock<Py<T>>,
    module: &'static str,
    item: &'static str,
) -> &'a Bound<'py, T> {
    lock.get_or_init(py, || import(module, item).unwrap())
        .bind(py)
}

// todo: use this to make templates optional, so library works on python < 3.14
fn get_or_import_maybe<'py, T: PyTypeCheck>(
    py: Python<'py>,
    lock: &'static PyOnceLock<Option<Py<T>>>,
    module: &'static str,
    item: &'static str,
) -> Option<&'static Bound<'py, T>> {
    lock.get_or_init(py, || import(module, item).ok())
        .as_ref()
        .map(|x| x.bind(py))
}

fn import<T: PyTypeCheck>(module: &str, item: &str) -> PyResult<Py<T>> {
    Python::attach(|py: Python<'_>| {
        let module = py.import(module)?;
        let attribute = module.getattr(item)?;
        Ok(attribute.cast_into::<T>()?.unbind())
    })
}
