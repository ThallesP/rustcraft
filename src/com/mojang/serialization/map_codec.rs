//! com/mojang/serialization/MapCodec.java (DataFixerUpper 10.0.21)
//!
//! A map encoder and decoder of one value: the fields a value writes into a map and reads back.
//! Like `Codec`, each map codec is a struct implementing `MapEncoder<T>` and `MapDecoder<T>` for
//! every `OpsValue` `T` (and so `MapCodec<T>`), a `BoxedMapCodec` where one is needed as a value,
//! and Java's methods making map codecs are `MapCodecExt`'s. `codec()` is `MapCodecCodec`, which
//! reads its input as a map and writes into the ops' record builder.
//!
//! `assumeMapUnsafe`, `dependent`, `mapResult`, `orElse` and its kin, `setPartial` and
//! `recursive` aren't ported yet: nothing the text components reach calls them.

use std::sync::Arc;

use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::codec::{BoxedCodec, CodecExt, FieldOf};
use crate::com::mojang::serialization::codecs::key_dispatch_codec::KeyDispatchCodec;
use crate::com::mojang::serialization::codecs::record_codec_builder::RecordCodecBuilder;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::{Encoder, Valued};
use crate::com::mojang::serialization::keyable::Keyable;
use crate::com::mojang::serialization::lifecycle::Lifecycle;
use crate::com::mojang::serialization::map_decoder::MapDecoder;
use crate::com::mojang::serialization::map_encoder::{self, MapEncoder};
use crate::com::mojang::serialization::map_like::MapLike;
use crate::com::mojang::serialization::record_builder::RecordBuilder;
use crate::net::minecraft::nbt::tag::Tag;

pub trait MapCodec<T: OpsValue>: MapEncoder<T> + MapDecoder<T> {}

impl<T: OpsValue, M: MapEncoder<T> + MapDecoder<T> + ?Sized> MapCodec<T> for M {}

/// A map codec held once for each kind of data, like `BoxedCodec`.
pub struct BoxedMapCodec<A> {
    nbt: Arc<dyn MapCodec<Tag, Value = A> + Send + Sync>,
}

impl<A> Clone for BoxedMapCodec<A> {
    fn clone(&self) -> Self {
        BoxedMapCodec {
            nbt: self.nbt.clone(),
        }
    }
}

impl<A> BoxedMapCodec<A> {
    pub fn new<M>(codec: M) -> BoxedMapCodec<A>
    where
        M: MapCodec<Tag, Value = A> + Send + Sync + 'static,
    {
        let codec = Arc::new(codec);
        BoxedMapCodec { nbt: codec }
    }

    /// The map codec, for NBT.
    pub fn nbt(&self) -> &(dyn MapCodec<Tag, Value = A> + Send + Sync) {
        &*self.nbt
    }
}

impl<A> Valued for BoxedMapCodec<A> {
    type Value = A;
}

impl<A, T: OpsValue> Keyable<T> for BoxedMapCodec<A> {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        T::boxed_map_codec(self).keys(ops)
    }
}

impl<A, T: OpsValue> MapEncoder<T> for BoxedMapCodec<A> {
    fn encode(&self, input: &A, ops: &dyn DynamicOps<T>, prefix: &mut dyn RecordBuilder<T>) {
        T::boxed_map_codec(self).encode(input, ops, prefix)
    }
}

impl<A, T: OpsValue> MapDecoder<T> for BoxedMapCodec<A> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<A> {
        T::boxed_map_codec(self).decode(ops, input)
    }
}

/// Java's methods that make map codecs out of a map codec.
pub trait MapCodecExt: MapCodec<Tag> + Sized {
    /// `codec()`: the map codec as a codec of a map.
    fn codec(self) -> MapCodecCodec<Self> {
        MapCodecCodec { codec: self }
    }

    /// `forGetter`: a record's field, read from the record with the getter.
    fn for_getter<O, G>(self, getter: G) -> RecordCodecBuilder<Self, G>
    where
        G: Fn(&O) -> Self::Value,
    {
        RecordCodecBuilder::of(getter, self)
    }

    /// `fieldOf`: `codec().fieldOf(name)`, the map under the name.
    fn field_of(self, name: &str) -> FieldOf<MapCodecCodec<Self>> {
        CodecExt::field_of(self.codec(), name)
    }

    /// `xmap`: `of(comap(from), map(to))`.
    fn xmap<S, F, G>(self, to: F, from: G) -> MapXmap<Self, F, G>
    where
        F: Fn(Self::Value) -> S,
        G: Fn(&S) -> Self::Value,
    {
        MapXmap {
            codec: self,
            to,
            from,
        }
    }

