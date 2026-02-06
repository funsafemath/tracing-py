use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::layer::fmt::time::format::{PyTimeFormat, TimeFormat};

#[pyclass(rename_all = "UPPERCASE", from_py_object)]
#[derive(Clone)]
pub enum Time {
    Utc,
    Local,
}

#[derive(Clone)]
pub enum Timer {
    SystemTime,
    Uptime,
    Custom(Time, TimeFormat),
}

// yes, from_py_object for non-trivially copyable types is inefficient, but who cares, it's for python, and it'll be called
// at most a few times when the program starts
#[pyclass(name = "Timer", from_py_object)]
#[derive(Clone)]
pub struct PyTimer(Timer);

impl PyTimer {
    pub fn timer(&self) -> &Timer {
        &self.0
    }
}

#[pymethods]
impl PyTimer {
    #[classattr]
    pub const SYSTEM_TIME: Self = Self(Timer::SystemTime);

    #[classattr]
    pub const UPTIME: Self = Self(Timer::Uptime);

    #[new]
    #[pyo3(signature = (format = Python::attach(|py| Py::new(py, PyTimeFormat::ISO_8601).unwrap().into_any()), time = Time::Utc))]
    fn new(py: Python<'_>, format: Py<PyAny>, time: Time) -> PyResult<Self> {
        let format = format.into_bound(py);

        let format = if let Ok(string) = format.extract() {
            TimeFormat::new(string)?
        } else if let Ok(format) = format.cast_into::<PyTimeFormat>() {
            format.borrow().format().clone()
        } else {
            return Err(PyTypeError::new_err(
                "`format` type must be TimeFormat or str",
            ));
        };
        Ok(Self(Timer::Custom(time, format)))
    }
}

macro mk_human_utc($(($name:ident, $format:ident),)*) {
    #[pymethods]
    impl PyTimer {
        $(
            #[classattr]
            pub const $name: Self = Self(Timer::Custom(Time::Utc, TimeFormat::$format));
        )*
    }
}

macro mk_human_local($(($name:ident, $format:ident),)*) {
    #[pymethods]
    impl PyTimer {
        $(
            #[classattr]
            pub const $name: Self = Self(Timer::Custom(Time::Local, TimeFormat::$format));
        )*
    }
}

// todo: use proc macro and add milliseconds variants
mk_human_utc!(
    (HUMAN_YMD_TIME_UTC, YYYY_MM_DD_HH_MM_SS),
    (HUMAN_MD_TIME_UTC, MM_DD_HH_MM_SS),
    (HUMAN_TIME_UTC, HH_MM_SS),
);

mk_human_local!(
    (HUMAN_YMD_TIME_LOCAL, YYYY_MM_DD_HH_MM_SS_OFFSET),
    (HUMAN_MD_TIME_LOCAL, MM_DD_HH_MM_SS_OFFSET),
    (HUMAN_TIME_LOCAL, HH_MM_SS_OFFSET),
    // using these for logs is usually a bad idea, but that's user's responsibility
    (HUMAN_YMD_TIME_LOCAL_NO_OFFSET, YYYY_MM_DD_HH_MM_SS),
    (HUMAN_MD_TIME_LOCAL_NO_OFFSET, MM_DD_HH_MM_SS),
    (HUMAN_TIME_LOCAL_NO_OFFSET, HH_MM_SS),
);
