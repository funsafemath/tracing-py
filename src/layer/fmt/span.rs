use pyo3::prelude::*;
use tracing_subscriber::fmt::format::FmtSpan;

#[pyclass(name = "FmtSpan")]
pub(crate) struct PyFmtSpan(FmtSpan);

#[pymethods]
impl PyFmtSpan {
    #[classattr]
    pub(crate) const NEW: Self = Self(FmtSpan::NEW);

    #[classattr]
    pub(crate) const ENTER: Self = Self(FmtSpan::ENTER);

    #[classattr]
    pub(crate) const EXIT: Self = Self(FmtSpan::EXIT);

    #[classattr]
    pub(crate) const CLOSE: Self = Self(FmtSpan::CLOSE);

    #[classattr]
    pub(crate) const NONE: Self = Self(FmtSpan::NONE);

    #[classattr]
    pub(crate) const ACTIVE: Self = Self(FmtSpan::ACTIVE);

    #[classattr]
    pub(crate) const FULL: Self = Self(FmtSpan::FULL);

    fn __or__<'py>(&self, other: Bound<'py, Self>) -> Self {
        Self(self.0.clone() | other.borrow().0.clone())
    }

    fn __and__<'py>(&self, other: Bound<'py, Self>) -> Self {
        Self(self.0.clone() & other.borrow().0.clone())
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

impl std::fmt::Debug for PyFmtSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&PyFmtSpan> for FmtSpan {
    fn from(value: &PyFmtSpan) -> Self {
        value.0.clone()
    }
}
