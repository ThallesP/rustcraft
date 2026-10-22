//! net/minecraft/network/codec/ByteBufCodecs.java
//!
//! Not ported yet:
//! - `BIT_SET` and `fixedBitSet`, which wait for `java.util.BitSet`;
//! - the NBT codecs (`TAG`, `TRUSTED_TAG`, `COMPOUND_TAG`, `TRUSTED_COMPOUND_TAG`,
//!   `OPTIONAL_COMPOUND_TAG`, `tagCodec`, `optionalTagCodec` and `compoundTagCodec`), which wait
//!   for NBT;
//! - the ones made from DataFixerUpper's codecs (`fromCodec`, `fromCodecTrusted` and the
//!   `fromCodecWithRegistries` ones), and `lenientJson`, which waits for Gson;
//! - `INSTANT`, which waits for `java.time.Instant`, and `PUBLIC_KEY`, which waits for `Crypt`;
//! - the registries' (`registry`, `holderRegistry`, `holder`, `holderSet` and `idMapper(IdMap)`);
//! - `either`, which waits for DataFixerUpper's `Either`, `map`, whose first caller settles which
//!   Rust map stands for its Java map (whose order `encode` writes in), and `lengthPrefixed` and
//!   `registryFriendlyLengthPrefixed`, which wait for `ByteBuf.slice`.
//!
//! An interface Java only uses for its constants and static methods, so a unit struct like `Mth`.
//! Its codecs are statics, which its constants refer to. Java throws Netty's `DecoderException`
//! and `EncoderException`, which are panics here.

use std::sync::LazyLock;

use crate::com::google::common::collect::immutable_multimap::ImmutableMultimap;
use crate::com::mojang::authlib::game_profile::GameProfile;
use crate::com::mojang::authlib::properties::property::Property;
use crate::com::mojang::authlib::properties::property_map::PropertyMap;
use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::util::array_list::ArrayList;
use crate::net::minecraft::core::uuid_util::UUIDUtil;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::utf8_string::Utf8String;
use crate::net::minecraft::network::var_int::VarInt;
use crate::net::minecraft::network::var_long::VarLong;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;
use crate::org::joml::quaternionf::Quaternionf;
use crate::org::joml::vector3f::Vector3f;

pub struct ByteBufCodecs;

static BOOL: StaticStreamCodec<ByteBuf, bool> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &bool| {
            output.write_boolean(*value);
        },
        |input: &mut ByteBuf| input.read_boolean(),
    ))
});

static BYTE: StaticStreamCodec<ByteBuf, i8> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &i8| {
            output.write_byte(i32::from(*value));
        },
        |input: &mut ByteBuf| input.read_byte(),
    ))
});

static ROTATION_BYTE: StaticStreamCodec<ByteBuf, f32> = LazyLock::new(|| {
    Box::new(ByteBufCodecs::BYTE.map(Mth::unpack_degrees, |angle: &f32| Mth::pack_degrees(*angle)))
});

static SHORT: StaticStreamCodec<ByteBuf, i16> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &i16| {
            output.write_short(i32::from(*value));
        },
        |input: &mut ByteBuf| input.read_short(),
    ))
});

static UNSIGNED_SHORT: StaticStreamCodec<ByteBuf, i32> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &i32| {
            output.write_short(*value);
        },
        |input: &mut ByteBuf| input.read_unsigned_short(),
    ))
});

static INT: StaticStreamCodec<ByteBuf, i32> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &i32| {
            output.write_int(*value);
        },
        |input: &mut ByteBuf| input.read_int(),
    ))
});

static VAR_INT: StaticStreamCodec<ByteBuf, i32> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &i32| {
            VarInt::write(output, *value);
        },
        VarInt::read,
    ))
});

// `OptionalInt`: 0 for empty, otherwise the int plus one, which wraps.
static OPTIONAL_VAR_INT: StaticStreamCodec<ByteBuf, Option<i32>> = LazyLock::new(|| {
    Box::new(ByteBufCodecs::VAR_INT.map(
        |i| {
            if i == 0 {
                None
            } else {
                Some(i.wrapping_sub(1))
            }
        },
        |o: &Option<i32>| o.map_or(0, |i| i.wrapping_add(1)),
    ))
});

