//! com/mojang/serialization/codecs/FieldDecoder.java (DataFixerUpper 10.0.21)
//!
//! A value read from under a name, which has to be there.

use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::map_like::MapLike;

/// `decode`: the value under the name, parsed, or an error naming the map.
pub fn decode<T: OpsValue, C: Decoder<T>>(
    name: &str,
    element_codec: &C,
    ops: &dyn DynamicOps<T>,
    input: &dyn MapLike<T>,
) -> DataResult<C::Value> {
    match input.get_str(name) {
        None => DataResult::error(|| format!("No key {name} in {input}")),
        Some(value) => element_codec.parse(ops, value),
    }
}
