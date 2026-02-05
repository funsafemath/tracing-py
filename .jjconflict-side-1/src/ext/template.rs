use pyo3::{Bound, types::PyTupleMethods};

use crate::template::{
    interpolation::{PyInterpolation, PyInterpolationMethods},
    template_string::{PyTemplate, PyTemplateMethods},
};

pub trait PyTemplateMethodsExt<'py> {
    fn format(&self) -> String;
}

impl<'py> PyTemplateMethodsExt<'py> for Bound<'py, PyTemplate> {
    // todo: use conversions & format specifiers
    fn format(&self) -> String {
        let strings = self.strings();
        let interpolations = self.interpolations();

        assert!(strings.len() == interpolations.len() + 1);

        let mut strings = strings.iter();
        let interpolations = interpolations.iter();

        let mut result = strings.next().unwrap().to_string();

        // probably could be improved by using interpolation __iter__, as it skips empty strings
        // not sure if this will have a performance benefit
        for (interp, str) in interpolations.zip(strings) {
            let interp = interp.cast::<PyInterpolation>().unwrap();
            result += &interp.value().to_string();
            result += &str.to_string();
        }

        result
    }
}
