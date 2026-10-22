//! net/minecraft/network/FriendlyByteBuf.java
//!
//! Not ported yet:
//! - the codecs' readers and writers (`readWithCodec`, `readWithCodecTrusted`, `writeWithCodec`,
//!   `readLenientJsonWithCodec`, `writeJsonWithCodec`), which wait for DataFixerUpper;
//! - `readEither` and `writeEither`, which wait for DataFixerUpper's `Either`;
//! - `readEnumSet` and `writeEnumSet`, whose `Class` argument the golden harness can't generate
//!   yet, and which need `java.util.BitSet` like the `BitSet` ones (`readBitSet`, `writeBitSet`,
//!   `readFixedBitSet`, `writeFixedBitSet`);
//! - `readChunkPos` and `writeChunkPos`, which wait for `ChunkPos`; `readGlobalPos`,
//!   `writeGlobalPos`, `readResourceKey`, `writeResourceKey` and `readRegistryKey`, which wait
//!   for `ResourceKey`; and `readNbt` and `writeNbt`, which wait for NBT.
//!
//! The methods overriding `ByteBuf`'s forward to the buffer it wraps, which it derefs to. Java
//! throws Netty's `DecoderException` and `EncoderException`, and `NegativeArraySizeException` for
//! a negative array length, which are panics here.
//!
//! The readers and writers taking a `StreamDecoder<? super FriendlyByteBuf, T>` take one of any
//! buffer this one is `AsMut` of, like the codecs (PORTING.md). A value Java lets be null, or
//! passes as an `Optional`, is an `Option`. `readEnum(Class)` takes the enum as a type argument,
//! `read_enum::<Action>()`.

use std::ops::{Deref, DerefMut};

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::r#enum::Enum;
use crate::java::util::uuid::UUID;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::network::codec::stream_decoder::StreamDecoder;
use crate::net::minecraft::network::codec::stream_encoder::StreamEncoder;
use crate::net::minecraft::network::utf8_string::Utf8String;
use crate::net::minecraft::network::var_int::VarInt;
use crate::net::minecraft::network::var_long::VarLong;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::org::joml::quaternionf::Quaternionf;
use crate::org::joml::vector3f::Vector3f;

/// A buffer with readers and writers for the game's types. Java's wraps a shared `ByteBuf`; this
/// one owns it.
#[derive(Clone, Debug)]
pub struct FriendlyByteBuf {
    source: ByteBuf,
}

impl FriendlyByteBuf {
    pub const MAX_STRING_LENGTH: i16 = 32767;
    pub const MAX_COMPONENT_STRING_LENGTH: i32 = 262144;

    pub fn new(source: ByteBuf) -> FriendlyByteBuf {
        FriendlyByteBuf { source }
    }

    /// A VarInt count, then the reader called that many times, or none for a negative count.
    pub fn read_with_count(&mut self, mut reader: impl FnMut(&mut FriendlyByteBuf)) {
        let count = self.read_var_int();

        for _ in 0..count {
            reader(self);
        }
    }

    /// A boolean, then the value if there is one.
    pub fn write_optional<O, T>(
        &mut self,
        value: Option<&T>,
        value_writer: impl StreamEncoder<O, T>,
    ) where
        Self: AsMut<O>,
    {
        if let Some(value) = value {
            self.write_boolean(true);
            value_writer.encode(self.as_mut(), value);
        } else {
            self.write_boolean(false);
        }
    }

    pub fn read_optional<I, T>(&mut self, value_reader: impl StreamDecoder<I, T>) -> Option<T>
    where
        Self: AsMut<I>,
    {
        if self.read_boolean() {
            Some(value_reader.decode(self.as_mut()))
        } else {
            None
        }
    }

    pub fn read_nullable<I, T>(&mut self, value_decoder: impl StreamDecoder<I, T>) -> Option<T>
    where
        Self: AsMut<I>,
    {
        FriendlyByteBuf::read_nullable_from(self, value_decoder)
    }

    /// `readNullable(ByteBuf, StreamDecoder)`: a boolean, then the value if it's true. Java's
    /// `B extends ByteBuf` is a buffer that is one or wraps one.
    pub fn read_nullable_from<B, I, T>(
        input: &mut B,
        value_decoder: impl StreamDecoder<I, T>,
    ) -> Option<T>
    where
        B: AsMut<ByteBuf> + AsMut<I>,
    {
        if AsMut::<ByteBuf>::as_mut(input).read_boolean() {
            Some(value_decoder.decode(input.as_mut()))
        } else {
            None
        }
    }

    pub fn write_nullable<O, T: ?Sized>(
        &mut self,
        value: Option<&T>,
        value_encoder: impl StreamEncoder<O, T>,
    ) where
        Self: AsMut<O>,
    {
        FriendlyByteBuf::write_nullable_to(self, value, value_encoder);
    }

