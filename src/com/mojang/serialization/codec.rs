//! com/mojang/serialization/Codec.java (DataFixerUpper 10.0.21)
//!
//! An encoder and a decoder of one value. Java's methods are generic in the data's type, which a
//! Rust trait object can't be, so each codec is a struct implementing `Encoder<T>` and
//! `Decoder<T>` for every `OpsValue` `T` (and so `Codec<T>`), and one that a dispatch, a
//! recursive codec or a static needs as a value is a `BoxedCodec`, which holds it once for each
//! kind of data. Java's default methods making codecs (`xmap`, `fieldOf`, `listOf`…) are
//! `CodecExt`'s, implemented for every codec of NBT and JSON, which every codec is.
//!
//! The primitive codecs are unit structs (`BOOL`, `INT`, `STRING`…). `PASSTHROUGH`, `EMPTY`,
//! `pair`, `xor`, `compoundList`, `simpleMap`, `dispatchedMap`, `mapResult`, `orElse` and its
//! kin, `promotePartial` and `string(min, max)` aren't ported yet: nothing the text
//! components reach calls them. Codecs' `toString` isn't either: no error message prints one.

use std::sync::{Arc, OnceLock, Weak};

use crate::com::google::gson::json_element::JsonElement;
use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::codecs::either_codec::EitherCodec;
use crate::com::mojang::serialization::codecs::field_decoder;
use crate::com::mojang::serialization::codecs::field_encoder;
use crate::com::mojang::serialization::codecs::key_dispatch_codec::KeyDispatchCodec;
use crate::com::mojang::serialization::codecs::list_codec::ListCodec;
use crate::com::mojang::serialization::codecs::optional_field_codec::OptionalFieldCodec;
use crate::com::mojang::serialization::codecs::primitive_codec::{self, PrimitiveCodec};
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::{Encoder, Valued};
use crate::com::mojang::serialization::keyable::Keyable;
use crate::com::mojang::serialization::lifecycle::Lifecycle;
use crate::com::mojang::serialization::map_codec::{BoxedMapCodec, MapCodecExt};
use crate::com::mojang::serialization::map_decoder::MapDecoder;
use crate::com::mojang::serialization::map_encoder::MapEncoder;
use crate::com::mojang::serialization::map_like::MapLike;
use crate::com::mojang::serialization::record_builder::RecordBuilder;
use crate::net::minecraft::nbt::tag::Tag;

pub trait Codec<T: OpsValue>: Encoder<T> + Decoder<T> {}

impl<T: OpsValue, C: Encoder<T> + Decoder<T> + ?Sized> Codec<T> for C {}

macro_rules! primitive {
    ($name:ident, $constant:ident, $value:ty, $display:literal,
     |$read_ops:ident, $input:ident| $read:expr,
     |$write_ops:ident, $written:ident| $write:expr) => {
        #[doc = concat!("`Codec.", stringify!($constant), "`.")]
        #[derive(Clone, Copy, Debug)]
        pub struct $name;

        pub const $constant: $name = $name;

        impl Valued for $name {
            type Value = $value;
        }

        impl<T: OpsValue> PrimitiveCodec<T> for $name {
            fn read(&self, $read_ops: &dyn DynamicOps<T>, $input: &T) -> DataResult<$value> {
                $read
            }

            fn write(&self, $write_ops: &dyn DynamicOps<T>, $written: &$value) -> T {
                $write
            }
        }

        impl<T: OpsValue> Encoder<T> for $name {
            fn encode(&self, input: &$value, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
                primitive_codec::encode(self, input, ops, prefix)
            }
        }

        impl<T: OpsValue> Decoder<T> for $name {
            fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<$value, T>> {
                primitive_codec::decode(self, ops, input)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str($display)
            }
        }
    };
}

primitive!(
    Bool,
    BOOL,
    bool,
    "Bool",
    |ops, input| ops.get_boolean_value(input),
    |ops, value| ops.create_boolean(*value)
);
primitive!(
    Byte,
    BYTE,
    i8,
    "Byte",
    |ops, input| ops.get_number_value(input).map(|n| n.byte_value()),
    |ops, value| ops.create_byte(*value)
);
primitive!(
    Short,
    SHORT,
    i16,
    "Short",
    |ops, input| ops.get_number_value(input).map(|n| n.short_value()),
    |ops, value| ops.create_short(*value)
);
primitive!(
    Int,
    INT,
    i32,
    "Int",
    |ops, input| ops.get_number_value(input).map(|n| n.int_value()),
    |ops, value| ops.create_int(*value)
);
primitive!(
    Long,
    LONG,
    i64,
    "Long",
    |ops, input| ops.get_number_value(input).map(|n| n.long_value()),
    |ops, value| ops.create_long(*value)
);
primitive!(
    Float,
    FLOAT,
    f32,
    "Float",
    |ops, input| ops.get_number_value(input).map(|n| n.float_value()),
    |ops, value| ops.create_float(*value)
);
primitive!(
    Double,
    DOUBLE,
    f64,
    "Double",
    |ops, input| ops.get_number_value(input).map(|n| n.double_value()),
    |ops, value| ops.create_double(*value)
);
primitive!(
    IntStream,
    INT_STREAM,
    Vec<i32>,
    "IntStream",
    |ops, input| ops.get_int_stream(input),
    |ops, value| ops.create_int_list(value)
);
primitive!(
    StringCodec,
    STRING,
    String,
    "String",
    |ops, input| ops.get_string_value(input),
    |ops, value| ops.create_string(value)
);

