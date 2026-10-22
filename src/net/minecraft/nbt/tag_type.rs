//! net/minecraft/nbt/TagType.java
//!
//! Not ported yet: `parse` and `parseRoot`, which stream tags to a `StreamTagVisitor`, and come
//! with the visitors.
//!
//! Java's `TagType<T>` is an interface that each tag class implements in an anonymous class, its
//! `TYPE` constant (`TagType.StaticSize` for the numbers, `TagType.VariableSize` for the rest), and
//! that `createInvalid` implements for any other id. So it's an enum of those, whose methods call
//! the tag class's code (`ByteTag::load`), and its type argument is gone: `load` returns a `Tag`.
//! Java's `IOException`s are panics here.

use crate::java::io::data_input::DataInput;
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
use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
use crate::net::minecraft::nbt::short_tag::ShortTag;
use crate::net::minecraft::nbt::string_tag::StringTag;
use crate::net::minecraft::nbt::tag::Tag;

/// How to read and skip a kind of tag: one of the tag classes' `TYPE`s, or one for an id no tag
/// has, which panics at anything but its names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagType {
    End,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    ByteArray,
    String,
    List,
    Compound,
    IntArray,
    LongArray,
    Invalid(i32),
}

/// A type for an id no tag has, whose reads and skips panic, where Java throws `IOException`.
pub fn create_invalid(id: i32) -> TagType {
    TagType::Invalid(id)
}

