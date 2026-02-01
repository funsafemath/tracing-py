use std::sync::{LazyLock, Mutex, MutexGuard};

use rapidhash::RapidHashSet;
use smallvec::SmallVec;
use tracing::warn;

// todo: use rwlock to improve free-threaded performance
static LEAKED_STRINGS: LazyLock<Mutex<RapidHashSet<&'static str>>> = LazyLock::new(Mutex::default);
static LEAKED_SLICES: LazyLock<Mutex<RapidHashSet<&'static [&'static str]>>> =
    LazyLock::new(Mutex::default);

pub(super) fn leak<T>(x: T) -> &'static T {
    Box::leak(Box::new(x))
}

// todo: generalize these structs
pub(super) struct Leaker<'a> {
    guard: MutexGuard<'a, RapidHashSet<&'static str>>,
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
                if self.guard.len() >= 100000 {
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
    }

    pub(super) fn leak_or_get_once(str: String) -> &'static str {
        Self::acquire().leak_or_get(str)
    }
}

pub(super) struct VecLeaker<'a> {
    guard: MutexGuard<'a, RapidHashSet<&'static [&'static str]>>,
}

impl<'a> VecLeaker<'a> {
    fn acquire() -> Self {
        Self {
            guard: LEAKED_SLICES.lock().unwrap(),
        }
    }

    fn leak_vec(x: SmallVec<[&'static str; 64]>) -> &'static [&'static str] {
        Box::leak(x.into_boxed_slice())
    }

    pub(super) fn leak_or_get(
        &mut self,
        item: SmallVec<[&'static str; 64]>,
    ) -> &'static [&'static str] {
        match self.guard.get(item.as_slice()) {
            Some(leaked) => leaked,
            None => {
                if self.guard.len() >= 100000 {
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
    }

    pub(super) fn leak_or_get_once(item: SmallVec<[&'static str; 64]>) -> &'static [&'static str] {
        Self::acquire().leak_or_get(item)
    }
}