/// A codec held once for each kind of data, so it can be shared as a value: what a static, a
/// dispatch's choice or a recursive codec holds. Cloning it shares the codec.
pub struct BoxedCodec<A> {
    nbt: Arc<dyn Codec<Tag, Value = A> + Send + Sync>,
    json: Arc<dyn Codec<JsonElement, Value = A> + Send + Sync>,
}

impl<A> Clone for BoxedCodec<A> {
    fn clone(&self) -> Self {
        BoxedCodec {
            nbt: self.nbt.clone(),
            json: self.json.clone(),
        }
    }
}

impl<A> BoxedCodec<A> {
    pub fn new<C>(codec: C) -> BoxedCodec<A>
    where
        C: Codec<Tag, Value = A> + Codec<JsonElement, Value = A> + Send + Sync + 'static,
    {
        let codec = Arc::new(codec);
        BoxedCodec {
            nbt: codec.clone(),
            json: codec,
        }
    }

    /// The codec, for NBT.
    pub fn nbt(&self) -> &(dyn Codec<Tag, Value = A> + Send + Sync) {
        &*self.nbt
    }

    /// The codec, for JSON.
    pub fn json(&self) -> &(dyn Codec<JsonElement, Value = A> + Send + Sync) {
        &*self.json
    }
}

impl<A> Valued for BoxedCodec<A> {
    type Value = A;
}

impl<A, T: OpsValue> Encoder<T> for BoxedCodec<A> {
    fn encode(&self, input: &A, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        T::boxed_codec(self).encode(input, ops, prefix)
    }
}

impl<A, T: OpsValue> Decoder<T> for BoxedCodec<A> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<A, T>> {
        T::boxed_codec(self).decode(ops, input)
    }
}

/// A static codec, made on first use: `Identifier::CODEC`.
pub type StaticCodec<A> = std::sync::LazyLock<BoxedCodec<A>>;

/// A static codec handed out to make other codecs with: `Identifier::CODEC.shared().field_of("id")`.
pub trait SharedCodec<A> {
    fn shared(&self) -> BoxedCodec<A>;
}

impl<A> SharedCodec<A> for StaticCodec<A> {
    fn shared(&self) -> BoxedCodec<A> {
        (**self).clone()
    }
}

/// Java's default methods that make codecs out of a codec.
pub trait CodecExt: Codec<Tag> + Codec<JsonElement> + Sized {
    /// `xmap`: the value mapped both ways.
    fn xmap<S, F, G>(self, to: F, from: G) -> Xmap<Self, F, G>
    where
        F: Fn(Self::Value) -> S,
        G: Fn(&S) -> Self::Value,
    {
        Xmap {
            codec: self,
            to,
            from,
        }
    }

    /// `comapFlatMap`: decoding can fail.
    fn comap_flat_map<S, F, G>(self, to: F, from: G) -> ComapFlatMap<Self, F, G>
    where
        F: Fn(Self::Value) -> DataResult<S>,
        G: Fn(&S) -> Self::Value,
    {
        ComapFlatMap {
            codec: self,
            to,
            from,
        }
    }

    /// `flatComapMap`: encoding can fail.
    fn flat_comap_map<S, F, G>(self, to: F, from: G) -> FlatComapMap<Self, F, G>
    where
        F: Fn(Self::Value) -> S,
        G: Fn(&S) -> DataResult<Self::Value>,
    {
        FlatComapMap {
            codec: self,
            to,
            from,
        }
    }

    /// `flatXmap`: either way can fail.
    fn flat_xmap<S, F, G>(self, to: F, from: G) -> FlatXmap<Self, F, G>
    where
        F: Fn(Self::Value) -> DataResult<S>,
        G: Fn(&S) -> DataResult<Self::Value>,
    {
        FlatXmap {
            codec: self,
            to,
            from,
        }
    }

    /// `validate`: `flatXmap` with the checker both ways.
    fn validate<F>(self, checker: F) -> Validate<Self, F>
    where
        F: Fn(Self::Value) -> DataResult<Self::Value>,
        Self::Value: Clone,
    {
        Validate {
            codec: self,
            checker,
        }
    }

    fn list_of(self) -> ListCodec<Self> {
        ListCodec::new(self, 0, i32::MAX)
    }

    /// `listOf(int, int)`.
    fn list_of_sized(self, min_size: i32, max_size: i32) -> ListCodec<Self> {
        ListCodec::new(self, min_size, max_size)
    }

    fn field_of(self, name: &str) -> FieldOf<Self> {
        FieldOf {
            name: name.to_owned(),
            codec: self,
        }
    }

    fn optional_field_of(self, name: &str) -> OptionalFieldCodec<Self> {
        OptionalFieldCodec::new(name, self, false)
    }

    fn lenient_optional_field_of(self, name: &str) -> OptionalFieldCodec<Self> {
        OptionalFieldCodec::new(name, self, true)
    }

    /// `optionalFieldOf(String, A)`: the default when the field's missing, and no field for it.
    fn optional_field_of_default(
        self,
        name: &str,
        default_value: Self::Value,
    ) -> OptionalDefault<Self>
    where
        Self::Value: Clone + PartialEq,
    {
        optional_default(OptionalFieldCodec::new(name, self, false), default_value)
    }

    /// `lenientOptionalFieldOf(String, A)`.
    fn lenient_optional_field_of_default(
        self,
        name: &str,
        default_value: Self::Value,
    ) -> OptionalDefault<Self>
    where
        Self::Value: Clone + PartialEq,
    {
        optional_default(OptionalFieldCodec::new(name, self, true), default_value)
    }

