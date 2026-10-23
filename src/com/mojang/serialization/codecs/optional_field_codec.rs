//! com/mojang/serialization/codecs/OptionalFieldCodec.java (DataFixerUpper 10.0.21)
//!
//! A value that may be under a name. A missing field is empty; one that fails to read is an
//! error, or empty when the codec's lenient.

use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::{Encoder, Valued};
use crate::com::mojang::serialization::keyable::Keyable;
use crate::com::mojang::serialization::map_decoder::MapDecoder;
use crate::com::mojang::serialization::map_encoder::MapEncoder;
use crate::com::mojang::serialization::map_like::MapLike;
use crate::com::mojang::serialization::record_builder::RecordBuilder;

#[derive(Clone)]
pub struct OptionalFieldCodec<C> {
    name: String,
    element_codec: C,
    lenient: bool,
}

impl<C> OptionalFieldCodec<C> {
    pub fn new(name: &str, element_codec: C, lenient: bool) -> OptionalFieldCodec<C> {
        OptionalFieldCodec {
            name: name.to_owned(),
            element_codec,
            lenient,
        }
    }
}

impl<C: Valued> Valued for OptionalFieldCodec<C> {
    type Value = Option<C::Value>;
}

impl<T: OpsValue, C> Keyable<T> for OptionalFieldCodec<C> {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        vec![ops.create_string(&self.name)]
    }
}

/// `decode`: an error keeps what it could read as its partial value, and an empty one when it
/// read nothing (`setPartial(parsed.resultOrPartial())`).
impl<T: OpsValue, C: Decoder<T>> MapDecoder<T> for OptionalFieldCodec<C> {
    fn decode(
        &self,
        ops: &dyn DynamicOps<T>,
        input: &dyn MapLike<T>,
    ) -> DataResult<Option<C::Value>> {
        let Some(value) = input.get_str(&self.name) else {
            return DataResult::success(None);
        };

        let parsed = self.element_codec.parse(ops, value);
        if parsed.is_error() && self.lenient {
            return DataResult::success(None);
        }
        let lifecycle = parsed.lifecycle();
        match parsed.split() {
            (value, None) => DataResult::success_lifecycle(value, lifecycle),
            (partial, Some(message)) => {
                DataResult::error_partial_lifecycle(|| message, partial, lifecycle)
            }
        }
    }
}

impl<T: OpsValue, C: Encoder<T>> MapEncoder<T> for OptionalFieldCodec<C> {
    fn encode(
        &self,
        input: &Option<C::Value>,
        ops: &dyn DynamicOps<T>,
        prefix: &mut dyn RecordBuilder<T>,
    ) {
        if let Some(value) = input {
            prefix.add_str_result(&self.name, self.element_codec.encode_start(ops, value));
        }
    }
}
