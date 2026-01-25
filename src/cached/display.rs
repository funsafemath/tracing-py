use std::fmt::Display;

use crate::cached::{CachedValue, GetValue};

pub enum CachedDisplay {}

impl<T: Display> GetValue<String, CachedDisplay> for T {
    fn value(&self) -> String {
        self.to_string()
    }
}

impl<T: GetValue<String, CachedDisplay>> Display for CachedValue<String, T, CachedDisplay> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_or_init())
    }
}
