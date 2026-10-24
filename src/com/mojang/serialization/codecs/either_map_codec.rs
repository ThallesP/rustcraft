//! com/mojang/serialization/codecs/EitherMapCodec.java (DataFixerUpper 10.0.21)
//!
//! One of two map codecs: the first's value where it reads, else the second's, else both errors
//! (`apply2`, keeping the second's partial value).

use crate::com::mojang::datafixers::util::either::Either;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::Valued;
use crate::com::mojang::serialization::keyable::Keyable;
use crate::com::mojang::serialization::map_decoder::MapDecoder;
use crate::com::mojang::serialization::map_encoder::MapEncoder;
use crate::com::mojang::serialization::map_like::MapLike;
use crate::com::mojang::serialization::record_builder::RecordBuilder;

#[derive(Clone)]
pub struct EitherMapCodec<F, S> {
    first: F,
    second: S,
}

impl<F, S> EitherMapCodec<F, S> {
    pub fn new(first: F, second: S) -> EitherMapCodec<F, S> {
        EitherMapCodec { first, second }
    }
}

impl<F: Valued, S: Valued> Valued for EitherMapCodec<F, S> {
    type Value = Either<F::Value, S::Value>;
}

/// Both codecs' keys, the first's then the second's.
impl<T: OpsValue, F: Keyable<T>, S: Keyable<T>> Keyable<T> for EitherMapCodec<F, S> {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        let mut keys = self.first.keys(ops);
        keys.extend(self.second.keys(ops));
        keys
    }
}

impl<T: OpsValue, F: MapDecoder<T>, S: MapDecoder<T>> MapDecoder<T> for EitherMapCodec<F, S> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<Self::Value> {
        let first_read = self.first.decode(ops, input).map(Either::Left);
        if first_read.is_success() {
            return first_read;
        }
        let second_read = self.second.decode(ops, input).map(Either::Right);
        if second_read.is_success() {
            second_read
        } else {
            first_read.apply2(|_, second| second, second_read)
        }
    }
}

impl<T: OpsValue, F: MapEncoder<T>, S: MapEncoder<T>> MapEncoder<T> for EitherMapCodec<F, S> {
    fn encode(
        &self,
        input: &Self::Value,
        ops: &dyn DynamicOps<T>,
        prefix: &mut dyn RecordBuilder<T>,
    ) {
        match input {
            Either::Left(value) => self.first.encode(value, ops, prefix),
            Either::Right(value) => self.second.encode(value, ops, prefix),
        }
    }
}
