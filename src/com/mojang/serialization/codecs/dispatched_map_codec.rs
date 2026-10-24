//! com/mojang/serialization/codecs/DispatchedMapCodec.java (DataFixerUpper 10.0.21)
//!
//! A map whose values' codec depends on their key. Java's value is an `ImmutableMap` copied from
//! the `Object2ObjectArrayMap` it reads into, which keeps the entries' order; here it's the entries
//! in that order. Decoding goes on past an entry that fails or repeats a key, keeping what it read.

use std::fmt;

use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::codec::BoxedCodec;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::{Encoder, Valued};
use crate::com::mojang::serialization::lifecycle::Lifecycle;

#[derive(Clone)]
pub struct DispatchedMapCodec<K, F> {
    key_codec: K,
    value_codec_function: F,
}

impl<K, F> DispatchedMapCodec<K, F> {
    pub fn new(key_codec: K, value_codec_function: F) -> DispatchedMapCodec<K, F> {
        DispatchedMapCodec {
            key_codec,
            value_codec_function,
        }
    }
}

impl<K: Valued, V, F: Fn(&K::Value) -> BoxedCodec<V>> Valued for DispatchedMapCodec<K, F> {
    type Value = Vec<(K::Value, V)>;
}

impl<T, K, V, F> Decoder<T> for DispatchedMapCodec<K, F>
where
    T: OpsValue + fmt::Display,
    K: Decoder<T>,
    K::Value: Clone + PartialEq + fmt::Display,
    V: Clone,
    F: Fn(&K::Value) -> BoxedCodec<V>,
{
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<Self::Value, T>> {
        ops.get_map(input).flat_map(|map| {
            let mut entries: Vec<(K::Value, V)> = Vec::new();
            let mut failed: Vec<Pair<T, T>> = Vec::new();
            let mut result = DataResult::success_lifecycle((), Lifecycle::stable());
            for pair in map.entries() {
                let key_result = self.key_codec.parse(ops, pair.get_first());
                let value_result = key_result
                    .clone()
                    .map(|key| (self.value_codec_function)(&key))
                    .flat_map(|value_codec| value_codec.parse(ops, pair.get_second()));
                let entry_result = key_result.apply2stable(|key, value| (key, value), value_result);
                if let Some((key, value)) = entry_result.clone().result_or_partial() {
                    if entries.iter().any(|(existing, _)| *existing == key) {
                        failed.push(pair);
                        let message = format!("Duplicate entry for key: '{key}'");
                        result =
                            result.apply2stable(|unit, _: ()| unit, DataResult::error(|| message));
                        continue;
                    }
                    entries.push((key, value));
                }
                if entry_result.is_error() {
                    failed.push(pair);
                }
                result = result.apply2stable(|unit, _| unit, entry_result);
            }
            let pair = Pair::of(entries, input.clone());
            let errors = ops.create_map(failed);
            result
                .map(|_| pair.clone())
                .set_partial(pair)
                .map_error(|error| format!("{error} missed input: {errors}"))
        })
    }
}

impl<T, K, V, F> Encoder<T> for DispatchedMapCodec<K, F>
where
    T: OpsValue,
    K: Encoder<T>,
    F: Fn(&K::Value) -> BoxedCodec<V>,
{
    fn encode(&self, input: &Self::Value, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        let mut builder = ops.map_builder();
        for (key, value) in input {
            builder.add_results(
                self.key_codec.encode_start(ops, key),
                (self.value_codec_function)(key).encode_start(ops, value),
            );
        }
        builder.build(prefix)
    }
}