    fn with_lifecycle(self, lifecycle: Lifecycle) -> WithLifecycle<Self> {
        WithLifecycle {
            codec: self,
            lifecycle,
        }
    }

    fn stable(self) -> WithLifecycle<Self> {
        self.with_lifecycle(Lifecycle::stable())
    }

    /// `withAlternative(Codec)`: this codec, or the alternative where it fails to decode.
    #[allow(clippy::type_complexity)]
    fn with_alternative<C2>(
        self,
        alternative: C2,
    ) -> WithAlternative<Self, C2, fn(Self::Value) -> Self::Value>
    where
        C2: Codec<Tag, Value = Self::Value>,
    {
        WithAlternative {
            codec: EitherCodec::new(self, alternative),
            converter: |value| value,
        }
    }

    /// `withAlternative(Codec, Function)`: the alternative's value converted.
    fn with_alternative_converted<C2, F>(
        self,
        alternative: C2,
        converter: F,
    ) -> WithAlternative<Self, C2, F>
    where
        C2: Codec<Tag>,
        F: Fn(C2::Value) -> Self::Value,
    {
        WithAlternative {
            codec: EitherCodec::new(self, alternative),
            converter,
        }
    }

    /// `dispatch(String, Function, Function)`: the type under the key, and the codec for it.
    fn dispatch<E, F, G>(self, type_key: &str, r#type: F, codec: G) -> KeyDispatchCodecOf<Self, E>
    where
        Self: Send + Sync + 'static,
        Self::Value: 'static,
        E: 'static,
        F: Fn(&E) -> Self::Value + Send + Sync + 'static,
        G: Fn(&Self::Value) -> BoxedMapCodec<E> + Send + Sync + 'static,
    {
        self.field_of(type_key).dispatch(r#type, codec)
    }

    /// `boxed()`: this codec as a `BoxedCodec`.
    fn boxed(self) -> BoxedCodec<Self::Value>
    where
        Self: Send + Sync + 'static,
    {
        BoxedCodec::new(self)
    }
}

impl<C: Codec<Tag> + Codec<JsonElement> + Sized> CodecExt for C {}

/// `dispatch`'s codec.
pub type KeyDispatchCodecOf<C, E> =
    crate::com::mojang::serialization::map_codec::MapCodecCodec<KeyDispatchCodec<FieldOf<C>, E>>;

/// `xmap`'s codec: `of(comap(from), map(to))`.
#[derive(Clone)]
pub struct Xmap<C, F, G> {
    codec: C,
    to: F,
    from: G,
}

impl<C: Valued, S, F: Fn(C::Value) -> S, G> Valued for Xmap<C, F, G> {
    type Value = S;
}

impl<T: OpsValue, C: Codec<T>, S, F: Fn(C::Value) -> S, G: Fn(&S) -> C::Value> Encoder<T>
    for Xmap<C, F, G>
{
    fn encode(&self, input: &S, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        self.codec.encode(&(self.from)(input), ops, prefix)
    }
}

impl<T: OpsValue, C: Codec<T>, S, F: Fn(C::Value) -> S, G: Fn(&S) -> C::Value> Decoder<T>
    for Xmap<C, F, G>
{
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<S, T>> {
        self.codec
            .decode(ops, input)
            .map(|pair| pair.map_first(&self.to))
    }
}

/// `comapFlatMap`'s codec: `of(comap(from), flatMap(to))`.
#[derive(Clone)]
pub struct ComapFlatMap<C, F, G> {
    codec: C,
    to: F,
    from: G,
}

impl<C: Valued, S, F: Fn(C::Value) -> DataResult<S>, G> Valued for ComapFlatMap<C, F, G> {
    type Value = S;
}

impl<T, C, S, F, G> Encoder<T> for ComapFlatMap<C, F, G>
where
    T: OpsValue,
    C: Codec<T>,
    F: Fn(C::Value) -> DataResult<S>,
    G: Fn(&S) -> C::Value,
{
    fn encode(&self, input: &S, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        self.codec.encode(&(self.from)(input), ops, prefix)
    }
}

impl<T, C, S, F, G> Decoder<T> for ComapFlatMap<C, F, G>
where
    T: OpsValue,
    C: Codec<T>,
    F: Fn(C::Value) -> DataResult<S>,
    G: Fn(&S) -> C::Value,
{
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<S, T>> {
        decode_flat_mapped(&self.codec, &self.to, ops, input)
    }
}

/// `flatComapMap`'s codec: `of(flatComap(from), map(to))`.
#[derive(Clone)]
pub struct FlatComapMap<C, F, G> {
    codec: C,
    to: F,
    from: G,
}

impl<C: Valued, S, F: Fn(C::Value) -> S, G> Valued for FlatComapMap<C, F, G> {
    type Value = S;
}

impl<T, C, S, F, G> Encoder<T> for FlatComapMap<C, F, G>
where
    T: OpsValue,
    C: Codec<T>,
    F: Fn(C::Value) -> S,
    G: Fn(&S) -> DataResult<C::Value>,
{
    fn encode(&self, input: &S, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        (self.from)(input).flat_map(|value| self.codec.encode(&value, ops, prefix))
    }
}

impl<T, C, S, F, G> Decoder<T> for FlatComapMap<C, F, G>
where
    T: OpsValue,
    C: Codec<T>,
    F: Fn(C::Value) -> S,
    G: Fn(&S) -> DataResult<C::Value>,
{
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<S, T>> {
        self.codec
            .decode(ops, input)
            .map(|pair| pair.map_first(&self.to))
    }
}

/// `flatXmap`'s codec: `of(flatComap(from), flatMap(to))`.
#[derive(Clone)]
pub struct FlatXmap<C, F, G> {
    codec: C,
    to: F,
    from: G,
}

impl<C: Valued, S, F: Fn(C::Value) -> DataResult<S>, G> Valued for FlatXmap<C, F, G> {
    type Value = S;
}

impl<T, C, S, F, G> Encoder<T> for FlatXmap<C, F, G>
where
    T: OpsValue,
    C: Codec<T>,
    F: Fn(C::Value) -> DataResult<S>,
    G: Fn(&S) -> DataResult<C::Value>,
{
    fn encode(&self, input: &S, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        (self.from)(input).flat_map(|value| self.codec.encode(&value, ops, prefix))
    }
}

impl<T, C, S, F, G> Decoder<T> for FlatXmap<C, F, G>
where
    T: OpsValue,
    C: Codec<T>,
    F: Fn(C::Value) -> DataResult<S>,
    G: Fn(&S) -> DataResult<C::Value>,
{
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<S, T>> {
        decode_flat_mapped(&self.codec, &self.to, ops, input)
    }
}

// `Decoder.flatMap`: the value read, then the function of it, with the rest of the input.
fn decode_flat_mapped<T: OpsValue, C: Decoder<T>, S>(
    codec: &C,
    to: &impl Fn(C::Value) -> DataResult<S>,
    ops: &dyn DynamicOps<T>,
    input: &T,
) -> DataResult<Pair<S, T>> {
    codec.decode(ops, input).flat_map(|pair| {
        let (value, rest) = pair.into_parts();
        to(value).map(|result| Pair::new(result, rest))
    })
}

/// `validate`'s codec: `flatXmap(checker, checker)`.
#[derive(Clone)]
pub struct Validate<C, F> {
    codec: C,
    checker: F,
}

impl<C: Valued, F> Valued for Validate<C, F> {
    type Value = C::Value;
}

impl<T, C, F> Encoder<T> for Validate<C, F>
where
    T: OpsValue,
    C: Codec<T>,
    C::Value: Clone,
    F: Fn(C::Value) -> DataResult<C::Value>,
{
    fn encode(&self, input: &C::Value, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        (self.checker)(input.clone()).flat_map(|value| self.codec.encode(&value, ops, prefix))
    }
}

impl<T, C, F> Decoder<T> for Validate<C, F>
where
    T: OpsValue,
    C: Codec<T>,
    F: Fn(C::Value) -> DataResult<C::Value>,
{
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<C::Value, T>> {
        decode_flat_mapped(&self.codec, &self.checker, ops, input)
    }
}

/// `withAlternative`'s codec: `either(this, alternative).xmap(…, Either::left)`, which writes with
/// this codec and reads the alternative's value converted.
#[derive(Clone)]
pub struct WithAlternative<C1, C2, F> {
    codec: EitherCodec<C1, C2>,
    converter: F,
}

impl<C1: Valued, C2, F> Valued for WithAlternative<C1, C2, F> {
    type Value = C1::Value;
}

impl<T: OpsValue, C1: Encoder<T>, C2, F> Encoder<T> for WithAlternative<C1, C2, F> {
    fn encode(&self, input: &C1::Value, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        self.codec.first().encode(input, ops, prefix)
    }
}

impl<T, C1, C2, F> Decoder<T> for WithAlternative<C1, C2, F>
where
    T: OpsValue,
    C1: Codec<T>,
    C2: Codec<T>,
    F: Fn(C2::Value) -> C1::Value,
{
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<C1::Value, T>> {
        self.codec
            .decode(ops, input)
            .map(|pair| pair.map_first(|either| either.map(|value| value, &self.converter)))
    }
}

/// `withLifecycle`'s codec: each result with the lifecycle.
#[derive(Clone)]
pub struct WithLifecycle<C> {
    codec: C,
    lifecycle: Lifecycle,
}

impl<C: Valued> Valued for WithLifecycle<C> {
    type Value = C::Value;
}

impl<T: OpsValue, C: Encoder<T>> Encoder<T> for WithLifecycle<C> {
    fn encode(&self, input: &C::Value, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        self.codec
            .encode(input, ops, prefix)
            .set_lifecycle(self.lifecycle)
    }
}

impl<T: OpsValue, C: Decoder<T>> Decoder<T> for WithLifecycle<C> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<C::Value, T>> {
        self.codec.decode(ops, input).set_lifecycle(self.lifecycle)
    }
}

/// `fieldOf`'s map codec: `MapCodec.of(FieldEncoder, FieldDecoder)`, the value under the name.
#[derive(Clone)]
pub struct FieldOf<C> {
    name: String,
    codec: C,
}

impl<C: Valued> Valued for FieldOf<C> {
    type Value = C::Value;
}

/// The encoder's key, then the decoder's.
impl<T: OpsValue, C> Keyable<T> for FieldOf<C> {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        vec![ops.create_string(&self.name), ops.create_string(&self.name)]
    }
}

