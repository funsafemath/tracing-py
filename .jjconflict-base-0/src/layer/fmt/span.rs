use pyo3::prelude::*;
use tracing_subscriber::fmt::format::FmtSpan;

#[pyclass(name = "FmtSpan")]
pub struct PyFmtSpan(FmtSpan);

#[pymethods]
impl PyFmtSpan {
    #[classattr]
    pub const NEW: Self = Self(FmtSpan::NEW);

    #[classattr]
    pub const ENTER: Self = Self(FmtSpan::ENTER);

    #[classattr]
    pub const EXIT: Self = Self(FmtSpan::EXIT);

    #[classattr]
    pub const CLOSE: Self = Self(FmtSpan::CLOSE);

    #[classattr]
    pub const NONE: Self = Self(FmtSpan::NONE);

    #[classattr]
    pub const ACTIVE: Self = Self(FmtSpan::ACTIVE);

    #[classattr]
    pub const FULL: Self = Self(FmtSpan::FULL);

    fn __or__(&self, other: Bound<'_, Self>) -> Self {
        Self(self.0.clone() | other.borrow().0.clone())
    }

    fn __and__(&self, other: Bound<'_, Self>) -> Self {
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
