use pyo3::{prelude::*, sync::PyOnceLock};

use crate::{
    ext::any::infallible_attr, imports::get_inspect_parameter_type, py_type::mk_imported_type,
};

mk_imported_type!(PyParameter, "inspect", "Parameter");

pub(crate) trait PyParameterMethods<'py> {
    fn kind(&self) -> ParamKind;
}

impl<'py> PyParameterMethods<'py> for Bound<'py, PyParameter> {
    fn kind(&self) -> ParamKind {
        ParamKind::maybe_from(infallible_attr!(self, "kind")).unwrap()
    }
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
    fn maybe_from(param: Bound<'_, PyAny>) -> Option<Self> {
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
                infallible_attr!(get_inspect_parameter_type(py), "POSITIONAL_ONLY").unbind()
            })
            .bind(py)
    }

    fn pos_kw(py: Python<'_>) -> &Bound<'_, PyAny> {
        static POS_KW: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

        POS_KW
            .get_or_init(py, || {
                infallible_attr!(get_inspect_parameter_type(py), "POSITIONAL_OR_KEYWORD").unbind()
            })
            .bind(py)
    }

    fn excess_args(py: Python<'_>) -> &Bound<'_, PyAny> {
        static EXCESS_ARGS: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

        EXCESS_ARGS
            .get_or_init(py, || {
                infallible_attr!(get_inspect_parameter_type(py), "VAR_POSITIONAL").unbind()
            })
            .bind(py)
    }

    fn kw_only(py: Python<'_>) -> &Bound<'_, PyAny> {
        static KW_ONLY: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

        KW_ONLY
            .get_or_init(py, || {
                infallible_attr!(get_inspect_parameter_type(py), "KEYWORD_ONLY").unbind()
            })
            .bind(py)
    }

    fn excess_kwargs(py: Python<'_>) -> &Bound<'_, PyAny> {
        static EXCESS_KWARGS: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

        EXCESS_KWARGS
            .get_or_init(py, || {
                infallible_attr!(get_inspect_parameter_type(py), "VAR_KEYWORD").unbind()
            })
            .bind(py)
    }
}
