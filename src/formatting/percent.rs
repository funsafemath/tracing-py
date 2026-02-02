use std::fmt::{self, Display};

use pyo3::{
    Bound,
    types::{PyAnyMethods, PyString, PyTuple},
};

// Args are intentionally PyTuple, not a PyAny, even though you can pass a single non-tuple argument to str.__mod__
// IMO accepting PyAny is a bad design on the Python side, as it's inconsistent and makes a bad function signature;
// also if you try to use a list instead of a tuple, you'll get a runtime error
//
// Accepting a tuple here also makes the users much less likely to make mistakes like using info(some string object, obj2),
// as type-checker will complain that obj2 is not a tuple.
// todo: should percent formatting be moved to a submodule and be accessible like tracing.legacy.function?
// That'll reduce the number possible mistakes and also allow to use *args to log multiple arbitrary objects
pub(crate) struct PercentFormatted<'a, 'py> {
    message: Bound<'py, PyString>,
    args: &'a Bound<'py, PyTuple>,
}

impl<'a, 'py> PercentFormatted<'a, 'py> {
    pub(crate) fn new(message: Bound<'py, PyString>, args: &'a Bound<'py, PyTuple>) -> Self {
        Self { message, args }
    }
}

// todo: PyUnicode_Format may be a bit faster, rewrite this function using it if you have nothing better to do
impl Display for PercentFormatted<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Is there any way to gracefully handle the error?
        // returning a fmt error panics,
        // and I can't check if arguments are valid for a given string before fmt() is called
        // (that's the point of lazy formatting)
        fmt::Display::fmt(
            &self
                .message
                .rem(self.args)
                .expect("failed to format a string"),
            f,
        )
    }
}
