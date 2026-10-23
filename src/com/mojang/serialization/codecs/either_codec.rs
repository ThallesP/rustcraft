//! com/mojang/serialization/codecs/EitherCodec.java (DataFixerUpper 10.0.21)
//!
//! One of two codecs: the first's value where it reads, else the second's, else whichever has a
//! partial value, else both errors.

use crate::com::mojang::datafixers::util::either::Either;
use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::{Encoder, Valued};

#[derive(Clone)]
pub struct EitherCodec<F, S> {
    first: F,
    second: S,
}

impl<F, S> EitherCodec<F, S> {
    pub fn new(first: F, second: S) -> EitherCodec<F, S> {
        EitherCodec { first, second }
    }

    pub fn first(&self) -> &F {
        &self.first
    }

    pub fn second(&self) -> &S {
        &self.second
    }
}

impl<F: Valued, S: Valued> Valued for EitherCodec<F, S> {
    type Value = Either<F::Value, S::Value>;
}

impl<T: OpsValue, F: Decoder<T>, S: Decoder<T>> Decoder<T> for EitherCodec<F, S> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<Self::Value, T>> {
        let first_read = self
            .first
            .decode(ops, input)
            .map(|pair| pair.map_first(Either::Left));
        if first_read.is_success() {
            return first_read;
        }

        let second_read = self
            .second
            .decode(ops, input)
            .map(|pair| pair.map_first(Either::Right));
        if second_read.is_success() {
            second_read
        } else if first_read.has_result_or_partial() {
            first_read
        } else if second_read.has_result_or_partial() {
            second_read
        } else {
            let first = first_read.error_ref().expect("an error").message();
            let second = second_read.error_ref().expect("an error").message();
            let message = format!("Failed to parse either. First: {first}; Second: {second}");
            DataResult::error(|| message)
        }
    }
}

impl<T: OpsValue, F: Encoder<T>, S: Encoder<T>> Encoder<T> for EitherCodec<F, S> {
    fn encode(&self, input: &Self::Value, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        match input {
            Either::Left(value) => self.first.encode(value, ops, prefix),
            Either::Right(value) => self.second.encode(value, ops, prefix),
        }
    }
}
