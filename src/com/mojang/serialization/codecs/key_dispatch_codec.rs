//! com/mojang/serialization/codecs/KeyDispatchCodec.java (DataFixerUpper 10.0.21)
//!
//! A value of one of several types, written as its type's fields with its type under a key. The
//! map codec of each type is a `BoxedMapCodec` of the dispatched value, which Java's
//! `MapCodec<? extends V>` is with a cast: a type's map codec of an enum's variant maps it in and
//! out of the enum. The compressed form (the value under `"value"`) isn't ported, like every
//! compressing ops. `toString` isn't either.

use std::sync::Arc;

use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::Valued;
use crate::com::mojang::serialization::keyable::Keyable;
use crate::com::mojang::serialization::map_codec::BoxedMapCodec;
use crate::com::mojang::serialization::map_decoder::MapDecoder;
use crate::com::mojang::serialization::map_encoder::MapEncoder;
use crate::com::mojang::serialization::map_like::MapLike;
use crate::com::mojang::serialization::record_builder::RecordBuilder;

/// The type of a value, and the map codec of a type.
type TypeFn<K, V> = Arc<dyn Fn(&V) -> DataResult<K> + Send + Sync>;
type CodecFn<K, V> = Arc<dyn Fn(&K) -> DataResult<BoxedMapCodec<V>> + Send + Sync>;
/// The map codec of a value.
type EncoderFn<V> = Arc<dyn Fn(&V) -> DataResult<BoxedMapCodec<V>> + Send + Sync>;

pub struct KeyDispatchCodec<M: Valued, V> {
    key_codec: M,
    r#type: TypeFn<M::Value, V>,
    decoder: CodecFn<M::Value, V>,
    encoder: EncoderFn<V>,
}

impl<M: Valued + Clone, V> Clone for KeyDispatchCodec<M, V> {
    fn clone(&self) -> Self {
        KeyDispatchCodec {
            key_codec: self.key_codec.clone(),
            r#type: self.r#type.clone(),
            decoder: self.decoder.clone(),
            encoder: self.encoder.clone(),
        }
    }
}

impl<M: Valued, V> KeyDispatchCodec<M, V>
where
    M::Value: 'static,
    V: 'static,
{
    /// The public constructor: a value's encoder is its type's codec (`getCodec`).
    pub fn new(
        key_codec: M,
        r#type: impl Fn(&V) -> DataResult<M::Value> + Send + Sync + 'static,
        codec: impl Fn(&M::Value) -> DataResult<BoxedMapCodec<V>> + Send + Sync + 'static,
    ) -> KeyDispatchCodec<M, V> {
        let r#type: TypeFn<M::Value, V> = Arc::new(r#type);
        let codec: CodecFn<M::Value, V> = Arc::new(codec);
        let (type_of, codec_of) = (r#type.clone(), codec.clone());
        KeyDispatchCodec {
            key_codec,
            r#type,
            decoder: codec,
            encoder: Arc::new(move |value| type_of(value).flat_map(|key| codec_of(&key))),
        }
    }
}

impl<M: Valued, V> Valued for KeyDispatchCodec<M, V> {
    type Value = V;
}

/// The key codec's keys, then `"value"`.
impl<T: OpsValue, M: Valued + Keyable<T>, V> Keyable<T> for KeyDispatchCodec<M, V> {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        let mut keys = self.key_codec.keys(ops);
        keys.push(ops.create_string("value"));
        keys
    }
}

/// The type from under its key, then the type's codec on the same map.
impl<T: OpsValue, M: MapDecoder<T>, V> MapDecoder<T> for KeyDispatchCodec<M, V> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<V> {
        assert!(!ops.compress_maps(), "compressed maps aren't ported");
        self.key_codec.decode(ops, input).flat_map(|r#type| {
            (self.decoder)(&r#type).flat_map(|element_decoder| element_decoder.decode(ops, input))
        })
    }
}

/// The type's fields first, then the type under its key; an error finding either goes into the
/// builder instead.
impl<T: OpsValue, M: MapEncoder<T>, V> MapEncoder<T> for KeyDispatchCodec<M, V> {
    fn encode(&self, input: &V, ops: &dyn DynamicOps<T>, prefix: &mut dyn RecordBuilder<T>) {
        assert!(!ops.compress_maps(), "compressed maps aren't ported");
        let (element_encoder, encoder_status) = (self.encoder)(input).split_status();
        let type_result = (self.r#type)(input);
        let type_is_error = type_result.is_error();
        let (r#type, type_status) = type_result.split_status();
        let encoder_is_error = encoder_status.is_error();
        prefix.with_errors_from(encoder_status);
        prefix.with_errors_from(type_status);
        if !encoder_is_error && !type_is_error {
            let element_encoder = element_encoder.expect("a success has its value");
            let r#type = r#type.expect("a success has its value");
            element_encoder.encode(input, ops, prefix);
            self.key_codec.encode(&r#type, ops, prefix);
        }
    }
}
