//! net/minecraft/util/ExtraCodecs.java
//!
//! The game's own codecs and combinators, as far as text components reach: `VECTOR4F`,
//! `ARGB_COLOR_CODEC`, `POSITIVE_INT`, `UNTRUSTED_URI`, `CHAT_STRING`, `NBT` (`converter(NbtOps.INSTANCE)`, which
//! reads any data as the tag it converts to), `nonEmptyList`, `orCompressed` of map codecs (the
//! normal one, since no ops the game's codecs reach compresses), `idResolverCodec` of a codec of
//! ids, and `LateBoundIdMapper`. The rest comes with its callers.
//!
//! `converter` is `Codec.PASSTHROUGH` mapped through `Dynamic.convert`, which leaves the value as
//! it is when the ops is the one it converts to, and otherwise converts it with its ops'
//! `convertTo`.

use std::sync::LazyLock;

use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::codec::{
    CodecExt, FLOAT, INT, STRING, StaticCodec, Validate,
};
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsKind, OpsValue};
use crate::com::mojang::serialization::encoder::{Encoder, Valued};
use crate::com::mojang::serialization::keyable::Keyable;
use crate::com::mojang::serialization::lifecycle::Lifecycle;
use crate::com::mojang::serialization::map_decoder::MapDecoder;
use crate::com::mojang::serialization::map_encoder::MapEncoder;
use crate::com::mojang::serialization::map_like::MapLike;
use crate::com::mojang::serialization::record_builder::RecordBuilder;
use crate::java::net::uri::URI;
use crate::net::minecraft::nbt::nbt_ops::NbtOps;
use crate::net::minecraft::nbt::tag::Tag;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::string_util::StringUtil;
use crate::net::minecraft::util::util::Util;
use crate::org::joml::vector4f::Vector4f;

pub struct ExtraCodecs;

static VECTOR4F: StaticCodec<Vector4f> = LazyLock::new(|| {
    FLOAT
        .list_of()
        .comap_flat_map(
            |input: Vec<f32>| {
                Util::fixed_size(input, 4).map(|d| Vector4f {
                    x: d[0],
                    y: d[1],
                    z: d[2],
                    w: d[3],
                })
            },
            |vec: &Vector4f| vec![vec.x, vec.y, vec.z, vec.w],
        )
        .boxed()
});

static ARGB_COLOR_CODEC: StaticCodec<i32> = LazyLock::new(|| {
    INT.with_alternative_converted(VECTOR4F.clone(), |v: Vector4f| {
        ARGB::color_from_float(v.w, v.x, v.y, v.z)
    })
    .boxed()
});

static POSITIVE_INT: StaticCodec<i32> = LazyLock::new(|| {
    int_range_with_message(1, i32::MAX, |n| format!("Value must be positive: {n}"))
});

static CHAT_STRING: StaticCodec<String> = LazyLock::new(|| {
    STRING
        .validate(|string| {
            for c in string.encode_utf16() {
                if !StringUtil::is_allowed_chat_character(i32::from(c)) {
                    let c = String::from_utf16_lossy(&[c]);
                    return DataResult::error(|| format!("Disallowed chat character: '{c}'"));
                }
            }
            DataResult::success(string)
        })
        .boxed()
});

static NBT: StaticCodec<Tag> = LazyLock::new(|| NbtConverter.boxed());

// A link from outside: an http or https URI, or the parser's message.
static UNTRUSTED_URI: StaticCodec<URI> = LazyLock::new(|| {
    STRING
        .comap_flat_map(
            |string: String| match Util::parse_and_validate_untrusted_uri(&string) {
                Ok(uri) => DataResult::success(uri),
                Err(error) => DataResult::error(|| error.to_string()),
            },
            URI::to_string,
        )
        .boxed()
});

// `intRangeWithMessage`: an int in the range, or the message for it.
fn int_range_with_message(
    min_inclusive: i32,
    max_inclusive: i32,
    error: fn(i32) -> String,
) -> crate::com::mojang::serialization::codec::BoxedCodec<i32> {
    INT.validate(move |value| {
        if value >= min_inclusive && value <= max_inclusive {
            DataResult::success(value)
        } else {
            DataResult::error(|| error(value))
        }
    })
    .boxed()
}

impl ExtraCodecs {
    pub const VECTOR4F: &'static StaticCodec<Vector4f> = &VECTOR4F;
    pub const ARGB_COLOR_CODEC: &'static StaticCodec<i32> = &ARGB_COLOR_CODEC;
    pub const POSITIVE_INT: &'static StaticCodec<i32> = &POSITIVE_INT;
    pub const UNTRUSTED_URI: &'static StaticCodec<URI> = &UNTRUSTED_URI;
    pub const CHAT_STRING: &'static StaticCodec<String> = &CHAT_STRING;
    pub const NBT: &'static StaticCodec<Tag> = &NBT;

