//! net/minecraft/nbt/Tag.java
//!
//! Not ported yet: `accept` and `acceptAsRoot`, which come with the visitors.
//!
//! `Tag` is a sealed interface, which permits a class for each kind of tag, so it's an enum with
//! a variant holding each class's struct, and `instanceof` is a `match`. The interfaces between it
//! and the classes are traits: `NumericTag` for the numbers and `CollectionTag` for lists and
//! arrays (`as_numeric_tag` and `as_collection_tag` are their `instanceof`). `PrimitiveTag` only
//! gives `copy` a default that every class it permits overrides, so it's left out.
//!
//! Tags are values here: a list or compound owns what it holds, and a getter lends it, where Java
//! shares the reference. A tag a caller changes after getting it from another tag (Java's
//! `getCompound(name).get().putInt(…)`) is a question for when a caller does that.

use std::fmt;

use crate::java::io::data_output::DataOutput;
use crate::java::lang::number::Number;
use crate::java::lang::object::HashCode;
use crate::net::minecraft::nbt::byte_array_tag::ByteArrayTag;
use crate::net::minecraft::nbt::byte_tag::ByteTag;
use crate::net::minecraft::nbt::collection_tag::CollectionTag;
use crate::net::minecraft::nbt::compound_tag::CompoundTag;
use crate::net::minecraft::nbt::double_tag::DoubleTag;
use crate::net::minecraft::nbt::end_tag::EndTag;
use crate::net::minecraft::nbt::float_tag::FloatTag;
use crate::net::minecraft::nbt::int_array_tag::IntArrayTag;
use crate::net::minecraft::nbt::int_tag::IntTag;
use crate::net::minecraft::nbt::list_tag::ListTag;
use crate::net::minecraft::nbt::long_array_tag::LongArrayTag;
use crate::net::minecraft::nbt::long_tag::LongTag;
use crate::net::minecraft::nbt::numeric_tag::NumericTag;
use crate::net::minecraft::nbt::short_tag::ShortTag;
use crate::net::minecraft::nbt::string_tag::StringTag;
use crate::net::minecraft::nbt::string_tag_visitor::StringTagVisitor;
use crate::net::minecraft::nbt::tag_type::TagType;

/// A value in NBT, the format the game sends items, block entities and text in.
#[derive(Clone, Debug, PartialEq)]
pub enum Tag {
    End(EndTag),
    Byte(ByteTag),
    Short(ShortTag),
    Int(IntTag),
    Long(LongTag),
    Float(FloatTag),
    Double(DoubleTag),
    ByteArray(ByteArrayTag),
    String(StringTag),
    List(ListTag),
    Compound(CompoundTag),
    IntArray(IntArrayTag),
    LongArray(LongArrayTag),
}

impl Tag {
    pub const OBJECT_HEADER: i32 = 8;
    pub const ARRAY_HEADER: i32 = 12;
    pub const OBJECT_REFERENCE: i32 = 4;
    pub const STRING_SIZE: i32 = 28;
    pub const TAG_END: i8 = 0;
    pub const TAG_BYTE: i8 = 1;
    pub const TAG_SHORT: i8 = 2;
    pub const TAG_INT: i8 = 3;
    pub const TAG_LONG: i8 = 4;
    pub const TAG_FLOAT: i8 = 5;
    pub const TAG_DOUBLE: i8 = 6;
    pub const TAG_BYTE_ARRAY: i8 = 7;
    pub const TAG_STRING: i8 = 8;
    pub const TAG_LIST: i8 = 9;
    pub const TAG_COMPOUND: i8 = 10;
    pub const TAG_INT_ARRAY: i8 = 11;
    pub const TAG_LONG_ARRAY: i8 = 12;
    pub const MAX_DEPTH: i32 = 512;