impl<T: OpsValue, C: Encoder<T>> MapEncoder<T> for FieldOf<C> {
    fn encode(&self, input: &C::Value, ops: &dyn DynamicOps<T>, prefix: &mut dyn RecordBuilder<T>) {
        field_encoder::encode(&self.name, &self.codec, input, ops, prefix);
    }
}

impl<T: OpsValue, C: Decoder<T>> MapDecoder<T> for FieldOf<C> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<C::Value> {
        field_decoder::decode(&self.name, &self.codec, ops, input)
    }
}

/// `optionalFieldOf(String, A)`'s map codec: the optional field `xmap`ped to the default, which
/// isn't written.
#[derive(Clone)]
pub struct OptionalDefault<C: Valued> {
    field: OptionalFieldCodec<C>,
    default_value: C::Value,
}

fn optional_default<C: Valued>(
    field: OptionalFieldCodec<C>,
    default_value: C::Value,
) -> OptionalDefault<C> {
    OptionalDefault {
        field,
        default_value,
    }
}

impl<C: Valued> Valued for OptionalDefault<C> {
    type Value = C::Value;
}

/// `MapCodec.of`'s: the encoder's keys, then the decoder's.
impl<T: OpsValue, C: Valued> Keyable<T> for OptionalDefault<C> {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        let mut keys = self.field.keys(ops);
        keys.extend(self.field.keys(ops));
        keys
    }
}