    /// `nonEmptyList`: the list codec, failing on an empty list.
    #[allow(clippy::type_complexity)]
    pub fn non_empty_list<C, E>(list_codec: C) -> Validate<C, fn(Vec<E>) -> DataResult<Vec<E>>>
    where
        C: CodecExt<Value = Vec<E>>,
        E: Clone,
    {
        list_codec.validate(|list: Vec<E>| {
            if list.is_empty() {
                DataResult::error(|| "List must have contents".to_owned())
            } else {
                DataResult::success(list)
            }
        })
    }

    /// `overrideLifecycle` with a stable lifecycle both ways, which a built-in registry's entry
    /// has: a decoded value's result and every encoded one are stable.
    pub fn override_lifecycle_stable<C: CodecExt>(codec: C) -> OverrideLifecycleStable<C> {
        OverrideLifecycleStable { codec }
    }

    /// `orCompressed(MapCodec, MapCodec)`: the normal map codec, or the compressed one for an ops
    /// that compresses maps; the keys are the compressed one's.
    pub fn or_compressed_map<M1, M2>(normal: M1, compressed: M2) -> OrCompressedMap<M1, M2> {
        OrCompressedMap { normal, compressed }
    }

    /// `idResolverCodec(Codec, Function, Function)`: an element by its id.
    #[allow(clippy::type_complexity)]
    pub fn id_resolver_codec<C, E>(
        value: C,
        from_id: impl Fn(&C::Value) -> Option<E> + Send + Sync + 'static,
        to_id: impl Fn(&E) -> Option<C::Value> + Send + Sync + 'static,
    ) -> crate::com::mojang::serialization::codec::BoxedCodec<E>
    where
        C: CodecExt + Send + Sync + 'static,
        C::Value: std::fmt::Display + 'static,
        E: std::fmt::Display + 'static,
    {
        value
            .flat_xmap(
                move |id: C::Value| match from_id(&id) {
                    Some(element) => DataResult::success(element),
                    None => DataResult::error(|| format!("Unknown element id: {id}")),
                },
                move |e: &E| match to_id(e) {
                    Some(id) => DataResult::success(id),
                    None => DataResult::error(|| format!("Element with unknown id: {e}")),
                },
            )
            .boxed()
    }
}

/// `orCompressed(MapCodec, MapCodec)`'s map codec.
#[derive(Clone)]
pub struct OrCompressedMap<M1, M2> {
    normal: M1,
    compressed: M2,
}

impl<M1: Valued, M2> Valued for OrCompressedMap<M1, M2> {
    type Value = M1::Value;
}

impl<T: OpsValue, M1, M2: Keyable<T>> Keyable<T> for OrCompressedMap<M1, M2> {
    fn keys(&self, ops: &dyn DynamicOps<T>) -> Vec<T> {
        self.compressed.keys(ops)
    }
}

impl<T: OpsValue, M1: MapEncoder<T>, M2: MapEncoder<T, Value = M1::Value>> MapEncoder<T>
    for OrCompressedMap<M1, M2>
{
    fn encode(
        &self,
        input: &M1::Value,
        ops: &dyn DynamicOps<T>,
        prefix: &mut dyn RecordBuilder<T>,
    ) {
        if ops.compress_maps() {
            self.compressed.encode(input, ops, prefix)
        } else {
            self.normal.encode(input, ops, prefix)
        }
    }
}

impl<T: OpsValue, M1: MapDecoder<T>, M2: MapDecoder<T, Value = M1::Value>> MapDecoder<T>
    for OrCompressedMap<M1, M2>
{
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &dyn MapLike<T>) -> DataResult<M1::Value> {
        if ops.compress_maps() {
            self.compressed.decode(ops, input)
        } else {
            self.normal.decode(ops, input)
        }
    }
}

/// `converter(NbtOps.INSTANCE)`: `Codec.PASSTHROUGH` read as the tag its value converts to, and a
/// tag written by converting it to the ops' data and merging that into the prefix.
#[derive(Clone, Copy)]
pub struct NbtConverter;

impl Valued for NbtConverter {
    type Value = Tag;
}

impl<T: OpsValue> Decoder<T> for NbtConverter {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<Tag, T>> {
        DataResult::success(Pair::new(
            convert(ops, &NbtOps::INSTANCE, input),
            ops.empty(),
        ))
    }
}

/// `PASSTHROUGH.encode` of the tag as a `Dynamic` of NBT.
impl<T: OpsValue> Encoder<T> for NbtConverter {
    fn encode(&self, input: &Tag, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        if matches!(input, Tag::End(_)) {
            return DataResult::success_lifecycle(prefix, Lifecycle::experimental());
        }
        passthrough_encode(convert(&NbtOps::INSTANCE, ops, input), ops, prefix)
    }
}

