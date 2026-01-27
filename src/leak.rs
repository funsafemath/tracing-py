use std::{
    collections::HashSet,
    sync::{LazyLock, Mutex, MutexGuard},
};

use pyo3::{
    Bound,
    types::{PyDict, PyDictMethods},
};

use crate::valuable::PyCachedValuable;

static LEAKED_STRINGS: LazyLock<Mutex<HashSet<&'static str>>> = LazyLock::new(Mutex::default);
static LEAKED_SLICES: LazyLock<Mutex<HashSet<&'static [&'static str]>>> =
    LazyLock::new(Mutex::default);

pub(super) fn leak<T>(x: T) -> &'static T {
    Box::leak(Box::new(x))
}

pub(crate) fn leak_or_get_kwargs<'py>(
    // todo: accept a mut ref
    leaker: Option<Leaker>,
    kwargs: Option<&Bound<'py, PyDict>>,
) -> (Vec<&'static str>, Vec<PyCachedValuable<'py>>) {
    let mut fields = vec![];
    let mut values = vec![];

    if let Some(kwargs) = kwargs {
        let mut leaker = leaker.unwrap_or(Leaker::acquire());

        for (key, value) in kwargs.iter() {
            fields.push(leaker.leak_or_get(key.to_string()));
            values.push(PyCachedValuable::from(value));
        }
    }

    (fields, values)
}

// todo: generalize these structs
pub(super) struct Leaker<'a> {
    guard: MutexGuard<'a, HashSet<&'static str>>,
}

impl<'a> Leaker<'a> {
    pub(super) fn acquire() -> Self {
        Self {
            guard: LEAKED_STRINGS.lock().unwrap(),
        }
    }

    fn leak_string(x: String) -> &'static str {
        Box::leak(x.into_boxed_str())
    }

    pub(super) fn leak_or_get(&mut self, str: String) -> &'static str {
        match self.guard.get(str.as_str()) {
            Some(leaked) => leaked,
            None => {
                let leaked = Self::leak_string(str);
                self.guard.insert(leaked);
                leaked
            }
        }
    }

    pub(super) fn leak_or_get_once(str: String) -> &'static str {
        Self::acquire().leak_or_get(str)
    }
}

pub(super) struct VecLeaker<'a> {
    guard: MutexGuard<'a, HashSet<&'static [&'static str]>>,
}

impl<'a> VecLeaker<'a> {
    fn acquire() -> Self {
        Self {
            guard: LEAKED_SLICES.lock().unwrap(),
        }
    }

    fn leak_vec(x: Vec<&'static str>) -> &'static [&'static str] {
        Box::leak(x.into_boxed_slice())
    }

    pub(super) fn leak_or_get(&mut self, item: Vec<&'static str>) -> &'static [&'static str] {
        match self.guard.get(item.as_slice()) {
            Some(leaked) => leaked,
            None => {
                let leaked = Self::leak_vec(item);
                self.guard.insert(leaked);
                leaked
            }
        }
    }

    pub(super) fn leak_or_get_once(item: Vec<&'static str>) -> &'static [&'static str] {
        Self::acquire().leak_or_get(item)
    }
}