impl<T: OpsValue, C: Encoder<T>> MapEncoder<T> for OptionalDefault<C>
where
    C::Value: Clone + PartialEq,
{
    fn encode(&self, input: &C::Value, ops: &dyn DynamicOps<T>, prefix: &mut dyn RecordBuilder<T>) {
        let value = if *input == self.default_value {
            None
        } else {
            Some(input.clone())
        };
        self.field.encode(&value, ops, prefix);
    }
}

impl<T: OpsValue, C: Decoder<T>> MapDecoder<T> for OptionalDefault<C>
where
    C::Value: Clone,
{
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<C::Value> {
        self.field
            .decode(ops, input)
            .map(|value| value.unwrap_or_else(|| self.default_value.clone()))
    }
}

/// `checkRange`: the value if it's in the range by its `compareTo`, else an error printing it
/// and the bounds with `to_string`.
fn check_range<N: Copy>(
    value: N,
    min_inclusive: N,
    max_inclusive: N,
    compare: fn(N, N) -> std::cmp::Ordering,
    to_string: fn(N) -> String,
) -> DataResult<N> {
    if compare(value, min_inclusive).is_ge() && compare(value, max_inclusive).is_le() {
        DataResult::success(value)
    } else {
        DataResult::error(|| {
            format!(
                "Value {} outside of range [{}:{}]",
                to_string(value),
                to_string(min_inclusive),
                to_string(max_inclusive)
            )
        })
    }
}

/// `Codec.intRange`: `INT` checked both ways.
pub fn int_range(min_inclusive: i32, max_inclusive: i32) -> BoxedCodec<i32> {
    let checker = move |value: i32| {
        check_range(
            value,
            min_inclusive,
            max_inclusive,
            |a, b| a.cmp(&b),
            |n| n.to_string(),
        )
    };
    INT.flat_xmap(checker, move |value: &i32| checker(*value))
        .boxed()
}

/// `Codec.floatRange`: `FLOAT` checked both ways, by `Float.compareTo`.
pub fn float_range(min_inclusive: f32, max_inclusive: f32) -> BoxedCodec<f32> {
    let checker = move |value: f32| {
        check_range(
            value,
            min_inclusive,
            max_inclusive,
            |a, b| crate::java::lang::double::Double::compare(f64::from(a), f64::from(b)),
            crate::java::lang::float::Float::to_string,
        )
    };
    FLOAT
        .flat_xmap(checker, move |value: &f32| checker(*value))
        .boxed()
}

/// `Codec.doubleRange`: `DOUBLE` checked both ways, by `Double.compareTo`.
pub fn double_range(min_inclusive: f64, max_inclusive: f64) -> BoxedCodec<f64> {
    let checker = move |value: f64| {
        check_range(
            value,
            min_inclusive,
            max_inclusive,
            crate::java::lang::double::Double::compare,
            crate::java::lang::double::Double::to_string,
        )
    };
    DOUBLE
        .flat_xmap(checker, move |value: &f64| checker(*value))
        .boxed()
}

/// `RecursiveCodec`: `recursive(name, wrapped)` and `lazyInitialized`, a codec made on first use
/// from a reference to itself.
pub struct RecursiveCodec<A> {
    wrapped: OnceLock<BoxedCodec<A>>,
    #[allow(clippy::type_complexity)]
    make: Box<dyn Fn(RecursiveRef<A>) -> BoxedCodec<A> + Send + Sync>,
    this: Weak<RecursiveCodec<A>>,
}

/// What `recursive`'s function gets: the recursive codec, which it holds without keeping it
/// alive.
pub struct RecursiveRef<A> {
    codec: Weak<RecursiveCodec<A>>,
}

impl<A> Clone for RecursiveRef<A> {
    fn clone(&self) -> Self {
        RecursiveRef {
            codec: self.codec.clone(),
        }
    }
}

/// `Codec.recursive`.
pub fn recursive<A: 'static>(
    wrapped: impl Fn(RecursiveRef<A>) -> BoxedCodec<A> + Send + Sync + 'static,
) -> Arc<RecursiveCodec<A>> {
    Arc::new_cyclic(|this| RecursiveCodec {
        wrapped: OnceLock::new(),
        make: Box::new(wrapped),
        this: this.clone(),
    })
}

impl<A> RecursiveCodec<A> {
    fn wrapped(&self) -> &BoxedCodec<A> {
        self.wrapped.get_or_init(|| {
            (self.make)(RecursiveRef {
                codec: self.this.clone(),
            })
        })
    }
}

impl<A> Valued for RecursiveCodec<A> {
    type Value = A;
}

