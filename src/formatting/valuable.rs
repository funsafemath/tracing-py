// use std::fmt::{Display, Formatter};

// use pyo3::{Bound, PyAny, types::PyAnyMethods};
// use valuable::{Valuable, Value, Visit};

// use crate::{
//     cached::{CachedValuable, CachedValue, GetValue},
//     ext::template::PyTemplateMethodsExt,
//     template::template_string::PyTemplate,
// };

// // todo: rework the entire valuable implementation, i don't like the current one

// impl<'py, T> GetValue<String, CachedValuable> for Bound<'py, T> {
//     fn value(&self) -> String {
//         self.to_string()
//     }
// }

// impl<'py, T> GetValue<f64, CachedValuable> for Bound<'py, T> {
//     fn value(&self) -> f64 {
//         self.as_any().extract().unwrap()
//     }
// }

// pub(crate) enum PyCachedValuable<'py> {
//     Any(CachedValue<String, Bound<'py, PyAny>, CachedValuable>),
//     Float(CachedValue<f64, Bound<'py, PyAny>, CachedValuable>),
//     Template(CachedValue<String, Bound<'py, PyTemplate>, CachedValuable>),
// }

// // todo: log lists/dicts/bools/ints/floats/nulls as their respective json types
// impl<'py> Valuable for PyCachedValuable<'py> {
//     fn as_value(&self) -> Value<'_> {
//         match self {
//             PyCachedValuable::Any(cached_value) => cached_value.as_value(),
//             PyCachedValuable::Template(cached_value) => cached_value.as_value(),
//             PyCachedValuable::Float(cached_value) => cached_value.as_value(),
//         }
//     }

//     fn visit(&self, visit: &mut dyn Visit) {
//         visit.visit_value(self.as_value());
//     }
// }

// impl<'py> From<Bound<'py, PyAny>> for PyCachedValuable<'py> {
//     fn from(value: Bound<'py, PyAny>) -> Self {
//         match value.cast_into::<PyTemplate>() {
//             Ok(template) => PyCachedValuable::Template(template.into()),
//             Err(value) => PyCachedValuable::Any(value.into_inner().into()),
//         }
//     }
// }

// impl<'py> Display for PyCachedValuable<'py> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let string = match self {
//             PyCachedValuable::Any(cached_value) => cached_value.inner_ref().to_string(),
//             PyCachedValuable::Template(cached_value) => cached_value.inner_ref().format(),
//             PyCachedValuable::Float(cached_value) => cached_value.inner_ref().to_string(),
//         };
//         write!(f, "{string}")
//     }
// }
