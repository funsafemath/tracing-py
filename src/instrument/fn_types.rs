pub mod async_generator;
pub mod coroutine;
pub mod generator;

use pyo3::prelude::*;

use crate::imports;

pub enum FunctionType {
    Normal,
    Generator,
    Async,
    AsyncGenerator,
}

impl FunctionType {
    pub fn guess_from_return_value(ret_val: &Bound<'_, PyAny>) -> PyResult<Self> {
        let py = ret_val.py();

        Ok(if ret_val.is_instance(imports::get_coroutine_type(py))? {
            Self::Async
        } else if ret_val.is_instance(imports::get_generator_type(py))? {
            Self::Generator
        } else if ret_val.is_instance(imports::get_async_generator_type(py))? {
            Self::AsyncGenerator
        } else {
            Self::Normal
        })
    }
}