    // The class's struct, for what every class implements its own way.
    fn class(&self) -> &dyn Class {
        match self {
            Tag::End(tag) => tag,
            Tag::Byte(tag) => tag,
            Tag::Short(tag) => tag,
            Tag::Int(tag) => tag,
            Tag::Long(tag) => tag,
            Tag::Float(tag) => tag,
            Tag::Double(tag) => tag,
            Tag::ByteArray(tag) => tag,
            Tag::String(tag) => tag,
            Tag::List(tag) => tag,
            Tag::Compound(tag) => tag,
            Tag::IntArray(tag) => tag,
            Tag::LongArray(tag) => tag,
        }
    }

    /// Writes the tag's payload, without its id.
    pub fn write(&self, output: &mut dyn DataOutput) {
        self.class().write(output);
    }

    pub fn get_id(&self) -> i8 {
        self.class().get_id()
    }

    pub fn get_type(&self) -> TagType {
        self.class().get_type()
    }

    /// A copy sharing nothing with the tag. Java's numbers and strings return themselves, which
    /// is the same value.
    pub fn copy(&self) -> Tag {
        self.class().copy()
    }

    /// Java's estimate of the bytes the tag takes in its heap.
    pub fn size_in_bytes(&self) -> i32 {
        self.class().size_in_bytes()
    }

    /// `instanceof NumericTag`.
    pub fn as_numeric_tag(&self) -> Option<&dyn NumericTag> {
        match self {
            Tag::Byte(tag) => Some(tag),
            Tag::Short(tag) => Some(tag),
            Tag::Int(tag) => Some(tag),
            Tag::Long(tag) => Some(tag),
            Tag::Float(tag) => Some(tag),
            Tag::Double(tag) => Some(tag),
            _ => None,
        }
    }

    /// `instanceof CollectionTag`.
    pub fn as_collection_tag(&self) -> Option<&dyn CollectionTag> {
        match self {
            Tag::List(tag) => Some(tag),
            Tag::ByteArray(tag) => Some(tag),
            Tag::IntArray(tag) => Some(tag),
            Tag::LongArray(tag) => Some(tag),
            _ => None,
        }
    }

    /// `instanceof CollectionTag`, to change it.
    pub fn as_collection_tag_mut(&mut self) -> Option<&mut dyn CollectionTag> {
        match self {
            Tag::List(tag) => Some(tag),
            Tag::ByteArray(tag) => Some(tag),
            Tag::IntArray(tag) => Some(tag),
            Tag::LongArray(tag) => Some(tag),
            _ => None,
        }
    }

