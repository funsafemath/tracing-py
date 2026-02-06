use pyo3::{exceptions::PyValueError, prelude::*};
use time::format_description::{self, BorrowedFormatItem, OwnedFormatItem};

#[derive(Clone)]
pub enum TimeFormat {
    Custom(OwnedFormatItem),
    Predefined(&'static [BorrowedFormatItem<'static>]),
    Iso8601,
    Iso8601NoSubseconds,
}

impl TimeFormat {
    pub fn new(format: String) -> PyResult<Self> {
        Ok(Self::Custom(
            format_description::parse_owned::<2>(&format)
                .map_err(|e| PyValueError::new_err(e.to_string()))?,
        ))
    }
}

#[pyclass(name = "TimeFormat", from_py_object)]
#[derive(Clone)]
pub struct PyTimeFormat(TimeFormat);

impl PyTimeFormat {
    pub fn format(&self) -> &TimeFormat {
        &self.0
    }
}

// pymethods doesn't allow macros inside?
macro mk_time_format_constants($($format:ident,)*) {
    impl TimeFormat {
        $(pub const $format: Self = Self::Predefined(super::formats::$format);)*
    }

    #[pymethods]
    impl PyTimeFormat {
        $(
            #[classattr]
            const $format: Self = Self(TimeFormat::$format);
        )*
    }
}

mk_time_format_constants!(
    YYYY_MM_DD_HH_MM_SS_OFFSET,
    YYYY_MM_DD_HH_MM_SS,
    MM_DD_HH_MM_SS_OFFSET,
    MM_DD_HH_MM_SS,
    HH_MM_SS_OFFSET,
    HH_MM_SS,
);

#[pymethods]
impl PyTimeFormat {
    #[classattr]
    pub const ISO_8601: Self = Self(TimeFormat::Iso8601);

    #[classattr]
    pub const ISO8601_NO_SUBSECONDS: Self = Self(TimeFormat::Iso8601NoSubseconds);
}
