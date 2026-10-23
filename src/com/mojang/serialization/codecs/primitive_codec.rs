//! com/mojang/serialization/codecs/PrimitiveCodec.java (DataFixerUpper 10.0.21)
//!
//! A codec of one of the ops' own kinds of value: read and written whole, with nothing left of
//! the input. Its default `decode` and `encode` are functions each primitive codec's impls call.

use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::Valued;

pub trait PrimitiveCodec<T: OpsValue>: Valued {
    fn read(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Self::Value>;

    fn write(&self, ops: &dyn DynamicOps<T>, value: &Self::Value) -> T;
}

/// `decode`: the value read, with the ops' empty value as the rest.
pub fn decode<T: OpsValue, P: PrimitiveCodec<T>>(
    codec: &P,
    ops: &dyn DynamicOps<T>,
    input: &T,
) -> DataResult<Pair<P::Value, T>> {
    codec
        .read(ops, input)
        .map(|result| Pair::new(result, ops.empty()))
}

/// `encode`: the value written, merged into the prefix as a primitive.
pub fn encode<T: OpsValue, P: PrimitiveCodec<T>>(
    codec: &P,
    input: &P::Value,
    ops: &dyn DynamicOps<T>,
    prefix: T,
) -> DataResult<T> {
    ops.merge_to_primitive(&prefix, codec.write(ops, input))
}
