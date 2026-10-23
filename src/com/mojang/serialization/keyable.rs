//! com/mojang/serialization/Keyable.java (DataFixerUpper 10.0.21)
//!
//! The keys a map codec reads and writes, which a compressing ops numbers. `forStrings` isn't
//! ported: nothing the game's codecs reach calls it.

use std::sync::Arc;

use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};

pub trait Keyable<T: OpsValue> {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T>;
}

impl<T: OpsValue, K: Keyable<T> + ?Sized> Keyable<T> for &K {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        (**self).keys(ops)
    }
}

impl<T: OpsValue, K: Keyable<T> + ?Sized> Keyable<T> for Box<K> {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        (**self).keys(ops)
    }
}

impl<T: OpsValue, K: Keyable<T> + ?Sized> Keyable<T> for Arc<K> {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        (**self).keys(ops)
    }
}
