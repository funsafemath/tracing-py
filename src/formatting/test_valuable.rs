use pyo3::{
    prelude::*,
    types::{PyFloat, PyInt, PyString},
};
use valuable::{Valuable, Value};

pub struct PyValuable<'py>(Bound<'py, PyAny>);

impl<'py> Valuable for PyValuable<'py> {
    // defer extraction/type checks as much as possible
    fn as_value(&self) -> valuable::Value<'_> {
        if let Ok(int) = self.0.cast::<PyInt>() {
            let num: i128 = int.extract().unwrap();
            Value::I128(num)
        } else if let Ok(int) = self.0.cast::<PyFloat>() {
            let num: f64 = int.extract().unwrap();
            Value::F64(num)
        } else {
            Value::Unit
            // Value::String(
            //     self.0
            //         .str()
            //         .unwrap()
            //         .cast_into::<PyString>()
            //         .unwrap()
            //         .to_str()
            //         .unwrap(),
            // )
        }
    }

    fn visit(&self, visit: &mut dyn valuable::Visit) {
        self.as_value().visit(visit);
    }
}
