//! com/mojang/serialization/Decoder.java (DataFixerUpper 10.0.21)
//!
//! What reads a value out of a `DynamicOps`' data, with the rest of the input. Like `Encoder`,
//! the trait is generic in the data's type, its value `Valued`'s. `map`, `flatMap` and
//! `withLifecycle` are `Codec`'s combinators, and `Decoder.unit` is `map_codec::Unit`'s
//! decoder. The `Dynamic` overloads, `Terminal`, `Boxed` and `Simple` aren't ported: nothing the
//! game's codecs reach makes one.

use std::sync::Arc;

use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::Valued;

pub trait Decoder<T: OpsValue>: Valued {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<Self::Value, T>>;

    fn parse(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Self::Value> {
        self.decode(ops, input).map(Pair::into_first)
    }
}

impl<T: OpsValue, C: Decoder<T> + ?Sized> Decoder<T> for &C {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<Self::Value, T>> {
        (**self).decode(ops, input)
    }
}

impl<T: OpsValue, C: Decoder<T> + ?Sized> Decoder<T> for Box<C> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<Self::Value, T>> {
        (**self).decode(ops, input)
    }
}

impl<T: OpsValue, C: Decoder<T> + ?Sized> Decoder<T> for Arc<C> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<Self::Value, T>> {
        (**self).decode(ops, input)
    }
}
