//! com/mojang/serialization/Encoder.java (DataFixerUpper 10.0.21)
//!
//! What writes a value into a `DynamicOps`' data. Java's interface is generic in the value's type
//! `A` and has methods generic in the data's type; here the trait is generic in the data's type
//! `T`, and `A` is `Valued`'s `Value`, which an encoder shares with its decoder. `comap` and
//! `flatComap` are `Codec`'s combinators (`codec.rs`), and `Encoder.empty()` is
//! `map_codec::Unit`'s encoder. `toString` isn't ported: no error message prints an encoder.

use std::sync::Arc;

use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};

/// What an encoder, decoder or codec reads and writes: Java's type argument `A`, which every
/// trait generic in the data's type shares.
pub trait Valued {
    type Value;
}

pub trait Encoder<T: OpsValue>: Valued {
    fn encode(&self, input: &Self::Value, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T>;

    fn encode_start(&self, ops: &dyn DynamicOps<T>, input: &Self::Value) -> DataResult<T> {
        self.encode(input, ops, ops.empty())
    }
}

impl<C: Valued + ?Sized> Valued for &C {
    type Value = C::Value;
}

impl<C: Valued + ?Sized> Valued for Box<C> {
    type Value = C::Value;
}

impl<C: Valued + ?Sized> Valued for Arc<C> {
    type Value = C::Value;
}

impl<T: OpsValue, C: Encoder<T> + ?Sized> Encoder<T> for &C {
    fn encode(&self, input: &Self::Value, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        (**self).encode(input, ops, prefix)
    }
}

impl<T: OpsValue, C: Encoder<T> + ?Sized> Encoder<T> for Box<C> {
    fn encode(&self, input: &Self::Value, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        (**self).encode(input, ops, prefix)
    }
}

impl<T: OpsValue, C: Encoder<T> + ?Sized> Encoder<T> for Arc<C> {
    fn encode(&self, input: &Self::Value, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        (**self).encode(input, ops, prefix)
    }
}