static LONG: StaticStreamCodec<ByteBuf, i64> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &i64| {
            output.write_long(*value);
        },
        |input: &mut ByteBuf| input.read_long(),
    ))
});

static VAR_LONG: StaticStreamCodec<ByteBuf, i64> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &i64| {
            VarLong::write(output, *value);
        },
        VarLong::read,
    ))
});

static FLOAT: StaticStreamCodec<ByteBuf, f32> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &f32| {
            output.write_float(*value);
        },
        |input: &mut ByteBuf| input.read_float(),
    ))
});

static DOUBLE: StaticStreamCodec<ByteBuf, f64> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &f64| {
            output.write_double(*value);
        },
        |input: &mut ByteBuf| input.read_double(),
    ))
});

static BYTE_ARRAY: StaticStreamCodec<ByteBuf, Vec<i8>> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &Vec<i8>| FriendlyByteBuf::write_byte_array_to(output, value),
        FriendlyByteBuf::read_byte_array_from,
    ))
});

static LONG_ARRAY: StaticStreamCodec<ByteBuf, Vec<i64>> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &Vec<i64>| {
            FriendlyByteBuf::write_long_array_to(output, value)
        },
        FriendlyByteBuf::read_long_array_from,
    ))
});

static STRING_UTF8: StaticStreamCodec<ByteBuf, String> =
    LazyLock::new(|| Box::new(ByteBufCodecs::string_utf8(32767)));

static VECTOR3F: StaticStreamCodec<ByteBuf, Vector3f> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        FriendlyByteBuf::write_vector3f_to,
        FriendlyByteBuf::read_vector3f_from,
    ))
});

static QUATERNIONF: StaticStreamCodec<ByteBuf, Quaternionf> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        FriendlyByteBuf::write_quaternion_to,
        FriendlyByteBuf::read_quaternion_from,
    ))
});

static CONTAINER_ID: StaticStreamCodec<ByteBuf, i32> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &i32| FriendlyByteBuf::write_container_id_to(output, *value),
        FriendlyByteBuf::read_container_id_from,
    ))
});

// At most 16 properties, each its name, value and signature, if any. Reading keeps them in an
// `ImmutableMultimap` by name, which groups them by name in the order each name first came, so
// they're written back in that order.
static GAME_PROFILE_PROPERTIES: StaticStreamCodec<ByteBuf, PropertyMap> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, properties: &PropertyMap| {
            ByteBufCodecs::write_count(output, properties.size(), 16);

            for property in properties.values() {
                Utf8String::write(output, property.name(), 64);
                Utf8String::write(output, property.value(), 32767);
                FriendlyByteBuf::write_nullable_to(
                    output,
                    property.signature(),
                    |out: &mut ByteBuf, signature: &str| Utf8String::write(out, signature, 1024),
                );
            }
        },
        |input: &mut ByteBuf| {
            let property_count = ByteBufCodecs::read_count(input, 16);
            let mut result = ImmutableMultimap::builder();

            for _ in 0..property_count {
                let name = Utf8String::read(input, 64);
                let value = Utf8String::read(input, 32767);
                let signature =
                    FriendlyByteBuf::read_nullable_from(input, |input: &mut ByteBuf| {
                        Utf8String::read(input, 1024)
                    });
                let property = Property::from_signature(name, value, signature);
                result.put(property.name().to_owned(), property);
            }

            PropertyMap::new(result.build())
        },
    ))
});

static PLAYER_NAME: StaticStreamCodec<ByteBuf, String> =
    LazyLock::new(|| Box::new(ByteBufCodecs::string_utf8(16)));

// The id, the name, then the properties.
static GAME_PROFILE: StaticStreamCodec<ByteBuf, GameProfile> = LazyLock::new(|| {
    Box::new(stream_codec::composite3(
        UUIDUtil::STREAM_CODEC,
        GameProfile::id,
        ByteBufCodecs::PLAYER_NAME,
        |profile: &GameProfile| profile.name().to_owned(),
        ByteBufCodecs::GAME_PROFILE_PROPERTIES,
        |profile: &GameProfile| profile.properties().clone(),
        GameProfile::from_properties,
    ))
});