    /// `writeNullable(ByteBuf, Object, StreamEncoder)`: a boolean, then the value if there is one.
    pub fn write_nullable_to<B, O, T: ?Sized>(
        output: &mut B,
        value: Option<&T>,
        value_encoder: impl StreamEncoder<O, T>,
    ) where
        B: AsMut<ByteBuf> + AsMut<O>,
    {
        if let Some(value) = value {
            AsMut::<ByteBuf>::as_mut(output).write_boolean(true);
            value_encoder.encode(output.as_mut(), value);
        } else {
            AsMut::<ByteBuf>::as_mut(output).write_boolean(false);
        }
    }

    /// As many bytes as are readable at most.
    pub fn read_byte_array(&mut self) -> Vec<i8> {
        FriendlyByteBuf::read_byte_array_from(&mut self.source)
    }

    /// `readByteArray(ByteBuf)`.
    pub fn read_byte_array_from(input: &mut ByteBuf) -> Vec<i8> {
        let max_size = input.readable_bytes();
        FriendlyByteBuf::read_byte_array_from_max(input, max_size)
    }

    pub fn write_byte_array(&mut self, bytes: &[i8]) -> &mut FriendlyByteBuf {
        FriendlyByteBuf::write_byte_array_to(&mut self.source, bytes);
        self
    }

    /// `writeByteArray(ByteBuf, byte[])`: the length in a VarInt, then the bytes.
    pub fn write_byte_array_to(output: &mut ByteBuf, bytes: &[i8]) {
        VarInt::write(output, bytes.len() as i32);
        output.write_bytes_array(bytes);
    }

    /// `readByteArray(int)`.
    pub fn read_byte_array_max(&mut self, max_size: i32) -> Vec<i8> {
        FriendlyByteBuf::read_byte_array_from_max(&mut self.source, max_size)
    }

    /// `readByteArray(ByteBuf, int)`: the array is made before its bytes are read, so a length
    /// past what's left throws only then.
    pub fn read_byte_array_from_max(input: &mut ByteBuf, max_size: i32) -> Vec<i8> {
        let size = VarInt::read(input);
        assert!(
            size <= max_size,
            "ByteArray with size {size} is bigger than allowed {max_size}"
        );

        let mut bytes = new_array(size);
        input.read_bytes_array(&mut bytes);
        bytes
    }

    pub fn write_var_int_array(&mut self, ints: &[i32]) -> &mut FriendlyByteBuf {
        self.write_var_int(ints.len() as i32);

        for &i in ints {
            self.write_var_int(i);
        }

        self
    }

    /// As many VarInts as there are bytes readable at most.
    pub fn read_var_int_array(&mut self) -> Vec<i32> {
        let max_size = self.readable_bytes();
        self.read_var_int_array_max(max_size)
    }

    /// `readVarIntArray(int)`.
    pub fn read_var_int_array_max(&mut self, max_size: i32) -> Vec<i32> {
        let size = self.read_var_int();
        assert!(
            size <= max_size,
            "VarIntArray with size {size} is bigger than allowed {max_size}"
        );

        let mut ints = new_array(size);

        for int in &mut ints {
            *int = self.read_var_int();
        }

        ints
    }

    pub fn write_long_array(&mut self, longs: &[i64]) -> &mut FriendlyByteBuf {
        FriendlyByteBuf::write_long_array_to(&mut self.source, longs);
        self
    }

    /// `writeLongArray(ByteBuf, long[])`: the length in a VarInt, then the longs.
    pub fn write_long_array_to(output: &mut ByteBuf, longs: &[i64]) {
        VarInt::write(output, longs.len() as i32);
        FriendlyByteBuf::write_fixed_size_long_array_to(output, longs);
    }

    pub fn write_fixed_size_long_array(&mut self, longs: &[i64]) -> &mut FriendlyByteBuf {
        FriendlyByteBuf::write_fixed_size_long_array_to(&mut self.source, longs);
        self
    }

    /// `writeFixedSizeLongArray(ByteBuf, long[])`.
    pub fn write_fixed_size_long_array_to(output: &mut ByteBuf, longs: &[i64]) {
        for &l in longs {
            output.write_long(l);
        }
    }

    pub fn read_long_array(&mut self) -> Vec<i64> {
        FriendlyByteBuf::read_long_array_from(&mut self.source)
    }

