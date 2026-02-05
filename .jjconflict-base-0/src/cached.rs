mod display;
mod valuable;

pub use display::CachedDisplay;
pub use valuable::CachedValuable;

use std::{cell::OnceCell, marker::PhantomData};

pub trait GetValue<V, M> {
    fn value(&self) -> V;
}

// marker allows to switch between blanket implementations, the type system is quite cool
pub struct CachedValue<V, T: GetValue<V, M>, M> {
    inner: T,
    cached: OnceCell<V>,
    marker: PhantomData<M>,
}

impl<V, T: GetValue<V, M>, M> From<T> for CachedValue<V, T, M> {
    fn from(value: T) -> Self {
        Self {
            inner: value,
            cached: OnceCell::new(),
            marker: PhantomData,
        }
    }
}

impl<V, T: GetValue<V, M>, M> CachedValue<V, T, M> {
    pub fn get_or_init(&self) -> &V {
        self.cached.get_or_init(|| self.inner.value())
    }
}
