pub mod debug;

use std::sync::{LazyLock, Mutex, MutexGuard};

use rapidhash::RapidHashSet;
use tracing::warn;

// todo: use rwlock to improve free-threaded performance
static LEAKED_STRINGS: LazyLock<Mutex<RapidHashSet<&'static str>>> = LazyLock::new(Mutex::default);
static LEAKED_SLICES: LazyLock<Mutex<RapidHashSet<&'static [&'static str]>>> =
    LazyLock::new(Mutex::default);

pub fn leak<T>(x: T) -> &'static T {
    Box::leak(Box::new(x))
}

// todo: generalize these structs
pub struct Leaker<'a> {
    guard: MutexGuard<'a, RapidHashSet<&'static str>>,
}

impl Leaker<'_> {
    pub fn acquire() -> Self {
        Self {
            guard: LEAKED_STRINGS.lock().unwrap(),
        }
    }

    fn leak_string(x: String) -> &'static str {
        Box::leak(x.into_boxed_str())
    }

    pub fn leak_or_get(&mut self, str: String) -> &'static str {
        if let Some(leaked) = self.guard.get(str.as_str()) {
            leaked
        } else {
            if self.guard.len() >= 100_000 {
                warn!(
                    "there are {} leaked strings, are you sure you're doing the right thing? using tracing in dynamically compiled code leaks memory",
                    self.guard.len()
                );
            }

            let leaked = Self::leak_string(str);
            self.guard.insert(leaked);
            leaked
        }
    }

    pub fn leak_or_get_once(str: String) -> &'static str {
        Self::acquire().leak_or_get(str)
    }
}

pub struct VecLeaker<'a> {
    guard: MutexGuard<'a, RapidHashSet<&'static [&'static str]>>,
}

impl VecLeaker<'_> {
    fn acquire() -> Self {
        Self {
            guard: LEAKED_SLICES.lock().unwrap(),
        }
    }

    fn leak_vec(x: Vec<&'static str>) -> &'static [&'static str] {
        Box::leak(x.into_boxed_slice())
    }

    pub fn leak_or_get(&mut self, item: Vec<&'static str>) -> &'static [&'static str] {
        if let Some(leaked) = self.guard.get(item.as_slice()) {
            leaked
        } else {
            if self.guard.len() >= 100_000 {
                warn!(
                    "there are {} leaked field combinations, are you sure you're doing the right thing? using tracing in dynamically compiled code or calling instrument() with different field combinations leaks memory",
                    self.guard.len()
                );
            }

            let leaked = Self::leak_vec(item);
            self.guard.insert(leaked);
            leaked
        }
    }

    pub fn leak_or_get_once(item: Vec<&'static str>) -> &'static [&'static str] {
        Self::acquire().leak_or_get(item)
    }
}
