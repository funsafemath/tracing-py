use valuable::Valuable;

use crate::cached::{CachedValue, GetValue};

pub enum CachedValuable {}

impl<V: Valuable, T: GetValue<V, CachedValuable>> Valuable for CachedValue<V, T, CachedValuable> {
    fn as_value(&self) -> valuable::Value<'_> {
        // self.get_or_init().as_value()
        valuable::Value::F64(123.0)
    }

    fn visit(&self, visit: &mut dyn valuable::Visit) {
        visit.visit_value(self.as_value());
    }
}
