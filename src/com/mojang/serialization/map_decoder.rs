//! com/mojang/serialization/MapDecoder.java (DataFixerUpper 10.0.21)
//!
//! What reads a value from a map's entries. `compressedDecode` reads the input as a map (its
//! compressed path, a list numbered by a `KeyCompressor`, isn't ported, and panics). `map`,
//! `flatMap`, `ap` and `withLifecycle` are `MapCodec`'s combinators.

use std::sync::Arc;

use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::Valued;
use crate::com::mojang::serialization::keyable::Keyable;
use crate::com::mojang::serialization::lifecycle::Lifecycle;
use crate::com::mojang::serialization::map_like::MapLike;

pub trait MapDecoder<T: OpsValue>: Valued + Keyable<T> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<Self::Value>;

    /// The input read as a map, stable, then decoded.
    fn compressed_decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Self::Value> {
        assert!(
            !ops.compress_maps(),
            "compressed maps (KeyCompressor) aren't ported"
        );
        ops.get_map(input)
            .set_lifecycle(Lifecycle::stable())
            .flat_map(|map| self.decode(ops, &*map))
    }
}

impl<T: OpsValue, C: MapDecoder<T> + ?Sized> MapDecoder<T> for &C {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<Self::Value> {
        (**self).decode(ops, input)
    }
}

impl<T: OpsValue, C: MapDecoder<T> + ?Sized> MapDecoder<T> for Box<C> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<Self::Value> {
        (**self).decode(ops, input)
    }
}

impl<T: OpsValue, C: MapDecoder<T> + ?Sized> MapDecoder<T> for Arc<C> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<Self::Value> {
        (**self).decode(ops, input)
    }
}