    /// `flatXmap`: `Codec.of(flatComap(from), flatMap(to))`.
    fn flat_xmap<S, F, G>(self, to: F, from: G) -> MapFlatXmap<Self, F, G>
    where
        F: Fn(Self::Value) -> DataResult<S>,
        G: Fn(&S) -> DataResult<Self::Value>,
    {
        MapFlatXmap {
            codec: self,
            to,
            from,
        }
    }

    fn with_lifecycle(self, lifecycle: Lifecycle) -> MapWithLifecycle<Self> {
        MapWithLifecycle {
            codec: self,
            lifecycle,
        }
    }

    fn stable(self) -> MapWithLifecycle<Self> {
        self.with_lifecycle(Lifecycle::stable())
    }

    /// `dispatch`: `partialDispatch` of functions that can't fail.
    fn dispatch<E, F, G>(self, r#type: F, codec: G) -> MapCodecCodec<KeyDispatchCodec<Self, E>>
    where
        Self: Send + Sync + 'static,
        Self::Value: 'static,
        E: 'static,
        F: Fn(&E) -> Self::Value + Send + Sync + 'static,
        G: Fn(&Self::Value) -> BoxedMapCodec<E> + Send + Sync + 'static,
    {
        self.dispatch_map(r#type, codec).codec()
    }

    /// `partialDispatch`.
    fn partial_dispatch<E, F, G>(
        self,
        r#type: F,
        codec: G,
    ) -> MapCodecCodec<KeyDispatchCodec<Self, E>>
    where
        Self: Send + Sync + 'static,
        Self::Value: 'static,
        E: 'static,
        F: Fn(&E) -> DataResult<Self::Value> + Send + Sync + 'static,
        G: Fn(&Self::Value) -> DataResult<BoxedMapCodec<E>> + Send + Sync + 'static,
    {
        KeyDispatchCodec::new(self, r#type, codec).codec()
    }

    /// `dispatchMap`: the dispatch as a map codec.
    fn dispatch_map<E, F, G>(self, r#type: F, codec: G) -> KeyDispatchCodec<Self, E>
    where
        Self: Send + Sync + 'static,
        Self::Value: 'static,
        E: 'static,
        F: Fn(&E) -> Self::Value + Send + Sync + 'static,
        G: Fn(&Self::Value) -> BoxedMapCodec<E> + Send + Sync + 'static,
    {
        KeyDispatchCodec::new(
            self,
            move |value: &E| DataResult::success(r#type(value)),
            move |key: &Self::Value| DataResult::success(codec(key)),
        )
    }

    /// `boxed()`: this map codec as a `BoxedMapCodec`.
    fn boxed(self) -> BoxedMapCodec<Self::Value>
    where
        Self: Send + Sync + 'static,
    {
        BoxedMapCodec::new(self)
    }
}

impl<M: MapCodec<Tag> + Sized> MapCodecExt for M {}

/// `MapCodec.MapCodecCodec`: the input read as a map (stable) and decoded, with the input as the
/// rest; written into the ops' record builder, then built on the prefix.
#[derive(Clone)]
pub struct MapCodecCodec<M> {
    codec: M,
}

impl<M> MapCodecCodec<M> {
    pub fn codec(&self) -> &M {
        &self.codec
    }
}

impl<M: Valued> Valued for MapCodecCodec<M> {
    type Value = M::Value;
}

impl<T: OpsValue, M: MapDecoder<T>> Decoder<T> for MapCodecCodec<M> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<M::Value, T>> {
        self.codec
            .compressed_decode(ops, input)
            .map(|result| Pair::new(result, input.clone()))
    }
}

impl<T: OpsValue, M: MapEncoder<T>> Encoder<T> for MapCodecCodec<M> {
    fn encode(&self, input: &M::Value, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        let mut builder = map_encoder::compressed_builder(ops);
        self.codec.encode(input, ops, &mut *builder);
        builder.build(prefix)
    }
}

/// `MapCodec.xmap`'s map codec.
#[derive(Clone)]
pub struct MapXmap<M, F, G> {
    codec: M,
    to: F,
    from: G,
}

impl<M: Valued, S, F: Fn(M::Value) -> S, G> Valued for MapXmap<M, F, G> {
    type Value = S;
}

impl<T: OpsValue, M: Keyable<T>, F, G> Keyable<T> for MapXmap<M, F, G> {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        let mut keys = self.codec.keys(ops);
        keys.extend(self.codec.keys(ops));
        keys
    }
}

impl<T, M, S, F, G> MapEncoder<T> for MapXmap<M, F, G>
where
    T: OpsValue,
    M: MapEncoder<T>,
    F: Fn(M::Value) -> S,
    G: Fn(&S) -> M::Value,
{
    fn encode(&self, input: &S, ops: &dyn DynamicOps<T>, prefix: &mut dyn RecordBuilder<T>) {
        self.codec.encode(&(self.from)(input), ops, prefix);
    }
}

impl<T, M, S, F, G> MapDecoder<T> for MapXmap<M, F, G>
where
    T: OpsValue,
    M: MapDecoder<T>,
    F: Fn(M::Value) -> S,
{
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<S> {
        self.codec.decode(ops, input).map(&self.to)
    }
}

/// `MapCodec.flatXmap`'s map codec.
#[derive(Clone)]
pub struct MapFlatXmap<M, F, G> {
    codec: M,
    to: F,
    from: G,
}

impl<M: Valued, S, F: Fn(M::Value) -> DataResult<S>, G> Valued for MapFlatXmap<M, F, G> {
    type Value = S;
}

impl<T: OpsValue, M: Keyable<T>, F, G> Keyable<T> for MapFlatXmap<M, F, G> {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        let mut keys = self.codec.keys(ops);
        keys.extend(self.codec.keys(ops));
        keys
    }
}

/// `flatComap`: the function's errors go into the builder, and its value (or partial one) is
/// written.
impl<T, M, S, F, G> MapEncoder<T> for MapFlatXmap<M, F, G>
where
    T: OpsValue,
    M: MapEncoder<T>,
    F: Fn(M::Value) -> DataResult<S>,
    G: Fn(&S) -> DataResult<M::Value>,
{
    fn encode(&self, input: &S, ops: &dyn DynamicOps<T>, prefix: &mut dyn RecordBuilder<T>) {
        let (value, status) = (self.from)(input).split_status();
        prefix.with_errors_from(status);
        if let Some(value) = value {
            self.codec.encode(&value, ops, prefix);
        }
    }
}

impl<T, M, S, F, G> MapDecoder<T> for MapFlatXmap<M, F, G>
where
    T: OpsValue,
    M: MapDecoder<T>,
    F: Fn(M::Value) -> DataResult<S>,
{
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<S> {
        self.codec.decode(ops, input).flat_map(&self.to)
    }
}

/// `MapCodec.withLifecycle`'s map codec.
#[derive(Clone)]
pub struct MapWithLifecycle<M> {
    codec: M,
    lifecycle: Lifecycle,
}

impl<M: Valued> Valued for MapWithLifecycle<M> {
    type Value = M::Value;
}

impl<T: OpsValue, M: Keyable<T>> Keyable<T> for MapWithLifecycle<M> {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        self.codec.keys(ops)
    }
}

impl<T: OpsValue, M: MapEncoder<T>> MapEncoder<T> for MapWithLifecycle<M> {
    fn encode(&self, input: &M::Value, ops: &dyn DynamicOps<T>, prefix: &mut dyn RecordBuilder<T>) {
        self.codec.encode(input, ops, prefix);
        prefix.set_lifecycle(self.lifecycle);
    }
}

impl<T: OpsValue, M: MapDecoder<T>> MapDecoder<T> for MapWithLifecycle<M> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<M::Value> {
        self.codec.decode(ops, input).set_lifecycle(self.lifecycle)
    }
}