impl TagType {
    /// Reads a tag of this kind, after its id (and name), accounting for it.
    pub fn load(self, input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> Tag {
        match self {
            TagType::End => Tag::End(EndTag::load(accounter)),
            TagType::Byte => Tag::Byte(ByteTag::load(input, accounter)),
            TagType::Short => Tag::Short(ShortTag::load(input, accounter)),
            TagType::Int => Tag::Int(IntTag::load(input, accounter)),
            TagType::Long => Tag::Long(LongTag::load(input, accounter)),
            TagType::Float => Tag::Float(FloatTag::load(input, accounter)),
            TagType::Double => Tag::Double(DoubleTag::load(input, accounter)),
            TagType::ByteArray => Tag::ByteArray(ByteArrayTag::load(input, accounter)),
            TagType::String => Tag::String(StringTag::load(input, accounter)),
            TagType::List => Tag::List(ListTag::load(input, accounter)),
            TagType::Compound => Tag::Compound(CompoundTag::load(input, accounter)),
            TagType::IntArray => Tag::IntArray(IntArrayTag::load(input, accounter)),
            TagType::LongArray => Tag::LongArray(LongArrayTag::load(input, accounter)),
            TagType::Invalid(id) => invalid(id),
        }
    }

    /// `skip(DataInput, int, NbtAccounter)`: skips `count` tags of this kind. The numbers skip
    /// their size times `count` at once (an int product, which wraps), the other kinds one at a
    /// time, and an `EndTag` has nothing to skip.
    pub fn skip_count(self, input: &mut dyn DataInput, count: i32, accounter: &mut NbtAccounter) {
        match self.size() {
            Some(size) => {
                input.skip_bytes(size.wrapping_mul(count));
            }
            None if self == TagType::End => {}
            None if let TagType::Invalid(id) = self => invalid(id),
            None => {
                for _ in 0..count {
                    self.skip(input, accounter);
                }
            }
        }
    }

    /// Skips a tag of this kind without making it. Only lists and compounds use the accounter,
    /// for their depth.
    pub fn skip(self, input: &mut dyn DataInput, accounter: &mut NbtAccounter) {
        match self {
            TagType::End => {}
            TagType::ByteArray => ByteArrayTag::skip(input),
            TagType::String => StringTag::skip_string(input),
            TagType::List => ListTag::skip(input, accounter),
            TagType::Compound => CompoundTag::skip(input, accounter),
            TagType::IntArray => IntArrayTag::skip(input),
            TagType::LongArray => LongArrayTag::skip(input),
            TagType::Invalid(id) => invalid(id),
            _ => {
                input.skip_bytes(self.size().expect("a number's type has a size"));
            }
        }
    }

    // `TagType.StaticSize.size()`: how many bytes a number's tag writes.
    fn size(self) -> Option<i32> {
        match self {
            TagType::Byte => Some(1),
            TagType::Short => Some(2),
            TagType::Int | TagType::Float => Some(4),
            TagType::Long | TagType::Double => Some(8),
            _ => None,
        }
    }

    pub fn get_name(self) -> String {
        match self {
            TagType::End => "END".to_owned(),
            TagType::Byte => "BYTE".to_owned(),
            TagType::Short => "SHORT".to_owned(),
            TagType::Int => "INT".to_owned(),
            TagType::Long => "LONG".to_owned(),
            TagType::Float => "FLOAT".to_owned(),
            TagType::Double => "DOUBLE".to_owned(),
            TagType::ByteArray => "BYTE[]".to_owned(),
            TagType::String => "STRING".to_owned(),
            TagType::List => "LIST".to_owned(),
            TagType::Compound => "COMPOUND".to_owned(),
            TagType::IntArray => "INT[]".to_owned(),
            TagType::LongArray => "LONG[]".to_owned(),
            TagType::Invalid(id) => format!("INVALID[{id}]"),
        }
    }

    pub fn get_pretty_name(self) -> String {
        match self {
            TagType::End => "TAG_End".to_owned(),
            TagType::Byte => "TAG_Byte".to_owned(),
            TagType::Short => "TAG_Short".to_owned(),
            TagType::Int => "TAG_Int".to_owned(),
            TagType::Long => "TAG_Long".to_owned(),
            TagType::Float => "TAG_Float".to_owned(),
            TagType::Double => "TAG_Double".to_owned(),
            TagType::ByteArray => "TAG_Byte_Array".to_owned(),
            TagType::String => "TAG_String".to_owned(),
            TagType::List => "TAG_List".to_owned(),
            TagType::Compound => "TAG_Compound".to_owned(),
            TagType::IntArray => "TAG_Int_Array".to_owned(),
            TagType::LongArray => "TAG_Long_Array".to_owned(),
            TagType::Invalid(id) => format!("UNKNOWN_{id}"),
        }
    }
}

fn invalid(id: i32) -> ! {
    panic!("Invalid tag id: {id}")
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{TagType, create_invalid};
    use crate::golden::{Decode, Encode, Golden};
    use crate::io::netty::buffer::byte_buf::ByteBuf;
    use crate::io::netty::buffer::byte_buf_input_stream::ByteBufInputStream;
    use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
    use crate::net::minecraft::nbt::tag_types::TagTypes;

    // By its name, `getName()`.
    impl Decode for TagType {
        fn decode(value: &Value) -> Self {
            let name = value.as_str().expect("a tag type is its name");
            if let Some(id) = name
                .strip_prefix("INVALID[")
                .and_then(|id| id.strip_suffix(']'))
            {
                return TagType::Invalid(id.parse().expect("an invalid type's id"));
            }
            (0..=12)
                .map(TagTypes::get_type)
                .find(|r#type| r#type.get_name() == name)
                .unwrap_or_else(|| panic!("no tag type is named {name}"))
        }
    }

    impl Encode for TagType {
        fn encode(&self) -> Value {
            Value::from(self.get_name())
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.TagType");
        for name in [
            "parse(Ljava/io/DataInput;Lnet/minecraft/nbt/StreamTagVisitor;Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/StreamTagVisitor$ValueResult;",
            "parseRoot(Ljava/io/DataInput;Lnet/minecraft/nbt/StreamTagVisitor;Lnet/minecraft/nbt/NbtAccounter;)V",
        ] {
            golden.pending(name, "comes with the stream visitors");
        }
        golden.check(
            "createInvalid(I)Lnet/minecraft/nbt/TagType;",
            create_invalid,
        );
        golden.check_mut(
            "load(Ljava/io/DataInput;Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/Tag;",
            |this: &mut TagType, input: &mut ByteBuf, accounter: &mut NbtAccounter| {
                this.load(&mut ByteBufInputStream::new(input), accounter)
            },
        );
        golden.check_mut(
            "skip(Ljava/io/DataInput;ILnet/minecraft/nbt/NbtAccounter;)V",
            |this: &mut TagType,
             input: &mut ByteBuf,
             count: &mut i32,
             accounter: &mut NbtAccounter| {
                this.skip_count(&mut ByteBufInputStream::new(input), *count, accounter)
            },
        );
        golden.check_mut(
            "skip(Ljava/io/DataInput;Lnet/minecraft/nbt/NbtAccounter;)V",
            |this: &mut TagType, input: &mut ByteBuf, accounter: &mut NbtAccounter| {
                this.skip(&mut ByteBufInputStream::new(input), accounter)
            },
        );
        golden.check_method("getName()Ljava/lang/String;", |this: &mut TagType| {
            this.get_name()
        });
        golden.check_method("getPrettyName()Ljava/lang/String;", |this: &mut TagType| {
            this.get_pretty_name()
        });
        golden.finish();
    }
}