    /// A string's value.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Tag::String(tag) => Some(tag.value()),
            _ => None,
        }
    }

    /// A number's value, boxed as its own kind (`NumericTag.asNumber`).
    pub fn as_number(&self) -> Option<Number> {
        self.as_numeric_tag().map(NumericTag::r#box)
    }

    // A number's conversions are `NumericTag`'s own, which override `Tag`'s, which would convert
    // `asNumber()`'s box but only ever see none.

    pub fn as_byte(&self) -> Option<i8> {
        self.as_numeric_tag().map(NumericTag::byte_value)
    }

    pub fn as_short(&self) -> Option<i16> {
        self.as_numeric_tag().map(NumericTag::short_value)
    }

    pub fn as_int(&self) -> Option<i32> {
        self.as_numeric_tag().map(NumericTag::int_value)
    }

    pub fn as_long(&self) -> Option<i64> {
        self.as_numeric_tag().map(NumericTag::long_value)
    }

    pub fn as_float(&self) -> Option<f32> {
        self.as_numeric_tag().map(NumericTag::float_value)
    }

    pub fn as_double(&self) -> Option<f64> {
        self.as_numeric_tag().map(NumericTag::double_value)
    }

    /// Whether a number's byte isn't 0.
    pub fn as_boolean(&self) -> Option<bool> {
        self.as_numeric_tag().map(|tag| tag.byte_value() != 0)
    }

    pub fn as_byte_array(&self) -> Option<&[i8]> {
        match self {
            Tag::ByteArray(tag) => Some(tag.get_as_byte_array()),
            _ => None,
        }
    }

    pub fn as_int_array(&self) -> Option<&[i32]> {
        match self {
            Tag::IntArray(tag) => Some(tag.get_as_int_array()),
            _ => None,
        }
    }

    pub fn as_long_array(&self) -> Option<&[i64]> {
        match self {
            Tag::LongArray(tag) => Some(tag.get_as_long_array()),
            _ => None,
        }
    }

    pub fn as_compound(&self) -> Option<&CompoundTag> {
        match self {
            Tag::Compound(tag) => Some(tag),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&ListTag> {
        match self {
            Tag::List(tag) => Some(tag),
            _ => None,
        }
    }
}

/// Each class's `hashCode`. An `EndTag`'s is `Object.hashCode`, an identity hash that changes
/// from run to run, which no tag the game reads holds, so its 0 here goes unchecked.
impl HashCode for Tag {
    fn hash_code(&self) -> i32 {
        match self {
            Tag::End(_) => 0,
            Tag::Byte(tag) => tag.hash_code(),
            Tag::Short(tag) => tag.hash_code(),
            Tag::Int(tag) => tag.hash_code(),
            Tag::Long(tag) => tag.hash_code(),
            Tag::Float(tag) => tag.hash_code(),
            Tag::Double(tag) => tag.hash_code(),
            Tag::ByteArray(tag) => tag.hash_code(),
            Tag::String(tag) => tag.hash_code(),
            Tag::List(tag) => tag.hash_code(),
            Tag::Compound(tag) => tag.hash_code(),
            Tag::IntArray(tag) => tag.hash_code(),
            Tag::LongArray(tag) => tag.hash_code(),
        }
    }
}

/// What every tag class implements of `Tag`, which the enum dispatches to.
pub(super) trait Class {
    fn write(&self, output: &mut dyn DataOutput);

    fn get_id(&self) -> i8;

    fn get_type(&self) -> TagType;

    fn copy(&self) -> Tag;

    fn size_in_bytes(&self) -> i32;
}

/// Makes a tag class one of `Tag`'s: its `Class` methods are its own, and it converts to its
/// variant. `class!(ByteTag, Byte);`
macro_rules! class {
    ($class:ident, $variant:ident) => {
        impl $crate::net::minecraft::nbt::tag::Class for $class {
            fn write(&self, output: &mut dyn $crate::java::io::data_output::DataOutput) {
                $class::write(self, output)
            }

            fn get_id(&self) -> i8 {
                $class::get_id(self)
            }

            fn get_type(&self) -> $crate::net::minecraft::nbt::tag_type::TagType {
                $class::get_type(self)
            }

            fn copy(&self) -> $crate::net::minecraft::nbt::tag::Tag {
                $crate::net::minecraft::nbt::tag::Tag::from($class::copy(self))
            }

            fn size_in_bytes(&self) -> i32 {
                $class::size_in_bytes(self)
            }
        }

        impl From<$class> for $crate::net::minecraft::nbt::tag::Tag {
            fn from(tag: $class) -> $crate::net::minecraft::nbt::tag::Tag {
                $crate::net::minecraft::nbt::tag::Tag::$variant(tag)
            }
        }
    };
}

pub(super) use class;

/// `toString()`: its SNBT (`StringTagVisitor`).
impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut visitor = StringTagVisitor::new();
        visitor.visit(self);
        f.write_str(&visitor.build())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::Value;

    use super::Tag;
    use crate::golden::{Decode, Encode, Golden, elements};
    use crate::io::netty::buffer::byte_buf::ByteBuf;
    use crate::io::netty::buffer::byte_buf_output_stream::ByteBufOutputStream;
    use crate::net::minecraft::nbt::byte_array_tag::ByteArrayTag;
    use crate::net::minecraft::nbt::byte_tag::ByteTag;
    use crate::net::minecraft::nbt::compound_tag::CompoundTag;
    use crate::net::minecraft::nbt::double_tag::DoubleTag;
    use crate::net::minecraft::nbt::end_tag::EndTag;
    use crate::net::minecraft::nbt::float_tag::FloatTag;
    use crate::net::minecraft::nbt::int_array_tag::IntArrayTag;
    use crate::net::minecraft::nbt::int_tag::IntTag;
    use crate::net::minecraft::nbt::list_tag::ListTag;
    use crate::net::minecraft::nbt::long_array_tag::LongArrayTag;
    use crate::net::minecraft::nbt::long_tag::LongTag;
    use crate::net::minecraft::nbt::short_tag::ShortTag;
    use crate::net::minecraft::nbt::string_tag::StringTag;

    // [id, payload], each class's own.
    impl Decode for Tag {
        fn decode(value: &Value) -> Self {
            let [id, _] = elements(value);
            match id.as_i64() {
                Some(0) => Tag::End(EndTag::decode(value)),
                Some(1) => Tag::Byte(ByteTag::decode(value)),
                Some(2) => Tag::Short(ShortTag::decode(value)),
                Some(3) => Tag::Int(IntTag::decode(value)),
                Some(4) => Tag::Long(LongTag::decode(value)),
                Some(5) => Tag::Float(FloatTag::decode(value)),
                Some(6) => Tag::Double(DoubleTag::decode(value)),
                Some(7) => Tag::ByteArray(ByteArrayTag::decode(value)),
                Some(8) => Tag::String(StringTag::decode(value)),
                Some(9) => Tag::List(ListTag::decode(value)),
                Some(10) => Tag::Compound(CompoundTag::decode(value)),
                Some(11) => Tag::IntArray(IntArrayTag::decode(value)),
                Some(12) => Tag::LongArray(LongArrayTag::decode(value)),
                _ => panic!("expected a tag, got {value}"),
            }
        }
    }

    impl Encode for Tag {
        fn encode(&self) -> Value {
            self.encoded().encode()
        }

        fn matches(&self, expected: &Value) -> bool {
            self.encoded().matches(expected)
        }
    }

    impl Tag {
        fn encoded(&self) -> &dyn Encode {
            match self {
                Tag::End(tag) => tag,
                Tag::Byte(tag) => tag,
                Tag::Short(tag) => tag,
                Tag::Int(tag) => tag,
                Tag::Long(tag) => tag,
                Tag::Float(tag) => tag,
                Tag::Double(tag) => tag,
                Tag::ByteArray(tag) => tag,
                Tag::String(tag) => tag,
                Tag::List(tag) => tag,
                Tag::Compound(tag) => tag,
                Tag::IntArray(tag) => tag,
                Tag::LongArray(tag) => tag,
            }
        }
    }

    /// The `[id, payload]` of a tag class's encoding, for its `Decode`.
    pub(crate) fn payload(value: &Value, id: i8) -> &Value {
        let [actual, payload] = elements(value);
        assert_eq!(
            actual.as_i64(),
            Some(i64::from(id)),
            "expected a tag of id {id}, got {value}"
        );
        payload
    }

    /// Encodes a tag class as `[id, payload]`, matching the payload with its own `matches`.
    pub(crate) fn encode_payload(id: i8, payload: &dyn Encode) -> Value {
        Value::from(vec![Value::from(id), payload.encode()])
    }

    pub(crate) fn matches_payload(id: i8, payload: &dyn Encode, expected: &Value) -> bool {
        match expected.as_array().map(Vec::as_slice) {
            Some([actual, expected]) => {
                actual.as_i64() == Some(i64::from(id)) && payload.matches(expected)
            }
            _ => false,
        }
    }

    /// A tag class's own `write`, `getId`, `getType`, `copy` and `sizeInBytes`, which every class
    /// declares, checked on its receivers.
    pub(crate) fn check_class<T>(golden: &Golden, copy: &str)
    where
        T: Decode + Encode + Clone + Into<Tag> + 'static,
    {
        golden.check_mut(
            "write(Ljava/io/DataOutput;)V",
            |this: &mut T, output: &mut ByteBuf| {
                this.clone()
                    .into()
                    .write(&mut ByteBufOutputStream::new(output))
            },
        );
        golden.check_method("getId()B", |this: &mut T| this.clone().into().get_id());
        golden.check_method("getType()Lnet/minecraft/nbt/TagType;", |this: &mut T| {
            this.clone().into().get_type()
        });
        golden.check_method(copy, |this: &mut T| this.clone().into().copy());
        golden.check_method("sizeInBytes()I", |this: &mut T| {
            this.clone().into().size_in_bytes()
        });
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.Tag");
        golden.check_method("toString()Ljava/lang/String;", |this: &mut Tag| {
            this.to_string()
        });
        for name in [
            "accept(Lnet/minecraft/nbt/StreamTagVisitor;)Lnet/minecraft/nbt/StreamTagVisitor$ValueResult;",
            "accept(Lnet/minecraft/nbt/TagVisitor;)V",
            "acceptAsRoot(Lnet/minecraft/nbt/StreamTagVisitor;)V",
        ] {
            golden.pending(name, "comes with the visitors");
        }
        golden.field("OBJECT_HEADER", Tag::OBJECT_HEADER);
        golden.field("ARRAY_HEADER", Tag::ARRAY_HEADER);
        golden.field("OBJECT_REFERENCE", Tag::OBJECT_REFERENCE);
        golden.field("STRING_SIZE", Tag::STRING_SIZE);
        golden.field("TAG_END", Tag::TAG_END);
        golden.field("TAG_BYTE", Tag::TAG_BYTE);
        golden.field("TAG_SHORT", Tag::TAG_SHORT);
        golden.field("TAG_INT", Tag::TAG_INT);
        golden.field("TAG_LONG", Tag::TAG_LONG);
        golden.field("TAG_FLOAT", Tag::TAG_FLOAT);
        golden.field("TAG_DOUBLE", Tag::TAG_DOUBLE);
        golden.field("TAG_BYTE_ARRAY", Tag::TAG_BYTE_ARRAY);
        golden.field("TAG_STRING", Tag::TAG_STRING);
        golden.field("TAG_LIST", Tag::TAG_LIST);
        golden.field("TAG_COMPOUND", Tag::TAG_COMPOUND);
        golden.field("TAG_INT_ARRAY", Tag::TAG_INT_ARRAY);
        golden.field("TAG_LONG_ARRAY", Tag::TAG_LONG_ARRAY);
        golden.field("MAX_DEPTH", Tag::MAX_DEPTH);
        golden.check_mut(
            "write(Ljava/io/DataOutput;)V",
            |this: &mut Tag, output: &mut ByteBuf| {
                this.write(&mut ByteBufOutputStream::new(output))
            },
        );
        golden.check_method("getId()B", |this: &mut Tag| this.get_id());
        golden.check_method("getType()Lnet/minecraft/nbt/TagType;", |this: &mut Tag| {
            this.get_type()
        });
        golden.check_method("copy()Lnet/minecraft/nbt/Tag;", |this: &mut Tag| {
            this.copy()
        });
        golden.check_method("sizeInBytes()I", |this: &mut Tag| this.size_in_bytes());
        golden.check_method("asString()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_string().map(str::to_owned)
        });
        golden.check_method("asNumber()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_number()
        });
        golden.check_method("asByte()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_byte()
        });
        golden.check_method("asShort()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_short()
        });
        golden.check_method("asInt()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_int()
        });
        golden.check_method("asLong()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_long()
        });
        golden.check_method("asFloat()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_float()
        });
        golden.check_method("asDouble()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_double()
        });
        golden.check_method("asBoolean()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_boolean()
        });
        golden.check_method("asByteArray()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_byte_array().map(<[i8]>::to_vec)
        });
        golden.check_method("asIntArray()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_int_array().map(<[i32]>::to_vec)
        });
        golden.check_method("asLongArray()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_long_array().map(<[i64]>::to_vec)
        });
        golden.check_method("asCompound()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_compound().cloned()
        });
        golden.check_method("asList()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_list().cloned()
        });
        golden.finish();
    }
}