/// `MapCodec.unit`: no fields, always the value.
#[derive(Clone)]
pub struct Unit<A> {
    value: A,
}

/// `MapCodec.unit(A)`.
pub fn unit<A: Clone>(value: A) -> Unit<A> {
    Unit { value }
}

impl<A> Valued for Unit<A> {
    type Value = A;
}

impl<T: OpsValue, A> Keyable<T> for Unit<A> {
    fn keys(&self, _ops: &dyn DynamicOps<T>) -> Vec<T> {
        Vec::new()
    }
}

impl<T: OpsValue, A> MapEncoder<T> for Unit<A> {
    fn encode(&self, _input: &A, _ops: &dyn DynamicOps<T>, _prefix: &mut dyn RecordBuilder<T>) {}
}

impl<T: OpsValue, A: Clone> MapDecoder<T> for Unit<A> {
    fn decode(&self, _ops: &dyn DynamicOps<T>, _input: &dyn MapLike<T>) -> DataResult<A> {
        DataResult::success(self.value.clone())
    }
}

/// `BoxedCodec` of a map codec's `codec()`.
pub fn boxed_codec<M>(codec: M) -> BoxedCodec<M::Value>
where
    M: MapCodec<Tag> + Send + Sync + 'static,
{
    BoxedCodec::new(codec.codec())
}