impl<A, T: OpsValue> Encoder<T> for RecursiveCodec<A> {
    fn encode(&self, input: &A, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        self.wrapped().encode(input, ops, prefix)
    }
}

impl<A, T: OpsValue> Decoder<T> for RecursiveCodec<A> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<A, T>> {
        self.wrapped().decode(ops, input)
    }
}

impl<A> RecursiveRef<A> {
    fn codec(&self) -> Arc<RecursiveCodec<A>> {
        self.codec
            .upgrade()
            .expect("a recursive codec outlives what it makes")
    }
}

impl<A> Valued for RecursiveRef<A> {
    type Value = A;
}

impl<A, T: OpsValue> Encoder<T> for RecursiveRef<A> {
    fn encode(&self, input: &A, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        self.codec().encode(input, ops, prefix)
    }
}

impl<A, T: OpsValue> Decoder<T> for RecursiveRef<A> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<A, T>> {
        self.codec().decode(ops, input)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{BOOL, BYTE, BoxedCodec, Codec, CodecExt, DOUBLE, FLOAT, INT, LONG, SHORT, STRING};
    use crate::com::mojang::datafixers::util::either::Either;
    use crate::com::mojang::serialization::codecs::either_codec::EitherCodec;
    use crate::com::mojang::serialization::codecs::record_codec_builder;
    use crate::com::mojang::serialization::data_result::DataResult;
    use crate::com::mojang::serialization::data_result::tests::encode_result;
    use crate::com::mojang::serialization::map_codec::{BoxedMapCodec, MapCodecExt};
    use crate::golden::{Decode, Encode, Golden};
    use crate::java::lang::integer::Integer;
    use crate::net::minecraft::nbt::nbt_ops::NbtOps;
    use crate::net::minecraft::nbt::tag::Tag;
    use crate::net::minecraft::resources::identifier::Identifier;

    // `N.encode` and `N.decode`, on NBT.
    fn check<C>(golden: &Golden, name: &str, codec: C)
    where
        C: Codec<Tag>,
        C::Value: Decode + Encode + Clone + 'static,
    {
        let ops = &NbtOps::INSTANCE;
        golden.check(&format!("{name}.encode"), |value: C::Value, prefix: Tag| {
            encode_result(&codec.encode(&value, ops, prefix), |tag| {
                Box::new(tag.clone())
            })
        });
        golden.check(&format!("{name}.decode"), |input: Tag| {
            encode_result(&codec.decode(ops, &input), |pair| {
                Box::new((pair.get_first().clone(), pair.get_second().clone()))
            })
        });
    }

    /// The recursive codec's value: an int, or a list of trees.
    #[derive(Clone, Debug, PartialEq)]
    struct Tree(Either<i32, Vec<Tree>>);

    impl Encode for Tree {
        fn encode(&self) -> Value {
            self.0.encode()
        }

        fn matches(&self, expected: &Value) -> bool {
            self.0.matches(expected)
        }
    }

    impl Decode for Tree {
        fn decode(value: &Value) -> Self {
            Tree(Either::decode(value))
        }
    }

    type Dispatched = Either<i32, (String, i32)>;

