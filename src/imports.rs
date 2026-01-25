use pyo3::{
    prelude::*,
    sync::PyOnceLock,
    types::{PyCFunction, PyType},
    PyTypeCheck,
};

pub(super) static TEMPLATE_TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();

pub(super) fn get_template_type<'py>(py: Python<'py>) -> &'py Bound<'py, PyType> {
    get_or_import(py, &TEMPLATE_TYPE, "string.templatelib", "Template")
}

pub(super) static INTERPOLATION_TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();

pub(super) fn get_interpolation_type<'py>(py: Python<'py>) -> &'py Bound<'py, PyType> {
    get_or_import(
        py,
        &INTERPOLATION_TYPE,
        "string.templatelib",
        "Interpolation",
    )
}

pub(super) static WRAPT_DECORATOR: PyOnceLock<Py<PyCFunction>> = PyOnceLock::new();

pub(super) fn get_wrapt_decorator<'py>(py: Python<'py>) -> &'py Bound<'py, PyCFunction> {
    get_or_import(py, &WRAPT_DECORATOR, "wrapt", "decorator")
}

fn get_or_import<'py, 'a, T: PyTypeCheck>(
    py: Python<'py>,
    lock: &'a PyOnceLock<Py<T>>,
    module: &'static str,
    item: &'static str,
) -> &'a Bound<'py, T> {
    lock.get_or_init(py, || import(module, item).unwrap())
        .bind(py)
}

fn import<T: PyTypeCheck>(module: &str, item: &str) -> PyResult<Py<T>> {
    let t = Python::attach(|py: Python<'_>| {
        let module = py.import(module)?;
        let attribute = module.getattr(item)?;
        Ok(attribute.cast_into::<T>()?.unbind())
    });
    t
}
