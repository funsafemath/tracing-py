use pyo3::{exceptions::PyValueError, prelude::*};
use time::{
    format_description::{self, BorrowedFormatItem, OwnedFormatItem},
    macros::format_description,
};

#[derive(Clone)]
pub enum TimeFormat {
    Custom(OwnedFormatItem),
    Predefined(&'static [BorrowedFormatItem<'static>]),
    Rfc3339,
}

#[pyclass(name = "TimeFormat", from_py_object)]
#[derive(Clone)]
pub struct PyTimeFormat(TimeFormat);

impl PyTimeFormat {
    pub fn format(&self) -> &TimeFormat {
        &self.0
    }
}

#[pymethods]
impl PyTimeFormat {
    #[classattr]
    const YYYY_MM_DD_HH_MM_SS: Self = Self(TimeFormat::Predefined(format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second]"
    )));

    #[classattr]
    const MM_DD_HH_MM_SS: Self = Self(TimeFormat::Predefined(format_description!(
        "[month]-[day] [hour]:[minute]:[second]"
    )));

    #[classattr]
    const HH_MM_SS: Self = Self(TimeFormat::Predefined(format_description!(
        "[hour]:[minute]:[second]"
    )));

    #[classattr]
    const RFC_3339: Self = Self(TimeFormat::Rfc3339);

    #[new]
    fn new(format: String) -> PyResult<Self> {
        Ok(Self(TimeFormat::Custom(
            format_description::parse_owned::<2>(&format)
                .map_err(|e| PyValueError::new_err(e.to_string()))?,
        )))
    }
}

#[pyclass(from_py_object)]
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
// at most few times when the program starts
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
    #[pyo3(signature = (time = Time::Utc, format = |py| PyTimeFormat::RFC_3339))]
    fn new(time: Time, format: Bound<'_, PyAny>) -> Self {
        Self(Timer::Custom(time, format.0))
    }
}