// Three bytes, red first. Writing drops the alpha, and reading makes it opaque.
static RGB_COLOR: StaticStreamCodec<ByteBuf, i32> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &i32| {
            output.write_byte(ARGB::red(*value));
            output.write_byte(ARGB::green(*value));
            output.write_byte(ARGB::blue(*value));
        },
        |input: &mut ByteBuf| {
            let red = i32::from(input.read_byte()) & 0xFF;
            let green = i32::from(input.read_byte()) & 0xFF;
            let blue = i32::from(input.read_byte()) & 0xFF;
            ARGB::color_rgb(red, green, blue)
        },
    ))
});

impl ByteBufCodecs {
    pub const MAX_INITIAL_COLLECTION_SIZE: i32 = 65536;
    pub const BOOL: &'static StaticStreamCodec<ByteBuf, bool> = &BOOL;
    pub const BYTE: &'static StaticStreamCodec<ByteBuf, i8> = &BYTE;
    /// A rotation in degrees, in 256ths of a turn.
    pub const ROTATION_BYTE: &'static StaticStreamCodec<ByteBuf, f32> = &ROTATION_BYTE;
    pub const SHORT: &'static StaticStreamCodec<ByteBuf, i16> = &SHORT;
    pub const UNSIGNED_SHORT: &'static StaticStreamCodec<ByteBuf, i32> = &UNSIGNED_SHORT;
    pub const INT: &'static StaticStreamCodec<ByteBuf, i32> = &INT;
    pub const VAR_INT: &'static StaticStreamCodec<ByteBuf, i32> = &VAR_INT;
    pub const OPTIONAL_VAR_INT: &'static StaticStreamCodec<ByteBuf, Option<i32>> =
        &OPTIONAL_VAR_INT;
    pub const LONG: &'static StaticStreamCodec<ByteBuf, i64> = &LONG;
    pub const VAR_LONG: &'static StaticStreamCodec<ByteBuf, i64> = &VAR_LONG;
    pub const FLOAT: &'static StaticStreamCodec<ByteBuf, f32> = &FLOAT;
    pub const DOUBLE: &'static StaticStreamCodec<ByteBuf, f64> = &DOUBLE;
    pub const BYTE_ARRAY: &'static StaticStreamCodec<ByteBuf, Vec<i8>> = &BYTE_ARRAY;
    pub const LONG_ARRAY: &'static StaticStreamCodec<ByteBuf, Vec<i64>> = &LONG_ARRAY;
    pub const STRING_UTF8: &'static StaticStreamCodec<ByteBuf, String> = &STRING_UTF8;
    pub const VECTOR3F: &'static StaticStreamCodec<ByteBuf, Vector3f> = &VECTOR3F;
    pub const QUATERNIONF: &'static StaticStreamCodec<ByteBuf, Quaternionf> = &QUATERNIONF;
    pub const CONTAINER_ID: &'static StaticStreamCodec<ByteBuf, i32> = &CONTAINER_ID;
    pub const GAME_PROFILE_PROPERTIES: &'static StaticStreamCodec<ByteBuf, PropertyMap> =
        &GAME_PROFILE_PROPERTIES;
    pub const PLAYER_NAME: &'static StaticStreamCodec<ByteBuf, String> = &PLAYER_NAME;
    pub const GAME_PROFILE: &'static StaticStreamCodec<ByteBuf, GameProfile> = &GAME_PROFILE;
    pub const RGB_COLOR: &'static StaticStreamCodec<ByteBuf, i32> = &RGB_COLOR;
    pub const PUBLIC_KEY_SIZE: i32 = 256;
    pub const MAX_PUBLIC_KEY_HEADER_SIZE: i32 = 256;
    pub const MAX_PUBLIC_KEY_LENGTH: i32 = 512;

