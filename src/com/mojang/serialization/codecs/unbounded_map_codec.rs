//! com/mojang/serialization/codecs/UnboundedMapCodec.java (DataFixerUpper 10.0.21), with
//! `BaseMapCodec`'s `decode` and `encode`, its only implementation here.
//!
//! A map of any keys to values of one codec. Java's value is an `ImmutableMap` copied from the
//! `Object2ObjectArrayMap` it reads into, which keeps the entries' order; here it's the entries in
//! that order. Decoding goes on past an entry that fails or repeats a key, keeping what it read.

use std::fmt;

use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::{Encoder, Valued};
use crate::com::mojang::serialization::lifecycle::Lifecycle;

#[derive(Clone)]
pub struct UnboundedMapCodec<K, V> {
    key_codec: K,
    element_codec: V,
}

impl<K, V> UnboundedMapCodec<K, V> {
    pub fn new(key_codec: K, element_codec: V) -> UnboundedMapCodec<K, V> {
        UnboundedMapCodec {
            key_codec,
            element_codec,
        }
    }
}

impl<K: Valued, V: Valued> Valued for UnboundedMapCodec<K, V> {
    type Value = Vec<(K::Value, V::Value)>;
}

impl<T, K, V> Decoder<T> for UnboundedMapCodec<K, V>
where
    T: OpsValue + fmt::Display,
    K: Decoder<T>,
    V: Decoder<T>,
    K::Value: Clone + PartialEq + fmt::Display,
    V::Value: Clone,
{
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<Self::Value, T>> {
        ops.get_map(input)
            .set_lifecycle(Lifecycle::stable())
            .flat_map(|map| {
                let mut read: Vec<(K::Value, V::Value)> = Vec::new();
                let mut failed: Vec<Pair<T, T>> = Vec::new();
                let mut result = DataResult::success_lifecycle((), Lifecycle::stable());
                for pair in map.entries() {
                    let key = self.key_codec.parse(ops, pair.get_first());
                    let value = self.element_codec.parse(ops, pair.get_second());
                    let entry_result = key.apply2stable(|key, value| (key, value), value);
                    if let Some((key, value)) = entry_result.clone().result_or_partial() {
                        if read.iter().any(|(existing, _)| *existing == key) {
                            failed.push(pair);
                            let message = format!("Duplicate entry for key: '{key}'");
                            result = result
                                .apply2stable(|unit, _: ()| unit, DataResult::error(|| message));
                            continue;
                        }
                        read.push((key, value));
                    }
                    if entry_result.is_error() {
                        failed.push(pair);
                    }
                    result = result.apply2stable(|unit, _| unit, entry_result);
                }
                let errors = ops.create_map(failed);
                result
                    .map(|_| read.clone())
                    .set_partial(read)
                    .map_error(|error| format!("{error} missed input: {errors}"))
            })
            .map(|map| Pair::of(map, input.clone()))
    }
}

impl<T: OpsValue, K: Encoder<T>, V: Encoder<T>> Encoder<T> for UnboundedMapCodec<K, V> {
    fn encode(&self, input: &Self::Value, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        let mut builder = ops.map_builder();
        for (key, value) in input {
            builder.add_results(
                self.key_codec.encode_start(ops, key),
                self.element_codec.encode_start(ops, value),
            );
        }
        builder.build(prefix)
    }
}