/// The rest of `Codec.PASSTHROUGH.encode` once the value is converted (`casted`): the value itself
/// onto an empty prefix, else merged in as a map, else as a list, else an error.
pub fn passthrough_encode<T: OpsValue>(
    casted: T,
    ops: &dyn DynamicOps<T>,
    prefix: T,
) -> DataResult<T> {
    if prefix == ops.empty() {
        return DataResult::success_lifecycle(casted, Lifecycle::experimental());
    }

    let to_map = ops
        .get_map(&casted)
        .flat_map(|map| ops.merge_to_map_like(&prefix, &*map));
    if let Some(merged) = to_map.into_result() {
        return DataResult::success(merged);
    }
    let to_list = ops
        .get_stream(&casted)
        .flat_map(|stream| ops.merge_to_list_values(&prefix, stream));
    if let Some(merged) = to_list.into_result() {
        return DataResult::success(merged);
    }
    let message = format!("Don't know how to merge {prefix} and {casted}");
    DataResult::error_partial_lifecycle(|| message, prefix, Lifecycle::experimental())
}

/// `Dynamic.convert(inOps, outOps, input)`: the value itself when the two ops are one, else
/// `inOps.convertTo(outOps, input)`, which for a `RegistryOps` leaves the value as it is too when
/// it's converted to its delegate (`DelegatingOps.convertTo`).
pub fn convert<T: OpsValue, U: OpsValue>(
    in_ops: &dyn DynamicOps<T>,
    out_ops: &dyn DynamicOps<U>,
    input: &T,
) -> U {
    let same = in_ops.kind() == out_ops.kind() && in_ops.kind() != OpsKind::Registry;
    let to_delegate = in_ops.kind() == OpsKind::Registry
        && out_ops.kind() != OpsKind::Registry
        && in_ops.delegate_kind() == out_ops.kind();
    if same || to_delegate {
        // The same ops, so the same kind of value.
        let input: &dyn std::any::Any = input;
        if let Some(same) = input.downcast_ref::<U>() {
            return same.clone();
        }
    }
    T::convert_to(out_ops, input)
}

/// `ExtraCodecs.LateBoundIdMapper`: values by id, put after the codec's made. Java's `HashBiMap`
/// iterates in the order its entries were put, which `values()` keeps.
pub struct LateBoundIdMapper<I, V> {
    id_to_value: Vec<(I, V)>,
}

impl<I: Clone + PartialEq, V: Clone + PartialEq> LateBoundIdMapper<I, V> {
    pub fn new() -> LateBoundIdMapper<I, V> {
        LateBoundIdMapper {
            id_to_value: Vec::new(),
        }
    }

    pub fn put(&mut self, id: I, value: V) -> &mut LateBoundIdMapper<I, V> {
        match self.id_to_value.iter_mut().find(|(key, _)| *key == id) {
            Some(entry) => entry.1 = value,
            None => self.id_to_value.push((id, value)),
        }
        self
    }

    pub fn values(&self) -> Vec<V> {
        self.id_to_value
            .iter()
            .map(|(_, value)| value.clone())
            .collect()
    }

    pub fn get(&self, id: &I) -> Option<V> {
        self.id_to_value
            .iter()
            .find(|(key, _)| key == id)
            .map(|(_, value)| value.clone())
    }

    pub fn get_id(&self, value: &V) -> Option<I> {
        self.id_to_value
            .iter()
            .find(|(_, candidate)| candidate == value)
            .map(|(id, _)| id.clone())
    }
}

impl<I: Clone + PartialEq, V: Clone + PartialEq> Default for LateBoundIdMapper<I, V> {
    fn default() -> Self {
        LateBoundIdMapper::new()
    }
}

/// `overrideLifecycle(codec, stable)`'s codec (`mapResult`).
#[derive(Clone)]
pub struct OverrideLifecycleStable<C> {
    codec: C,
}

impl<C: Valued> Valued for OverrideLifecycleStable<C> {
    type Value = C::Value;
}

impl<T: OpsValue, C: Decoder<T>> Decoder<T> for OverrideLifecycleStable<C> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<C::Value, T>> {
        let result = self.codec.decode(ops, input);
        if result.is_success() {
            result.set_lifecycle(Lifecycle::stable())
        } else {
            result
        }
    }
}

impl<T: OpsValue, C: Encoder<T>> Encoder<T> for OverrideLifecycleStable<C> {
    fn encode(&self, input: &C::Value, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        self.codec
            .encode(input, ops, prefix)
            .set_lifecycle(Lifecycle::stable())
    }
}