    /// `byteArray(int)`: a length-prefixed array of at most `max_size` bytes.
    pub fn byte_array(max_size: i32) -> impl StreamCodec<Buffer = ByteBuf, Value = Vec<i8>> {
        stream_codec::of(
            move |output: &mut ByteBuf, value: &Vec<i8>| {
                assert!(
                    value.len() as i32 <= max_size,
                    "ByteArray with size {} is bigger than allowed {max_size}",
                    value.len()
                );
                FriendlyByteBuf::write_byte_array_to(output, value);
            },
            move |input: &mut ByteBuf| FriendlyByteBuf::read_byte_array_from_max(input, max_size),
        )
    }

    /// `stringUtf8(int)`: a string of at most `max_string_length` UTF-16 units.
    pub fn string_utf8(
        max_string_length: i32,
    ) -> impl StreamCodec<Buffer = ByteBuf, Value = String> {
        stream_codec::of(
            move |output: &mut ByteBuf, value: &String| {
                Utf8String::write(output, value, max_string_length);
            },
            move |input: &mut ByteBuf| Utf8String::read(input, max_string_length),
        )
    }

    /// `optional(StreamCodec)`: a boolean, then the value if it's true.
    pub fn optional<C: StreamCodec>(
        original: C,
    ) -> impl StreamCodec<Buffer = C::Buffer, Value = Option<C::Value>>
    where
        C::Buffer: AsMut<ByteBuf>,
    {
        struct Optional<C> {
            original: C,
        }

        impl<C: StreamCodec> StreamCodec for Optional<C>
        where
            C::Buffer: AsMut<ByteBuf>,
        {
            type Buffer = C::Buffer;
            type Value = Option<C::Value>;

            fn decode(&self, input: &mut C::Buffer) -> Option<C::Value> {
                if input.as_mut().read_boolean() {
                    Some(self.original.decode(input))
                } else {
                    None
                }
            }

            fn encode(&self, output: &mut C::Buffer, value: &Option<C::Value>) {
                match value {
                    Some(value) => {
                        output.as_mut().write_boolean(true);
                        self.original.encode(output, value);
                    }
                    None => {
                        output.as_mut().write_boolean(false);
                    }
                }
            }
        }

        Optional { original }
    }

    /// `readCount(ByteBuf, int)`: a VarInt of at most `max_size`, though it may be negative.
    pub fn read_count(input: &mut ByteBuf, max_size: i32) -> i32 {
        let count = VarInt::read(input);
        assert!(
            count <= max_size,
            "{count} elements exceeded max size of: {max_size}"
        );
        count
    }

    /// `writeCount(ByteBuf, int, int)`.
    pub fn write_count(output: &mut ByteBuf, count: i32, max_size: i32) {
        assert!(
            count <= max_size,
            "{count} elements exceeded max size of: {max_size}"
        );

        VarInt::write(output, count);
    }

    /// `collection(IntFunction, StreamCodec)`: any number of elements.
    pub fn collection<C, F>(constructor: F, element_codec: C) -> Collection<C, F> {
        ByteBufCodecs::collection_max(constructor, element_codec, i32::MAX)
    }

    /// `collection(IntFunction, StreamCodec, int)`: a count, then the elements. The constructor
    /// gets the count up to `MAX_INITIAL_COLLECTION_SIZE`, so a long one can't make it allocate
    /// much before the elements run out.
    pub fn collection_max<C, F>(
        constructor: F,
        element_codec: C,
        max_size: i32,
    ) -> Collection<C, F> {
        Collection {
            constructor,
            element_codec,
            max_size,
        }
    }

    /// `collection(IntFunction)`, which makes a collection of any codec it's applied to.
    pub fn collection_operation<C, F>(constructor: F) -> impl FnOnce(C) -> Collection<C, F> {
        move |original| ByteBufCodecs::collection(constructor, original)
    }

    /// `collection(IntFunction, int)`.
    pub fn collection_operation_max<C, F>(
        constructor: F,
        max_size: i32,
    ) -> impl FnOnce(C) -> Collection<C, F> {
        move |original| ByteBufCodecs::collection_max(constructor, original, max_size)
    }

    /// `list()`: a collection in an `ArrayList`.
    pub fn list<C: StreamCodec>() -> impl FnOnce(C) -> List<C> {
        |original| ByteBufCodecs::collection(ArrayList::new as fn(i32) -> Vec<C::Value>, original)
    }

