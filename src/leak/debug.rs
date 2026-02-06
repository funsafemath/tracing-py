use pyo3::prelude::*;

use crate::{
    callsite::leaked_callsites_count,
    leak::{LEAKED_SLICES, LEAKED_STRINGS},
};

#[pyclass(get_all)]
#[derive(Debug)]
pub struct LeakInfo {
    callsites: usize,
    strings: usize,
    slices: usize,
}

#[pymethods]
impl LeakInfo {
    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

#[pyfunction]
pub fn leak_info() -> LeakInfo {
    LeakInfo {
        callsites: leaked_callsites_count(),
        strings: LEAKED_STRINGS.lock().unwrap().len(),
        slices: LEAKED_SLICES.lock().unwrap().len(),
    }
}
