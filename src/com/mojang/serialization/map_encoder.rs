//! com/mojang/serialization/MapEncoder.java (DataFixerUpper 10.0.21)
//!
//! What writes a value's fields into a record builder. The builder is changed in place, where
//! Java's returns it (`RecordBuilder`). `compressedBuilder` is the ops' `mapBuilder()`: no ops the
//! game's codecs reach compresses maps, so `makeCompressedBuilder` and `KeyCompressor` aren't
//! ported, and a compressing ops panics. `comap`, `flatComap` and `withLifecycle` are
//! `MapCodec`'s combinators.

use std::sync::Arc;

use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::Valued;
use crate::com::mojang::serialization::keyable::Keyable;
use crate::com::mojang::serialization::record_builder::RecordBuilder;

pub trait MapEncoder<T: OpsValue>: Valued + Keyable<T> {
    fn encode(
        &self,
        input: &Self::Value,
        ops: &dyn DynamicOps<T>,
        prefix: &mut dyn RecordBuilder<T>,
    );
}

/// `compressedBuilder`: the ops' record builder.
pub fn compressed_builder<'a, T: OpsValue>(
    ops: &'a dyn DynamicOps<T>,
) -> Box<dyn RecordBuilder<T> + 'a> {
    assert!(
        !ops.compress_maps(),
        "compressed maps (KeyCompressor) aren't ported"
    );
    ops.map_builder()
}

impl<T: OpsValue, C: MapEncoder<T> + ?Sized> MapEncoder<T> for &C {
    fn encode(
        &self,
        input: &Self::Value,
        ops: &dyn DynamicOps<T>,
        prefix: &mut dyn RecordBuilder<T>,
    ) {
        (**self).encode(input, ops, prefix)
    }
}

impl<T: OpsValue, C: MapEncoder<T> + ?Sized> MapEncoder<T> for Box<C> {
    fn encode(
        &self,
        input: &Self::Value,
        ops: &dyn DynamicOps<T>,
        prefix: &mut dyn RecordBuilder<T>,
    ) {
        (**self).encode(input, ops, prefix)
    }
}

impl<T: OpsValue, C: MapEncoder<T> + ?Sized> MapEncoder<T> for Arc<C> {
    fn encode(
        &self,
        input: &Self::Value,
        ops: &dyn DynamicOps<T>,
        prefix: &mut dyn RecordBuilder<T>,
    ) {
        (**self).encode(input, ops, prefix)
    }
}