    /// `list(int)`.
    pub fn list_max<C: StreamCodec>(max_size: i32) -> impl FnOnce(C) -> List<C> {
        move |original| {
            ByteBufCodecs::collection_max(
                ArrayList::new as fn(i32) -> Vec<C::Value>,
                original,
                max_size,
            )
        }
    }

    /// `fixedSizeCollection(IntFunction, StreamCodec, int)`: `size` elements, without a count.
    pub fn fixed_size_collection<C, F>(
        constructor: F,
        element_codec: C,
        size: i32,
    ) -> FixedSizeCollection<C, F> {
        FixedSizeCollection {
            constructor,
            element_codec,
            size,
        }
    }

    /// `fixedSizeCollection(IntFunction, int)`.
    pub fn fixed_size_collection_operation<C, F>(
        constructor: F,
        size: i32,
    ) -> impl FnOnce(C) -> FixedSizeCollection<C, F> {
        move |original| ByteBufCodecs::fixed_size_collection(constructor, original, size)
    }

    /// `fixedSizeList(int)`.
    pub fn fixed_size_list<C: StreamCodec>(size: i32) -> impl FnOnce(C) -> FixedSizeList<C> {
        move |original| {
            ByteBufCodecs::fixed_size_collection(
                ArrayList::new as fn(i32) -> Vec<C::Value>,
                original,
                size,
            )
        }
    }

    /// `idMapper(IntFunction, ToIntFunction)`: a value by its id, in a VarInt.
    pub fn id_mapper<T>(
        by_id: impl Fn(i32) -> T,
        to_id: impl Fn(&T) -> i32,
    ) -> impl StreamCodec<Buffer = ByteBuf, Value = T> {
        stream_codec::of(
            move |output: &mut ByteBuf, value: &T| {
                let id = to_id(value);
                VarInt::write(output, id);
            },
            move |input: &mut ByteBuf| {
                let id = VarInt::read(input);
                by_id(id)
            },
        )
    }
}

/// What `collection` makes: Java's collection is anything with `add`, `size` and an iterator,
/// which is a Rust collection that can be extended and iterated over by reference.
pub struct Collection<C, F> {
    constructor: F,
    element_codec: C,
    max_size: i32,
}

impl<C, F, V> StreamCodec for Collection<C, F>
where
    C: StreamCodec,
    C::Buffer: AsMut<ByteBuf>,
    F: Fn(i32) -> V,
    V: Extend<C::Value>,
    for<'a> &'a V: IntoIterator<Item = &'a C::Value, IntoIter: ExactSizeIterator>,
{
    type Buffer = C::Buffer;
    type Value = V;

    fn decode(&self, input: &mut C::Buffer) -> V {
        let count = ByteBufCodecs::read_count(input.as_mut(), self.max_size);
        let mut result = (self.constructor)(count.min(65536));

        for _ in 0..count {
            result.extend([self.element_codec.decode(input)]);
        }

        result
    }

    fn encode(&self, output: &mut C::Buffer, value: &V) {
        let elements = value.into_iter();
        ByteBufCodecs::write_count(output.as_mut(), elements.len() as i32, self.max_size);

        for element in elements {
            self.element_codec.encode(output, element);
        }
    }
}

/// What `list` makes: a collection in an `ArrayList`.
pub type List<C> = Collection<C, fn(i32) -> Vec<<C as StreamCodec>::Value>>;

/// What `fixedSizeCollection` makes.
pub struct FixedSizeCollection<C, F> {
    constructor: F,
    element_codec: C,
    size: i32,
}

/// What `fixedSizeList` makes.
pub type FixedSizeList<C> = FixedSizeCollection<C, fn(i32) -> Vec<<C as StreamCodec>::Value>>;

