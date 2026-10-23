//! com/mojang/serialization/codecs/FieldEncoder.java (DataFixerUpper 10.0.21)
//!
//! A value written under a name. `Codec::field_of`'s `FieldOf` encodes with this and decodes with
//! `FieldDecoder`'s, Java's `MapCodec.of` of the two.

use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::Encoder;
use crate::com::mojang::serialization::record_builder::RecordBuilder;

/// `encode`: the value encoded on its own, added under the name.
pub fn encode<T: OpsValue, C: Encoder<T>>(
    name: &str,
    element_codec: &C,
    input: &C::Value,
    ops: &dyn DynamicOps<T>,
    prefix: &mut dyn RecordBuilder<T>,
) {
    let encoded_contents = element_codec.encode_start(ops, input);
    prefix.add_str_result(name, encoded_contents);
}
