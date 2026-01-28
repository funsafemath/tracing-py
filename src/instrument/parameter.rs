use pyo3::{prelude::*, sync::PyOnceLock, PyTypeInfo};

use crate::{any_ext::InfallibleAttr, imports::get_inspect_parameter_type};

#[repr(transparent)]
pub(crate) struct PyParameter(PyAny);

// SAFETY: type_object_raw infallibly produces a valid pointer to the type object
unsafe impl PyTypeInfo for PyParameter {
    const NAME: &'static str = "Parameter";

    const MODULE: Option<&'static str> = Some("inspect");

    fn type_object_raw(py: Python<'_>) -> *mut pyo3::ffi::PyTypeObject {
        get_inspect_parameter_type(py).as_type_ptr()
    }
}

pub(crate) trait PyParameterMethods<'py> {
    fn kind(&self) -> ParamKind;
    fn has_default(&self) -> bool;
}

impl<'py> PyParameterMethods<'py> for Bound<'py, PyParameter> {
    fn kind(&self) -> ParamKind {
        ParamKind::maybe_from(self.infallible_attr::<"kind", PyAny>().as_borrowed()).unwrap()
    }

    fn has_default(&self) -> bool {
        // funnily enough, if a function specifies inspect.Parameter.empty as the default parameter value,
        // inspect's functions will treat it as if there is no default parameter
        // todo: fix it??
        // also it breaks the invariant that required parameters can follow optional ones (inspect doesn't validate it)
        !self
            .infallible_attr::<"default", PyAny>()
            .is(empty_default(self.py()))
    }
}

fn empty_default(py: Python<'_>) -> &Bound<'_, PyAny> {
    static EMPTY_DEFAULT: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

    EMPTY_DEFAULT
        .get_or_init(py, || {
            get_inspect_parameter_type(py)
                .infallible_attr::<"empty", PyAny>()
                .unbind()
        })
        .bind(py)
}

#[derive(Debug)]
pub(crate) enum ParamKind {
    PositionalOnly,
    PositionalOrKeyword,
    ExcessArgs,
    KeywordOnly,
    ExcessKwargs,
}

impl ParamKind {
    fn maybe_from(param: Borrowed<'_, '_, PyAny>) -> Option<Self> {
        let py = param.py();

        if param.is(Self::pos_only(py)) {
            Some(ParamKind::PositionalOnly)
        } else if param.is(Self::pos_kw(py)) {
            Some(ParamKind::PositionalOrKeyword)
        } else if param.is(Self::excess_args(py)) {
            Some(ParamKind::ExcessArgs)
        } else if param.is(Self::kw_only(py)) {
            Some(ParamKind::KeywordOnly)
        } else if param.is(Self::excess_kwargs(py)) {
            Some(ParamKind::ExcessKwargs)
        } else {
            None
        }
    }

    fn pos_only(py: Python<'_>) -> &Bound<'_, PyAny> {
        static POS_ONLY: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

        POS_ONLY
            .get_or_init(py, || {
                get_inspect_parameter_type(py)
                    .infallible_attr::<"POSITIONAL_ONLY", PyAny>()
                    .unbind()
            })
            .bind(py)
    }

    fn pos_kw(py: Python<'_>) -> &Bound<'_, PyAny> {
        static POS_KW: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

        POS_KW
            .get_or_init(py, || {
                get_inspect_parameter_type(py)
                    .infallible_attr::<"POSITIONAL_OR_KEYWORD", PyAny>()
                    .unbind()
            })
            .bind(py)
    }

    fn excess_args(py: Python<'_>) -> &Bound<'_, PyAny> {
        static EXCESS_ARGS: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

        EXCESS_ARGS
            .get_or_init(py, || {
                get_inspect_parameter_type(py)
                    .infallible_attr::<"VAR_POSITIONAL", PyAny>()
                    .unbind()
            })
            .bind(py)
    }

    fn kw_only(py: Python<'_>) -> &Bound<'_, PyAny> {
        static KW_ONLY: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

        KW_ONLY
            .get_or_init(py, || {
                get_inspect_parameter_type(py)
                    .infallible_attr::<"KEYWORD_ONLY", PyAny>()
                    .unbind()
            })
            .bind(py)
    }

    fn excess_kwargs(py: Python<'_>) -> &Bound<'_, PyAny> {
        static EXCESS_KWARGS: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

        EXCESS_KWARGS
            .get_or_init(py, || {
                get_inspect_parameter_type(py)
                    .infallible_attr::<"VAR_KEYWORD", PyAny>()
                    .unbind()
            })
            .bind(py)
    }
}