impl<C, F, V> StreamCodec for FixedSizeCollection<C, F>
where
    C: StreamCodec,
    F: Fn(i32) -> V,
    V: Extend<C::Value>,
    for<'a> &'a V: IntoIterator<Item = &'a C::Value, IntoIter: ExactSizeIterator>,
{
    type Buffer = C::Buffer;
    type Value = V;

    fn decode(&self, input: &mut C::Buffer) -> V {
        let mut result = (self.constructor)(self.size);

        for _ in 0..self.size {
            result.extend([self.element_codec.decode(input)]);
        }

        result
    }

    fn encode(&self, output: &mut C::Buffer, value: &V) {
        let elements = value.into_iter();
        assert!(
            elements.len() as i32 == self.size,
            "Invalid list size, expected {}, but got {}",
            self.size,
            elements.len()
        );

        for element in elements {
            self.element_codec.encode(output, element);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ByteBufCodecs;
    use crate::golden::Golden;
    use crate::io::netty::buffer::byte_buf::ByteBuf;
    use crate::java::util::array_list::ArrayList;
    use crate::net::minecraft::core::direction::Direction;
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;

    const CODEC: &str = "Lnet/minecraft/network/codec/StreamCodec;";
    const OPERATION: &str = "Lnet/minecraft/network/codec/StreamCodec$CodecOperation;";

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.codec.ByteBufCodecs");
        golden.pending("BIT_SET", "waits for java.util.BitSet");
        golden.pending(
            "fixedBitSet(I)Lnet/minecraft/network/codec/StreamCodec;",
            "waits for java.util.BitSet",
        );
        for name in [
            "TAG",
            "TRUSTED_TAG",
            "COMPOUND_TAG",
            "TRUSTED_COMPOUND_TAG",
            "OPTIONAL_COMPOUND_TAG",
            "tagCodec(Ljava/util/function/Supplier;)Lnet/minecraft/network/codec/StreamCodec;",
            "optionalTagCodec(Ljava/util/function/Supplier;)Lnet/minecraft/network/codec/StreamCodec;",
            "compoundTagCodec(Ljava/util/function/Supplier;)Lnet/minecraft/network/codec/StreamCodec;",
        ] {
            golden.pending(name, "waits for NBT");
        }
        for name in [
            "fromCodec(Lcom/mojang/serialization/Codec;)Lnet/minecraft/network/codec/StreamCodec;",
            "fromCodec(Lcom/mojang/serialization/Codec;Ljava/util/function/Supplier;)Lnet/minecraft/network/codec/StreamCodec;",
            "fromCodec(Lcom/mojang/serialization/DynamicOps;Lcom/mojang/serialization/Codec;)Lnet/minecraft/network/codec/StreamCodec$CodecOperation;",
            "fromCodecTrusted(Lcom/mojang/serialization/Codec;)Lnet/minecraft/network/codec/StreamCodec;",
            "fromCodecWithRegistries(Lcom/mojang/serialization/Codec;)Lnet/minecraft/network/codec/StreamCodec;",
            "fromCodecWithRegistries(Lcom/mojang/serialization/Codec;Ljava/util/function/Supplier;)Lnet/minecraft/network/codec/StreamCodec;",
            "fromCodecWithRegistriesTrusted(Lcom/mojang/serialization/Codec;)Lnet/minecraft/network/codec/StreamCodec;",
        ] {
            golden.pending(name, "waits for DataFixerUpper's codecs");
        }
        golden.pending(
            "lenientJson(I)Lnet/minecraft/network/codec/StreamCodec;",
            "waits for Gson",
        );
        golden.pending("INSTANT", "waits for java.time.Instant");
        golden.pending("PUBLIC_KEY", "waits for Crypt");
        for name in [
            "registry(Lnet/minecraft/resources/ResourceKey;)Lnet/minecraft/network/codec/StreamCodec;",
            "holderRegistry(Lnet/minecraft/resources/ResourceKey;)Lnet/minecraft/network/codec/StreamCodec;",
            "holder(Lnet/minecraft/resources/ResourceKey;Lnet/minecraft/network/codec/StreamCodec;)Lnet/minecraft/network/codec/StreamCodec;",
            "holderSet(Lnet/minecraft/resources/ResourceKey;)Lnet/minecraft/network/codec/StreamCodec;",
            "idMapper(Lnet/minecraft/core/IdMap;)Lnet/minecraft/network/codec/StreamCodec;",
        ] {
            golden.pending(name, "waits for the registries");
        }
        golden.pending(
            "either(Lnet/minecraft/network/codec/StreamCodec;Lnet/minecraft/network/codec/StreamCodec;)Lnet/minecraft/network/codec/StreamCodec;",
            "waits for DataFixerUpper's Either",
        );
        for name in [
            "map(Ljava/util/function/IntFunction;Lnet/minecraft/network/codec/StreamCodec;Lnet/minecraft/network/codec/StreamCodec;)Lnet/minecraft/network/codec/StreamCodec;",
            "map(Ljava/util/function/IntFunction;Lnet/minecraft/network/codec/StreamCodec;Lnet/minecraft/network/codec/StreamCodec;I)Lnet/minecraft/network/codec/StreamCodec;",
        ] {
            golden.pending(
                name,
                "its first caller settles which Rust map stands for its Java map, whose order encode writes in",
            );
        }
        for name in [
            "lengthPrefixed(I)Lnet/minecraft/network/codec/StreamCodec$CodecOperation;",
            "lengthPrefixed(ILjava/util/function/BiFunction;)Lnet/minecraft/network/codec/StreamCodec$CodecOperation;",
            "registryFriendlyLengthPrefixed(I)Lnet/minecraft/network/codec/StreamCodec$CodecOperation;",
        ] {
            golden.pending(name, "waits for ByteBuf.slice");
        }
        golden.field(
            "MAX_INITIAL_COLLECTION_SIZE",
            ByteBufCodecs::MAX_INITIAL_COLLECTION_SIZE,
        );
        golden.field("PUBLIC_KEY_SIZE", ByteBufCodecs::PUBLIC_KEY_SIZE);
        golden.field(
            "MAX_PUBLIC_KEY_HEADER_SIZE",
            ByteBufCodecs::MAX_PUBLIC_KEY_HEADER_SIZE,
        );
        golden.field(
            "MAX_PUBLIC_KEY_LENGTH",
            ByteBufCodecs::MAX_PUBLIC_KEY_LENGTH,
        );
        golden.check_codec("BOOL", || ByteBufCodecs::BOOL);
        golden.check_codec("BYTE", || ByteBufCodecs::BYTE);
        golden.check_codec("ROTATION_BYTE", || ByteBufCodecs::ROTATION_BYTE);
        golden.check_codec("SHORT", || ByteBufCodecs::SHORT);
        golden.check_codec("UNSIGNED_SHORT", || ByteBufCodecs::UNSIGNED_SHORT);
        golden.check_codec("INT", || ByteBufCodecs::INT);
        golden.check_codec("VAR_INT", || ByteBufCodecs::VAR_INT);
        golden.check_codec("OPTIONAL_VAR_INT", || ByteBufCodecs::OPTIONAL_VAR_INT);
        golden.check_codec("LONG", || ByteBufCodecs::LONG);
        golden.check_codec("VAR_LONG", || ByteBufCodecs::VAR_LONG);
        golden.check_codec("FLOAT", || ByteBufCodecs::FLOAT);
        golden.check_codec("DOUBLE", || ByteBufCodecs::DOUBLE);
        golden.check_codec("BYTE_ARRAY", || ByteBufCodecs::BYTE_ARRAY);
        golden.check_codec("LONG_ARRAY", || ByteBufCodecs::LONG_ARRAY);
        golden.check_codec("STRING_UTF8", || ByteBufCodecs::STRING_UTF8);
        golden.check_codec("VECTOR3F", || ByteBufCodecs::VECTOR3F);
        golden.check_codec("QUATERNIONF", || ByteBufCodecs::QUATERNIONF);
        golden.check_codec("CONTAINER_ID", || ByteBufCodecs::CONTAINER_ID);
        golden.check_codec("GAME_PROFILE_PROPERTIES", || {
            ByteBufCodecs::GAME_PROFILE_PROPERTIES
        });
        golden.check_codec("PLAYER_NAME", || ByteBufCodecs::PLAYER_NAME);
        golden.check_codec("GAME_PROFILE", || ByteBufCodecs::GAME_PROFILE);
        golden.check_codec("RGB_COLOR", || ByteBufCodecs::RGB_COLOR);
        golden.check_codec_from(&format!("byteArray(I){CODEC}"), ByteBufCodecs::byte_array);
        golden.check_codec_from(&format!("stringUtf8(I){CODEC}"), ByteBufCodecs::string_utf8);
        golden.check_mut(
            "readCount(Lio/netty/buffer/ByteBuf;I)I",
            |input: &mut ByteBuf, max_size: &mut i32| ByteBufCodecs::read_count(input, *max_size),
        );
        golden.check_mut(
            "writeCount(Lio/netty/buffer/ByteBuf;II)V",
            |output: &mut ByteBuf, count: &mut i32, max_size: &mut i32| {
                ByteBufCodecs::write_count(output, *count, *max_size)
            },
        );
        golden.check_codec(&format!("optional({CODEC}){CODEC}[VAR_INT]"), || {
            ByteBufCodecs::optional(ByteBufCodecs::VAR_INT)
        });
        golden.check_codec(&format!("optional({CODEC}){CODEC}[STRING_UTF8]"), || {
            ByteBufCodecs::optional(ByteBufCodecs::STRING_UTF8)
        });
        golden.check_codec(
            &format!("collection(Ljava/util/function/IntFunction;{CODEC}){CODEC}[ArrayList::new, VAR_INT]"),
            || ByteBufCodecs::collection(ArrayList::new, ByteBufCodecs::VAR_INT),
        );
        golden.check_codec_from(
            &format!("collection(Ljava/util/function/IntFunction;{CODEC}I){CODEC}[ArrayList::new, STRING_UTF8]"),
            |max_size| ByteBufCodecs::collection_max(ArrayList::new, ByteBufCodecs::STRING_UTF8, max_size),
        );
        golden.check_codec(
            &format!(
                "collection(Ljava/util/function/IntFunction;){OPERATION}[ArrayList::new, VAR_INT]"
            ),
            || ByteBufCodecs::VAR_INT.apply(ByteBufCodecs::collection_operation(ArrayList::new)),
        );
        golden.check_codec_from(
            &format!(
                "collection(Ljava/util/function/IntFunction;I){OPERATION}[ArrayList::new, VAR_INT]"
            ),
            |max_size| {
                ByteBufCodecs::VAR_INT.apply(ByteBufCodecs::collection_operation_max(
                    ArrayList::new,
                    max_size,
                ))
            },
        );
        golden.check_codec(
            &format!("list(){OPERATION}[Direction.STREAM_CODEC]"),
            || Direction::STREAM_CODEC.apply(ByteBufCodecs::list()),
        );
        golden.check_codec_from(&format!("list(I){OPERATION}[STRING_UTF8]"), |max_size| {
            ByteBufCodecs::STRING_UTF8.apply(ByteBufCodecs::list_max(max_size))
        });
        golden.check_codec_from(
            &format!("fixedSizeCollection(Ljava/util/function/IntFunction;{CODEC}I){CODEC}[ArrayList::new, VAR_INT]"),
            |size| ByteBufCodecs::fixed_size_collection(ArrayList::new, ByteBufCodecs::VAR_INT, size),
        );
        golden.check_codec_from(
            &format!("fixedSizeCollection(Ljava/util/function/IntFunction;I){OPERATION}[ArrayList::new, FLOAT]"),
            |size| {
                ByteBufCodecs::FLOAT.apply(ByteBufCodecs::fixed_size_collection_operation(
                    ArrayList::new,
                    size,
                ))
            },
        );
        golden.check_codec_from(&format!("fixedSizeList(I){OPERATION}[BOOL]"), |size| {
            ByteBufCodecs::BOOL.apply(ByteBufCodecs::fixed_size_list(size))
        });
        golden.check_codec(
            &format!("idMapper(Ljava/util/function/IntFunction;Ljava/util/function/ToIntFunction;){CODEC}[Direction::from3DDataValue, Direction::get3DDataValue]"),
            || ByteBufCodecs::id_mapper(Direction::from_3d_data_value, Direction::get_3d_data_value),
        );
        golden.finish();
    }
}