    pub fn read_fixed_size_long_array<'a>(&mut self, output: &'a mut [i64]) -> &'a mut [i64] {
        FriendlyByteBuf::read_fixed_size_long_array_from(&mut self.source, output)
    }

    /// `readLongArray(ByteBuf)`: as many longs as are readable at most.
    pub fn read_long_array_from(input: &mut ByteBuf) -> Vec<i64> {
        let size = VarInt::read(input);
        let max_size = input.readable_bytes() / 8;
        assert!(
            size <= max_size,
            "LongArray with size {size} is bigger than allowed {max_size}"
        );
        let mut output = new_array(size);
        FriendlyByteBuf::read_fixed_size_long_array_from(input, &mut output);
        output
    }

    /// `readFixedSizeLongArray(ByteBuf, long[])`: fills the array, so a buffer that runs out
    /// leaves it partly filled.
    pub fn read_fixed_size_long_array_from<'a>(
        input: &mut ByteBuf,
        output: &'a mut [i64],
    ) -> &'a mut [i64] {
        for long in output.iter_mut() {
            *long = input.read_long();
        }

        output
    }

    pub fn read_block_pos(&mut self) -> BlockPos {
        FriendlyByteBuf::read_block_pos_from(&mut self.source)
    }

    /// `readBlockPos(ByteBuf)`: packed in a long.
    pub fn read_block_pos_from(input: &mut ByteBuf) -> BlockPos {
        BlockPos::of(input.read_long())
    }

    pub fn write_block_pos(&mut self, pos: &BlockPos) -> &mut FriendlyByteBuf {
        FriendlyByteBuf::write_block_pos_to(&mut self.source, pos);
        self
    }

    /// `writeBlockPos(ByteBuf, BlockPos)`.
    pub fn write_block_pos_to(output: &mut ByteBuf, pos: &BlockPos) {
        output.write_long(pos.as_long());
    }

    pub fn read_vector3f(&mut self) -> Vector3f {
        FriendlyByteBuf::read_vector3f_from(&mut self.source)
    }

    /// `readVector3f(ByteBuf)`.
    pub fn read_vector3f_from(input: &mut ByteBuf) -> Vector3f {
        let x = input.read_float();
        let y = input.read_float();
        let z = input.read_float();
        Vector3f::from_xyz(x, y, z)
    }

    pub fn write_vector3f(&mut self, v: &Vector3f) {
        FriendlyByteBuf::write_vector3f_to(&mut self.source, v);
    }

    /// `writeVector3f(ByteBuf, Vector3fc)`.
    pub fn write_vector3f_to(output: &mut ByteBuf, v: &Vector3f) {
        output.write_float(v.x);
        output.write_float(v.y);
        output.write_float(v.z);
    }

    pub fn read_quaternion(&mut self) -> Quaternionf {
        FriendlyByteBuf::read_quaternion_from(&mut self.source)
    }

    /// `readQuaternion(ByteBuf)`.
    pub fn read_quaternion_from(input: &mut ByteBuf) -> Quaternionf {
        let x = input.read_float();
        let y = input.read_float();
        let z = input.read_float();
        let w = input.read_float();
        Quaternionf::from_xyzw(x, y, z, w)
    }

    pub fn write_quaternion(&mut self, q: &Quaternionf) {
        FriendlyByteBuf::write_quaternion_to(&mut self.source, q);
    }

    /// `writeQuaternion(ByteBuf, Quaternionfc)`.
    pub fn write_quaternion_to(output: &mut ByteBuf, value: &Quaternionf) {
        output.write_float(value.x);
        output.write_float(value.y);
        output.write_float(value.z);
        output.write_float(value.w);
    }

    /// A VarInt ordinal. Panics on one past the constants, or negative, where Java throws
    /// `ArrayIndexOutOfBoundsException`.
    pub fn read_enum<T: Enum>(&mut self) -> T {
        let ordinal = self.read_var_int();
        *usize::try_from(ordinal)
            .ok()
            .and_then(|index| T::ENUM_CONSTANTS.get(index))
            .unwrap_or_else(|| {
                panic!(
                    "ArrayIndexOutOfBoundsException: Index {ordinal} out of bounds for length {}",
                    T::ENUM_CONSTANTS.len()
                )
            })
    }

    pub fn write_enum(&mut self, value: &impl Enum) -> &mut FriendlyByteBuf {
        self.write_var_int(value.ordinal())
    }

    /// A VarInt id, turned into the value it stands for.
    pub fn read_by_id<T>(&mut self, converter: impl Fn(i32) -> T) -> T {
        let id = self.read_var_int();
        converter(id)
    }

    pub fn write_by_id<T>(
        &mut self,
        converter: impl Fn(&T) -> i32,
        value: &T,
    ) -> &mut FriendlyByteBuf {
        let id = converter(value);
        self.write_var_int(id)
    }

    pub fn read_var_int(&mut self) -> i32 {
        VarInt::read(&mut self.source)
    }

    pub fn read_var_long(&mut self) -> i64 {
        VarLong::read(&mut self.source)
    }

    pub fn write_uuid(&mut self, uuid: &UUID) -> &mut FriendlyByteBuf {
        FriendlyByteBuf::write_uuid_to(&mut self.source, uuid);
        self
    }

    /// `writeUUID(ByteBuf, UUID)`: the most significant long, then the least.
    pub fn write_uuid_to(output: &mut ByteBuf, uuid: &UUID) {
        output.write_long(uuid.get_most_significant_bits());
        output.write_long(uuid.get_least_significant_bits());
    }

    pub fn read_uuid(&mut self) -> UUID {
        FriendlyByteBuf::read_uuid_from(&mut self.source)
    }

    /// `readUUID(ByteBuf)`.
    pub fn read_uuid_from(input: &mut ByteBuf) -> UUID {
        let most_sig_bits = input.read_long();
        let least_sig_bits = input.read_long();
        UUID::new(most_sig_bits, least_sig_bits)
    }

    pub fn write_var_int(&mut self, value: i32) -> &mut FriendlyByteBuf {
        VarInt::write(&mut self.source, value);
        self
    }

    pub fn write_var_long(&mut self, value: i64) -> &mut FriendlyByteBuf {
        VarLong::write(&mut self.source, value);
        self
    }

    /// Up to `MAX_STRING_LENGTH` UTF-16 units.
    pub fn read_utf(&mut self) -> String {
        self.read_utf_max(32767)
    }

    /// `readUtf(int)`.
    pub fn read_utf_max(&mut self, max_length: i32) -> String {
        Utf8String::read(&mut self.source, max_length)
    }

    pub fn write_utf(&mut self, value: &str) -> &mut FriendlyByteBuf {
        self.write_utf_max(value, 32767)
    }

    /// `writeUtf(String, int)`.
    pub fn write_utf_max(&mut self, value: &str, max_length: i32) -> &mut FriendlyByteBuf {
        Utf8String::write(&mut self.source, value, max_length);
        self
    }

    /// A string `Identifier.parse` takes, so one without a namespace is `minecraft`'s.
    pub fn read_identifier(&mut self) -> Identifier {
        Identifier::parse(&self.read_utf_max(32767))
    }

    pub fn write_identifier(&mut self, identifier: &Identifier) -> &mut FriendlyByteBuf {
        self.write_utf(&identifier.to_string());
        self
    }

    /// `readContainerId(ByteBuf)`: a VarInt.
    pub fn read_container_id_from(input: &mut ByteBuf) -> i32 {
        VarInt::read(input)
    }

    pub fn read_container_id(&mut self) -> i32 {
        FriendlyByteBuf::read_container_id_from(&mut self.source)
    }

    /// `writeContainerId(ByteBuf, int)`.
    pub fn write_container_id_to(output: &mut ByteBuf, id: i32) {
        VarInt::write(output, id);
    }

    pub fn write_container_id(&mut self, id: i32) {
        FriendlyByteBuf::write_container_id_to(&mut self.source, id);
    }
}

