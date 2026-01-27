use pyo3::{
    prelude::*,
    sync::PyOnceLock,
    types::{PyFunction, PyType},
    PyTypeCheck,
};

pub(super) fn get_template_type<'py>(py: Python<'py>) -> &'py Bound<'py, PyType> {
    static TEMPLATE_TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();

    get_or_import(py, &TEMPLATE_TYPE, "string.templatelib", "Template")
}

pub(super) fn get_interpolation_type<'py>(py: Python<'py>) -> &'py Bound<'py, PyType> {
    static INTERPOLATION_TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();

    get_or_import(
        py,
        &INTERPOLATION_TYPE,
        "string.templatelib",
        "Interpolation",
    )
}

pub(super) fn get_wrapt_decorator<'py>(py: Python<'py>) -> &'py Bound<'py, PyFunction> {
    // todo: reconsider using wrapt, as it's quite slow
    static WRAPT_DECORATOR: PyOnceLock<Py<PyFunction>> = PyOnceLock::new();

    get_or_import(py, &WRAPT_DECORATOR, "wrapt", "decorator")
}

pub(super) fn get_inspect_signature<'py>(py: Python<'py>) -> &'py Bound<'py, PyFunction> {
    static INSPECT_SIGNATURE: PyOnceLock<Py<PyFunction>> = PyOnceLock::new();

    get_or_import(py, &INSPECT_SIGNATURE, "inspect", "signature")
}

pub(super) fn get_inspect_signature_type<'py>(py: Python<'py>) -> &'py Bound<'py, PyType> {
    static INSPECT_SIGNATURE_TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();

    get_or_import(py, &INSPECT_SIGNATURE_TYPE, "inspect", "Signature")
}

pub(super) fn get_inspect_parameter_type<'py>(py: Python<'py>) -> &'py Bound<'py, PyType> {
    static INSPECT_PARAMETER_TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();

    get_or_import(py, &INSPECT_PARAMETER_TYPE, "inspect", "Parameter")
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