    // The dispatch suites' two types: an int under "value", and a name with a number.
    fn dispatched_codecs() -> (BoxedMapCodec<Dispatched>, BoxedMapCodec<Dispatched>) {
        let int_value = INT
            .field_of("value")
            .xmap(Either::Left, |e: &Dispatched| *e.left().expect("an int"))
            .boxed();
        let pair = record_codec_builder::map_codec(
            (
                STRING
                    .field_of("name")
                    .for_getter(|r: &(String, i32)| r.0.clone()),
                INT.optional_field_of_default("n", 0)
                    .for_getter(|r: &(String, i32)| r.1),
            ),
            |name, n| (name, n),
        )
        .xmap(Either::Right, |e: &Dispatched| {
            e.right().expect("a pair").clone()
        })
        .boxed();
        (int_value, pair)
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.serialization.Codec");
        check(&golden, "BOOL", BOOL);
        check(&golden, "BYTE", BYTE);
        check(&golden, "SHORT", SHORT);
        check(&golden, "INT", INT);
        check(&golden, "LONG", LONG);
        check(&golden, "FLOAT", FLOAT);
        check(&golden, "DOUBLE", DOUBLE);
        check(&golden, "STRING", STRING);
        check(
            &golden,
            "Identifier.CODEC",
            super::SharedCodec::shared(Identifier::CODEC),
        );
        check(&golden, "INT.listOf()", INT.list_of());
        check(&golden, "STRING.listOf(1, 3)", STRING.list_of_sized(1, 3));
        check(&golden, "INT.listOf().listOf()", INT.list_of().list_of());
        check(
            &golden,
            "either(INT, STRING)",
            EitherCodec::new(INT, STRING),
        );
        check(
            &golden,
            "either(INT.listOf(), BOOL)",
            EitherCodec::new(INT.list_of(), BOOL),
        );
        check(
            &golden,
            "INT.xmap(x * 3, x / 3)",
            INT.xmap(|x| x.wrapping_mul(3), |x: &i32| x / 3),
        );
        check(
            &golden,
            "STRING.comapFlatMap(parseInt, toString)",
            STRING.comap_flat_map(
                |s| match Integer::parse_int(&s) {
                    Ok(value) => DataResult::success(value),
                    Err(_) => DataResult::error(|| format!("Not an int: {s}")),
                },
                |value: &i32| value.to_string(),
            ),
        );
        check(
            &golden,
            "INT.flatXmap(half of even, double of non-negative)",
            INT.flat_xmap(
                |x| {
                    if x % 2 == 0 {
                        DataResult::success(x / 2)
                    } else {
                        DataResult::error_partial(|| format!("Odd: {x}"), x / 2)
                    }
                },
                |&x: &i32| {
                    if x >= 0 {
                        DataResult::success(x.wrapping_mul(2))
                    } else {
                        DataResult::error(|| format!("Negative: {x}"))
                    }
                },
            ),
        );
        check(
            &golden,
            "INT.validate(non-negative)",
            INT.validate(|x| {
                if x >= 0 {
                    DataResult::success(x)
                } else {
                    DataResult::error(|| format!("Negative: {x}"))
                }
            }),
        );
        check(
            &golden,
            "STRING.flatComapMap(s + !, without !)",
            STRING.flat_comap_map(
                |s| s + "!",
                |s: &String| match s.strip_suffix('!') {
                    Some(stripped) => DataResult::success(stripped.to_owned()),
                    None => DataResult::error(|| format!("No !: {s}")),
                },
            ),
        );
        check(
            &golden,
            "withAlternative(INT, STRING, length)",
            INT.with_alternative_converted(STRING, |s: String| s.encode_utf16().count() as i32),
        );
        check(
            &golden,
            "withAlternative(INT, FLOAT.xmap(toInt, toFloat))",
            INT.with_alternative(FLOAT.xmap(|f| f as i32, |&i: &i32| i as f32)),
        );
        check(&golden, "INT.fieldOf(a).codec()", INT.field_of("a").codec());
        check(
            &golden,
            "STRING.optionalFieldOf(b).codec()",
            STRING.optional_field_of("b").codec(),
        );
        check(
            &golden,
            "INT.lenientOptionalFieldOf(c).codec()",
            INT.lenient_optional_field_of("c").codec(),
        );
        check(
            &golden,
            "INT.optionalFieldOf(d, 7).codec()",
            INT.optional_field_of_default("d", 7).codec(),
        );
        check(
            &golden,
            "BOOL.lenientOptionalFieldOf(e, true).codec()",
            BOOL.lenient_optional_field_of_default("e", true).codec(),
        );
        check(
            &golden,
            "INT.listOf().optionalFieldOf(f, []).codec()",
            INT.list_of()
                .optional_field_of_default("f", Vec::new())
                .codec(),
        );
        check(
            &golden,
            "record1",
            record_codec_builder::create((INT.field_of("a").for_getter(|r: &(i32,)| r.0),), |a| {
                (a,)
            }),
        );
        check(
            &golden,
            "record1 optional",
            record_codec_builder::create(
                (STRING
                    .optional_field_of("a")
                    .for_getter(|r: &(Option<String>,)| r.0.clone()),),
                |a| (a,),
            ),
        );
        let record2 = || {
            record_codec_builder::map_codec(
                (
                    INT.field_of("a")
                        .for_getter(|r: &(i32, Option<String>)| r.0),
                    STRING
                        .optional_field_of("b")
                        .for_getter(|r: &(i32, Option<String>)| r.1.clone()),
                ),
                |a, b| (a, b),
            )
        };
        check(&golden, "record2", record2().codec());
        check(
            &golden,
            "record3",
            record_codec_builder::create(
                (
                    STRING
                        .field_of("a")
                        .for_getter(|r: &(String, i32, Option<bool>)| r.0.clone()),
                    INT.optional_field_of_default("b", 0)
                        .for_getter(|r: &(String, i32, Option<bool>)| r.1),
                    BOOL.lenient_optional_field_of("c")
                        .for_getter(|r: &(String, i32, Option<bool>)| r.2),
                ),
                |a, b, c| (a, b, c),
            ),
        );
        type Record4 = (i32, i32, Vec<String>, Either<i32, String>);
        check(
            &golden,
            "record4",
            record_codec_builder::create(
                (
                    INT.field_of("a").for_getter(|r: &Record4| r.0),
                    INT.field_of("b").for_getter(|r: &Record4| r.1),
                    STRING
                        .list_of()
                        .field_of("c")
                        .for_getter(|r: &Record4| r.2.clone()),
                    EitherCodec::new(INT, STRING)
                        .field_of("d")
                        .for_getter(|r: &Record4| r.3.clone()),
                ),
                |a, b, c, d| (a, b, c, d),
            ),
        );
        type Record5 = (i32, String, Option<bool>, Vec<i32>, String);
        check(
            &golden,
            "record5",
            record_codec_builder::create(
                (
                    INT.field_of("a").for_getter(|r: &Record5| r.0),
                    STRING.field_of("b").for_getter(|r: &Record5| r.1.clone()),
                    BOOL.optional_field_of("c").for_getter(|r: &Record5| r.2),
                    INT.list_of()
                        .field_of("d")
                        .for_getter(|r: &Record5| r.3.clone()),
                    STRING
                        .optional_field_of_default("e", "x".to_owned())
                        .for_getter(|r: &Record5| r.4.clone()),
                ),
                |a, b, c, d, e| (a, b, c, d, e),
            ),
        );
        type Record11 = (
            Option<i32>,
            Option<i32>,
            Option<bool>,
            Option<bool>,
            Option<bool>,
            Option<bool>,
            Option<bool>,
            String,
            Option<Vec<i32>>,
            Option<String>,
            String,
        );
        check(
            &golden,
            "record11",
            record_codec_builder::create(
                (
                    INT.optional_field_of("color")
                        .for_getter(|r: &Record11| r.0),
                    INT.optional_field_of("shadow_color")
                        .for_getter(|r: &Record11| r.1),
                    BOOL.optional_field_of("bold")
                        .for_getter(|r: &Record11| r.2),
                    BOOL.optional_field_of("italic")
                        .for_getter(|r: &Record11| r.3),
                    BOOL.optional_field_of("underlined")
                        .for_getter(|r: &Record11| r.4),
                    BOOL.optional_field_of("strikethrough")
                        .for_getter(|r: &Record11| r.5),
                    BOOL.optional_field_of("obfuscated")
                        .for_getter(|r: &Record11| r.6),
                    STRING
                        .field_of("text")
                        .for_getter(|r: &Record11| r.7.clone()),
                    INT.list_of()
                        .optional_field_of("extra")
                        .for_getter(|r: &Record11| r.8.clone()),
                    STRING
                        .optional_field_of("insertion")
                        .for_getter(|r: &Record11| r.9.clone()),
                    STRING
                        .field_of("font")
                        .for_getter(|r: &Record11| r.10.clone()),
                ),
                |a, b, c, d, e, f, g, h, i, j, k| (a, b, c, d, e, f, g, h, i, j, k),
            ),
        );
        type Record16 = (
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            i32,
            String,
            String,
            String,
            String,
            bool,
            bool,
            bool,
            bool,
        );
        check(
            &golden,
            "record16",
            record_codec_builder::create(
                (
                    INT.field_of("a").for_getter(|r: &Record16| r.0),
                    INT.field_of("b").for_getter(|r: &Record16| r.1),
                    INT.field_of("c").for_getter(|r: &Record16| r.2),
                    INT.field_of("d").for_getter(|r: &Record16| r.3),
                    INT.field_of("e").for_getter(|r: &Record16| r.4),
                    INT.field_of("f").for_getter(|r: &Record16| r.5),
                    INT.field_of("g").for_getter(|r: &Record16| r.6),
                    INT.field_of("h").for_getter(|r: &Record16| r.7),
                    STRING.field_of("i").for_getter(|r: &Record16| r.8.clone()),
                    STRING.field_of("j").for_getter(|r: &Record16| r.9.clone()),
                    STRING.field_of("k").for_getter(|r: &Record16| r.10.clone()),
                    STRING.field_of("l").for_getter(|r: &Record16| r.11.clone()),
                    BOOL.field_of("m").for_getter(|r: &Record16| r.12),
                    BOOL.field_of("n").for_getter(|r: &Record16| r.13),
                    BOOL.field_of("o").for_getter(|r: &Record16| r.14),
                    BOOL.field_of("p").for_getter(|r: &Record16| r.15),
                ),
                |a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p| {
                    (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p)
                },
            ),
        );
        check(
            &golden,
            "record2.stable().codec()",
            record2().stable().codec(),
        );
        check(
            &golden,
            "record2.xmap(swap).codec()",
            record2()
                .xmap(
                    |(a, b)| (b, a),
                    |(b, a): &(Option<String>, i32)| (*a, b.clone()),
                )
                .codec(),
        );
        check(
            &golden,
            "record2.flatXmap(a non-negative).codec()",
            record2()
                .flat_xmap(
                    |r| {
                        if r.0 >= 0 {
                            DataResult::success(r)
                        } else {
                            DataResult::error_partial(|| "Negative a".to_owned(), r)
                        }
                    },
                    |r: &(i32, Option<String>)| {
                        if r.0 >= 0 {
                            DataResult::success(r.clone())
                        } else {
                            let a = r.0;
                            DataResult::error_partial(
                                || format!("Negative a: {a}"),
                                (0, r.1.clone()),
                            )
                        }
                    },
                )
                .codec(),
        );

        let (int_value, pair) = dispatched_codecs();
        check(
            &golden,
            "STRING.validate(known).dispatch(type)",
            STRING
                .validate(|s| {
                    if s == "int" || s == "pair" {
                        DataResult::success(s)
                    } else {
                        DataResult::error(|| format!("Unknown type: {s}"))
                    }
                })
                .dispatch(
                    "type",
                    |e: &Dispatched| match e {
                        Either::Left(_) => "int".to_owned(),
                        Either::Right(_) => "pair".to_owned(),
                    },
                    move |t: &String| {
                        if t == "int" {
                            int_value.clone()
                        } else {
                            pair.clone()
                        }
                    },
                ),
        );
        let (int_value, pair) = dispatched_codecs();
        check(
            &golden,
            "STRING.partialDispatch(kind)",
            STRING.field_of("kind").partial_dispatch(
                |e: &Dispatched| match e {
                    Either::Left(l) if *l < 0 => {
                        let l = *l;
                        DataResult::error(move || format!("No kind for {l}"))
                    }
                    Either::Left(_) => DataResult::success("int".to_owned()),
                    Either::Right(_) => DataResult::success("pair".to_owned()),
                },
                move |t: &String| match t.as_str() {
                    "int" => DataResult::success(int_value.clone()),
                    "pair" => DataResult::success(pair.clone()),
                    _ => DataResult::error(|| format!("Unknown kind: {t}")),
                },
            ),
        );
        check(
            &golden,
            "recursive(tree)",
            super::recursive(|this| {
                EitherCodec::new(INT, this.list_of())
                    .xmap(Tree, |tree: &Tree| tree.0.clone())
                    .boxed()
            }),
        );
        let _: Option<BoxedCodec<i32>> = None;
        golden.finish();
    }
}