impl Deref for FriendlyByteBuf {
    type Target = ByteBuf;

    fn deref(&self) -> &ByteBuf {
        &self.source
    }
}

impl DerefMut for FriendlyByteBuf {
    fn deref_mut(&mut self) -> &mut ByteBuf {
        &mut self.source
    }
}

/// For the codecs: a codec of a `ByteBuf` or a `FriendlyByteBuf` works on this one.
impl AsMut<ByteBuf> for FriendlyByteBuf {
    fn as_mut(&mut self) -> &mut ByteBuf {
        &mut self.source
    }
}

impl AsMut<FriendlyByteBuf> for FriendlyByteBuf {
    fn as_mut(&mut self) -> &mut FriendlyByteBuf {
        self
    }
}

/// `unwrap()`: the buffer it wraps, which `StreamCodec::map_stream` hands back to its caller.
impl From<FriendlyByteBuf> for ByteBuf {
    fn from(buffer: FriendlyByteBuf) -> ByteBuf {
        buffer.source
    }
}

/// `new T[size]`, which throws `NegativeArraySizeException` for a negative size.
fn new_array<T: Clone + Default>(size: i32) -> Vec<T> {
    assert!(size >= 0, "NegativeArraySizeException: {size}");
    vec![T::default(); size as usize]
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::FriendlyByteBuf;
    use crate::golden::{Decode, Encode, Golden};
    use crate::io::netty::buffer::byte_buf::ByteBuf;
    use crate::java::util::uuid::UUID;
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::direction::Direction;
    use crate::net::minecraft::core::position::Position;
    use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
    use crate::net::minecraft::network::protocol::common::serverbound_resource_pack_packet::Action;
    use crate::net::minecraft::network::utf8_string::Utf8String;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::world::level::block::rotation::Rotation;
    use crate::net::minecraft::world::phys::vec3::Vec3;
    use crate::org::joml::quaternionf::Quaternionf;
    use crate::org::joml::vector3f::Vector3f;

    const DECODER: &str = "Lnet/minecraft/network/codec/StreamDecoder;";
    const ENCODER: &str = "Lnet/minecraft/network/codec/StreamEncoder;";

    // The buffer it wraps.
    impl Decode for FriendlyByteBuf {
        fn decode(value: &Value) -> Self {
            FriendlyByteBuf::new(ByteBuf::decode(value))
        }
    }

    impl Encode for FriendlyByteBuf {
        fn encode(&self) -> Value {
            self.source.encode()
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.FriendlyByteBuf");
        for name in [
            "readWithCodecTrusted(Lcom/mojang/serialization/DynamicOps;Lcom/mojang/serialization/Codec;)Ljava/lang/Object;",
            "readWithCodec(Lcom/mojang/serialization/DynamicOps;Lcom/mojang/serialization/Codec;Lnet/minecraft/nbt/NbtAccounter;)Ljava/lang/Object;",
            "writeWithCodec(Lcom/mojang/serialization/DynamicOps;Lcom/mojang/serialization/Codec;Ljava/lang/Object;)Lnet/minecraft/network/FriendlyByteBuf;",
            "readLenientJsonWithCodec(Lcom/mojang/serialization/Codec;)Ljava/lang/Object;",
            "writeJsonWithCodec(Lcom/mojang/serialization/Codec;Ljava/lang/Object;)V",
        ] {
            golden.pending(name, "waits for DataFixerUpper's codecs");
        }
        for name in [
            "readEither(Lnet/minecraft/network/codec/StreamDecoder;Lnet/minecraft/network/codec/StreamDecoder;)Lcom/mojang/datafixers/util/Either;",
            "writeEither(Lcom/mojang/datafixers/util/Either;Lnet/minecraft/network/codec/StreamEncoder;Lnet/minecraft/network/codec/StreamEncoder;)V",
        ] {
            golden.pending(name, "waits for DataFixerUpper's Either");
        }
        for name in [
            "readEnumSet(Ljava/lang/Class;)Ljava/util/EnumSet;",
            "writeEnumSet(Ljava/util/EnumSet;Ljava/lang/Class;)V",
        ] {
            golden.pending(
                name,
                "vanilla can't generate a Class argument yet, and it waits for java.util.BitSet",
            );
        }
        for name in [
            "readBitSet()Ljava/util/BitSet;",
            "writeBitSet(Ljava/util/BitSet;)V",
            "readFixedBitSet(I)Ljava/util/BitSet;",
            "readFixedBitSet(Lio/netty/buffer/ByteBuf;I)Ljava/util/BitSet;",
            "writeFixedBitSet(Ljava/util/BitSet;I)V",
            "writeFixedBitSet(Lio/netty/buffer/ByteBuf;Ljava/util/BitSet;I)V",
        ] {
            golden.pending(name, "waits for java.util.BitSet");
        }
        for name in [
            "readChunkPos()Lnet/minecraft/world/level/ChunkPos;",
            "readChunkPos(Lio/netty/buffer/ByteBuf;)Lnet/minecraft/world/level/ChunkPos;",
            "writeChunkPos(Lnet/minecraft/world/level/ChunkPos;)Lnet/minecraft/network/FriendlyByteBuf;",
            "writeChunkPos(Lio/netty/buffer/ByteBuf;Lnet/minecraft/world/level/ChunkPos;)V",
        ] {
            golden.pending(name, "waits for ChunkPos");
        }
        for name in [
            "readGlobalPos()Lnet/minecraft/core/GlobalPos;",
            "writeGlobalPos(Lnet/minecraft/core/GlobalPos;)V",
            "readResourceKey(Lnet/minecraft/resources/ResourceKey;)Lnet/minecraft/resources/ResourceKey;",
            "writeResourceKey(Lnet/minecraft/resources/ResourceKey;)V",
            "readRegistryKey()Lnet/minecraft/resources/ResourceKey;",
        ] {
            golden.pending(name, "waits for ResourceKey");
        }
        for name in [
            "readNbt()Lnet/minecraft/nbt/CompoundTag;",
            "readNbt(Lio/netty/buffer/ByteBuf;)Lnet/minecraft/nbt/CompoundTag;",
            "readNbt(Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/Tag;",
            "readNbt(Lio/netty/buffer/ByteBuf;Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/Tag;",
            "writeNbt(Lnet/minecraft/nbt/Tag;)Lnet/minecraft/network/FriendlyByteBuf;",
            "writeNbt(Lio/netty/buffer/ByteBuf;Lnet/minecraft/nbt/Tag;)V",
        ] {
            golden.pending(name, "waits for NBT");
        }
        golden.field("MAX_STRING_LENGTH", FriendlyByteBuf::MAX_STRING_LENGTH);
        golden.field(
            "MAX_COMPONENT_STRING_LENGTH",
            FriendlyByteBuf::MAX_COMPONENT_STRING_LENGTH,
        );
        golden.check("<init>(Lio/netty/buffer/ByteBuf;)V", FriendlyByteBuf::new);
        golden.check_method(
            "readWithCount(Ljava/util/function/Consumer;)V[readVarInt]",
            |this: &mut FriendlyByteBuf| {
                this.read_with_count(|input| {
                    input.read_var_int();
                })
            },
        );
        golden.check_method(
            "readWithCount(Ljava/util/function/Consumer;)V[readUtf]",
            |this: &mut FriendlyByteBuf| {
                this.read_with_count(|input| {
                    input.read_utf();
                })
            },
        );
        golden.check_method(
            &format!("writeOptional(Ljava/util/Optional;{ENCODER})V[VAR_INT]"),
            |this: &mut FriendlyByteBuf, value: Option<i32>| {
                this.write_optional(value.as_ref(), |output: &mut ByteBuf, value: &i32| {
                    ByteBufCodecs::VAR_INT.encode(output, value)
                })
            },
        );
        golden.check_method(
            &format!("writeOptional(Ljava/util/Optional;{ENCODER})V[Vec3's doubles]"),
            |this: &mut FriendlyByteBuf, value: Option<Vec3>| {
                this.write_optional(value.as_ref(), |o: &mut FriendlyByteBuf, pos: &Vec3| {
                    o.write_double(pos.x());
                    o.write_double(pos.y());
                    o.write_double(pos.z());
                })
            },
        );
        golden.check_method(
            &format!("readOptional({DECODER})Ljava/util/Optional;[VAR_INT]"),
            |this: &mut FriendlyByteBuf| {
                this.read_optional(|input: &mut ByteBuf| ByteBufCodecs::VAR_INT.decode(input))
            },
        );
        golden.check_method(
            &format!("readOptional({DECODER})Ljava/util/Optional;[Vec3's doubles]"),
            |this: &mut FriendlyByteBuf| {
                this.read_optional(|i: &mut FriendlyByteBuf| {
                    Vec3::new(i.read_double(), i.read_double(), i.read_double())
                })
            },
        );
        golden.check_method(
            &format!("readNullable({DECODER})Ljava/lang/Object;[readIdentifier]"),
            |this: &mut FriendlyByteBuf| this.read_nullable(FriendlyByteBuf::read_identifier),
        );
        golden.check_method(
            &format!("readNullable({DECODER})Ljava/lang/Object;[readUtf]"),
            |this: &mut FriendlyByteBuf| this.read_nullable(FriendlyByteBuf::read_utf),
        );
        golden.check_mut(
            &format!(
                "readNullable(Lio/netty/buffer/ByteBuf;{DECODER})Ljava/lang/Object;[Utf8String.read(1024)]"
            ),
            |input: &mut ByteBuf| {
                FriendlyByteBuf::read_nullable_from(input, |input: &mut ByteBuf| {
                    Utf8String::read(input, 1024)
                })
            },
        );
        golden.check_method(
            &format!("writeNullable(Ljava/lang/Object;{ENCODER})V[writeIdentifier]"),
            |this: &mut FriendlyByteBuf, value: Option<Identifier>| {
                this.write_nullable(
                    value.as_ref(),
                    |output: &mut FriendlyByteBuf, identifier: &Identifier| {
                        output.write_identifier(identifier);
                    },
                )
            },
        );
        golden.check_method(
            &format!("writeNullable(Ljava/lang/Object;{ENCODER})V[writeUtf]"),
            |this: &mut FriendlyByteBuf, value: Option<String>| {
                this.write_nullable(
                    value.as_ref(),
                    |output: &mut FriendlyByteBuf, value: &String| {
                        output.write_utf(value);
                    },
                )
            },
        );
        golden.check_mut(
            &format!(
                "writeNullable(Lio/netty/buffer/ByteBuf;Ljava/lang/Object;{ENCODER})V[Utf8String.write(1024)]"
            ),
            |output: &mut ByteBuf, value: &mut Option<String>| {
                FriendlyByteBuf::write_nullable_to(
                    output,
                    value.as_ref(),
                    |out: &mut ByteBuf, signature: &String| Utf8String::write(out, signature, 1024),
                )
            },
        );
        golden.check_method("readByteArray()[B", |this: &mut FriendlyByteBuf| {
            this.read_byte_array()
        });
        golden.check_mut(
            "readByteArray(Lio/netty/buffer/ByteBuf;)[B",
            |input: &mut ByteBuf| FriendlyByteBuf::read_byte_array_from(input),
        );
        golden.check_method(
            "writeByteArray([B)Lnet/minecraft/network/FriendlyByteBuf;",
            |this: &mut FriendlyByteBuf, bytes: Vec<i8>| this.write_byte_array(&bytes).clone(),
        );
        golden.check_mut(
            "writeByteArray(Lio/netty/buffer/ByteBuf;[B)V",
            |output: &mut ByteBuf, bytes: &mut Vec<i8>| {
                FriendlyByteBuf::write_byte_array_to(output, bytes)
            },
        );
        golden.check_method(
            "readByteArray(I)[B",
            |this: &mut FriendlyByteBuf, max_size: i32| this.read_byte_array_max(max_size),
        );
        golden.check_mut(
            "readByteArray(Lio/netty/buffer/ByteBuf;I)[B",
            |input: &mut ByteBuf, max_size: &mut i32| {
                FriendlyByteBuf::read_byte_array_from_max(input, *max_size)
            },
        );
        golden.check_method(
            "writeVarIntArray([I)Lnet/minecraft/network/FriendlyByteBuf;",
            |this: &mut FriendlyByteBuf, ints: Vec<i32>| this.write_var_int_array(&ints).clone(),
        );
        golden.check_method("readVarIntArray()[I", |this: &mut FriendlyByteBuf| {
            this.read_var_int_array()
        });
        golden.check_method(
            "readVarIntArray(I)[I",
            |this: &mut FriendlyByteBuf, max_size: i32| this.read_var_int_array_max(max_size),
        );
        golden.check_method(
            "writeLongArray([J)Lnet/minecraft/network/FriendlyByteBuf;",
            |this: &mut FriendlyByteBuf, longs: Vec<i64>| this.write_long_array(&longs).clone(),
        );
        golden.check_mut(
            "writeLongArray(Lio/netty/buffer/ByteBuf;[J)V",
            |output: &mut ByteBuf, longs: &mut Vec<i64>| {
                FriendlyByteBuf::write_long_array_to(output, longs)
            },
        );
        golden.check_method(
            "writeFixedSizeLongArray([J)Lnet/minecraft/network/FriendlyByteBuf;",
            |this: &mut FriendlyByteBuf, longs: Vec<i64>| {
                this.write_fixed_size_long_array(&longs).clone()
            },
        );
        golden.check_mut(
            "writeFixedSizeLongArray(Lio/netty/buffer/ByteBuf;[J)V",
            |output: &mut ByteBuf, longs: &mut Vec<i64>| {
                FriendlyByteBuf::write_fixed_size_long_array_to(output, longs)
            },
        );
        golden.check_method("readLongArray()[J", |this: &mut FriendlyByteBuf| {
            this.read_long_array()
        });
        golden.check_mut(
            "readFixedSizeLongArray([J)[J",
            |this: &mut FriendlyByteBuf, output: &mut Vec<i64>| {
                this.read_fixed_size_long_array(output).to_vec()
            },
        );
        golden.check_mut(
            "readLongArray(Lio/netty/buffer/ByteBuf;)[J",
            |input: &mut ByteBuf| FriendlyByteBuf::read_long_array_from(input),
        );
        golden.check_mut(
            "readFixedSizeLongArray(Lio/netty/buffer/ByteBuf;[J)[J",
            |input: &mut ByteBuf, output: &mut Vec<i64>| {
                FriendlyByteBuf::read_fixed_size_long_array_from(input, output).to_vec()
            },
        );
        golden.check_method(
            "readBlockPos()Lnet/minecraft/core/BlockPos;",
            |this: &mut FriendlyByteBuf| this.read_block_pos(),
        );
        golden.check_mut(
            "readBlockPos(Lio/netty/buffer/ByteBuf;)Lnet/minecraft/core/BlockPos;",
            |input: &mut ByteBuf| FriendlyByteBuf::read_block_pos_from(input),
        );
        golden.check_method(
            "writeBlockPos(Lnet/minecraft/core/BlockPos;)Lnet/minecraft/network/FriendlyByteBuf;",
            |this: &mut FriendlyByteBuf, pos: BlockPos| this.write_block_pos(&pos).clone(),
        );
        golden.check_mut(
            "writeBlockPos(Lio/netty/buffer/ByteBuf;Lnet/minecraft/core/BlockPos;)V",
            |output: &mut ByteBuf, pos: &mut BlockPos| {
                FriendlyByteBuf::write_block_pos_to(output, pos)
            },
        );
        golden.check_method(
            "readVector3f()Lorg/joml/Vector3f;",
            |this: &mut FriendlyByteBuf| this.read_vector3f(),
        );
        golden.check_mut(
            "readVector3f(Lio/netty/buffer/ByteBuf;)Lorg/joml/Vector3f;",
            |input: &mut ByteBuf| FriendlyByteBuf::read_vector3f_from(input),
        );
        golden.check_method(
            "writeVector3f(Lorg/joml/Vector3f;)V",
            |this: &mut FriendlyByteBuf, v: Vector3f| this.write_vector3f(&v),
        );
        golden.check_mut(
            "writeVector3f(Lio/netty/buffer/ByteBuf;Lorg/joml/Vector3fc;)V",
            |output: &mut ByteBuf, v: &mut Vector3f| FriendlyByteBuf::write_vector3f_to(output, v),
        );
        golden.check_method(
            "readQuaternion()Lorg/joml/Quaternionf;",
            |this: &mut FriendlyByteBuf| this.read_quaternion(),
        );
        golden.check_mut(
            "readQuaternion(Lio/netty/buffer/ByteBuf;)Lorg/joml/Quaternionf;",
            |input: &mut ByteBuf| FriendlyByteBuf::read_quaternion_from(input),
        );
        golden.check_method(
            "writeQuaternion(Lorg/joml/Quaternionf;)V",
            |this: &mut FriendlyByteBuf, q: Quaternionf| this.write_quaternion(&q),
        );
        golden.check_mut(
            "writeQuaternion(Lio/netty/buffer/ByteBuf;Lorg/joml/Quaternionfc;)V",
            |output: &mut ByteBuf, value: &mut Quaternionf| {
                FriendlyByteBuf::write_quaternion_to(output, value)
            },
        );
        golden.check_method(
            "readEnum(Ljava/lang/Class;)Ljava/lang/Enum;[ServerboundResourcePackPacket.Action]",
            |this: &mut FriendlyByteBuf| this.read_enum::<Action>(),
        );
        golden.check_method(
            "writeEnum(Ljava/lang/Enum;)Lnet/minecraft/network/FriendlyByteBuf;[ServerboundResourcePackPacket.Action]",
            |this: &mut FriendlyByteBuf, value: Action| this.write_enum(&value).clone(),
        );
        golden.check_method(
            "readById(Ljava/util/function/IntFunction;)Ljava/lang/Object;[Direction.BY_ID]",
            |this: &mut FriendlyByteBuf| this.read_by_id(Direction::BY_ID.as_ref()),
        );
        golden.check_method(
            "readById(Ljava/util/function/IntFunction;)Ljava/lang/Object;[Rotation.BY_ID]",
            |this: &mut FriendlyByteBuf| this.read_by_id(Rotation::BY_ID.as_ref()),
        );
        golden.check_method(
            "writeById(Ljava/util/function/ToIntFunction;Ljava/lang/Object;)Lnet/minecraft/network/FriendlyByteBuf;[Direction::get3DDataValue]",
            |this: &mut FriendlyByteBuf, value: Direction| {
                this.write_by_id(Direction::get_3d_data_value, &value).clone()
            },
        );
        golden.check_method(
            "writeById(Ljava/util/function/ToIntFunction;Ljava/lang/Object;)Lnet/minecraft/network/FriendlyByteBuf;[Direction::get2DDataValue]",
            |this: &mut FriendlyByteBuf, value: Direction| {
                this.write_by_id(Direction::get_2d_data_value, &value).clone()
            },
        );
        golden.check_method("readVarInt()I", |this: &mut FriendlyByteBuf| {
            this.read_var_int()
        });
        golden.check_method("readVarLong()J", |this: &mut FriendlyByteBuf| {
            this.read_var_long()
        });
        golden.check_method(
            "writeUUID(Ljava/util/UUID;)Lnet/minecraft/network/FriendlyByteBuf;",
            |this: &mut FriendlyByteBuf, uuid: UUID| this.write_uuid(&uuid).clone(),
        );
        golden.check_mut(
            "writeUUID(Lio/netty/buffer/ByteBuf;Ljava/util/UUID;)V",
            |output: &mut ByteBuf, uuid: &mut UUID| FriendlyByteBuf::write_uuid_to(output, uuid),
        );
        golden.check_method(
            "readUUID()Ljava/util/UUID;",
            |this: &mut FriendlyByteBuf| this.read_uuid(),
        );
        golden.check_mut(
            "readUUID(Lio/netty/buffer/ByteBuf;)Ljava/util/UUID;",
            |input: &mut ByteBuf| FriendlyByteBuf::read_uuid_from(input),
        );
        golden.check_method(
            "writeVarInt(I)Lnet/minecraft/network/FriendlyByteBuf;",
            |this: &mut FriendlyByteBuf, value: i32| this.write_var_int(value).clone(),
        );
        golden.check_method(
            "writeVarLong(J)Lnet/minecraft/network/FriendlyByteBuf;",
            |this: &mut FriendlyByteBuf, value: i64| this.write_var_long(value).clone(),
        );
        golden.check_method(
            "readUtf()Ljava/lang/String;",
            |this: &mut FriendlyByteBuf| this.read_utf(),
        );
        golden.check_method(
            "readUtf(I)Ljava/lang/String;",
            |this: &mut FriendlyByteBuf, max_length: i32| this.read_utf_max(max_length),
        );
        golden.check_method(
            "writeUtf(Ljava/lang/String;)Lnet/minecraft/network/FriendlyByteBuf;",
            |this: &mut FriendlyByteBuf, value: String| this.write_utf(&value).clone(),
        );
        golden.check_method(
            "writeUtf(Ljava/lang/String;I)Lnet/minecraft/network/FriendlyByteBuf;",
            |this: &mut FriendlyByteBuf, value: String, max_length: i32| {
                this.write_utf_max(&value, max_length).clone()
            },
        );
        golden.check_method(
            "readIdentifier()Lnet/minecraft/resources/Identifier;",
            |this: &mut FriendlyByteBuf| this.read_identifier(),
        );
        golden.check_method(
            "writeIdentifier(Lnet/minecraft/resources/Identifier;)Lnet/minecraft/network/FriendlyByteBuf;",
            |this: &mut FriendlyByteBuf, identifier: Identifier| {
                this.write_identifier(&identifier).clone()
            },
        );
        golden.check_mut(
            "readContainerId(Lio/netty/buffer/ByteBuf;)I",
            |input: &mut ByteBuf| FriendlyByteBuf::read_container_id_from(input),
        );
        golden.check_method("readContainerId()I", |this: &mut FriendlyByteBuf| {
            this.read_container_id()
        });
        golden.check_mut(
            "writeContainerId(Lio/netty/buffer/ByteBuf;I)V",
            |output: &mut ByteBuf, id: &mut i32| {
                FriendlyByteBuf::write_container_id_to(output, *id)
            },
        );
        golden.check_method(
            "writeContainerId(I)V",
            |this: &mut FriendlyByteBuf, id: i32| this.write_container_id(id),
        );
        golden.finish();
    }
}
