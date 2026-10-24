//! net/minecraft/nbt/CompoundTag.java
//!
//! Not ported yet: `toString` and `accept`, like every tag's (see `Tag`); `store`,
//! `storeNullable` and `read`, which wait for DataFixerUpper's codecs; `forEach`, whose
//! `BiConsumer` the golden harness can't generate (iterate `entry_set` instead). The
//! package-private `shallowCopy`, which `NbtOps` calls, copies the map (`new HashMap<>(tags)`,
//! sized for its size) with its tags, which Java shares: a copy of a value nothing changes after.
//!
//! `CODEC` is `Codec.PASSTHROUGH` read as the tag its data converts to, which must be a compound,
//! and written as the compound's data: `ExtraCodecs`' `NbtConverter`, which is that passthrough
//! for NBT, made to take and give compounds. A compound read from NBT is a copy (`copy()`, a new
//! map grown by putting each tag, so its table can differ), and so is what's written, like Java's.
//! `TagParser.LENIENT_CODEC`'s goldens check it.
//!
//! Its tags are in a `java::util::hash_map::HashMap`, a port of Java's, since a compound writes
//! its tags in the order its map iterates them. A key's getters hand out what they find, or the
//! `…Or` default, for a key that's missing or holds another kind of tag.

use std::borrow::Cow;
use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::codec::{BoxedCodec, StaticCodec};
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::{Encoder, Valued};

use crate::java::io::data_input::DataInput;
use crate::java::io::data_output::DataOutput;
use crate::java::util::hash_map::HashMap;
use crate::net::minecraft::nbt::byte_array_tag::ByteArrayTag;
use crate::net::minecraft::nbt::byte_tag::ByteTag;
use crate::net::minecraft::nbt::double_tag::DoubleTag;
use crate::net::minecraft::nbt::float_tag::FloatTag;
use crate::net::minecraft::nbt::int_array_tag::IntArrayTag;
use crate::net::minecraft::nbt::int_tag::IntTag;
use crate::net::minecraft::nbt::list_tag::ListTag;
use crate::net::minecraft::nbt::long_array_tag::LongArrayTag;
use crate::net::minecraft::nbt::long_tag::LongTag;
use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
use crate::net::minecraft::nbt::nbt_ops::NbtOps;
use crate::net::minecraft::nbt::numeric_tag::NumericTag;
use crate::net::minecraft::nbt::short_tag::ShortTag;
use crate::net::minecraft::nbt::string_tag::{self, StringTag};
use crate::net::minecraft::nbt::string_tag_visitor::StringTagVisitor;
use crate::net::minecraft::nbt::tag::{Tag, class};
use crate::net::minecraft::nbt::tag_type::TagType;
use crate::net::minecraft::nbt::tag_types::TagTypes;
use crate::net::minecraft::util::extra_codecs::{self, NbtConverter};

static CODEC: StaticCodec<CompoundTag> = LazyLock::new(|| BoxedCodec::new(CompoundTagCodec));

/// `CODEC`: `Codec.PASSTHROUGH.comapFlatMap(…)`, decoding a compound (a copy, when converting hands
/// back the input itself) and encoding a copy.
#[derive(Clone, Copy)]
struct CompoundTagCodec;

impl Valued for CompoundTagCodec {
    type Value = CompoundTag;
}

impl<T: OpsValue> Decoder<T> for CompoundTagCodec {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<CompoundTag, T>> {
        let same = extra_codecs::converts_to_input(ops, &NbtOps::INSTANCE);
        NbtConverter.decode(ops, input).flat_map(|pair| {
            let (tag, rest) = pair.into_parts();
            let result = match tag {
                Tag::Compound(compound_tag) if same => DataResult::success(compound_tag.copy()),
                Tag::Compound(compound_tag) => DataResult::success(compound_tag),
                tag => DataResult::error(move || format!("Not a compound tag: {tag}")),
            };
            result.map(|result| Pair::new(result, rest))
        })
    }
}

impl<T: OpsValue> Encoder<T> for CompoundTagCodec {
    fn encode(&self, input: &CompoundTag, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        NbtConverter.encode(&Tag::Compound(input.copy()), ops, prefix)
    }
}

/// Tags by name in NBT.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CompoundTag {
    tags: HashMap<String, Tag>,
}

impl CompoundTag {
    pub const CODEC: &'static StaticCodec<CompoundTag> = &CODEC;
    pub const TYPE: TagType = TagType::Compound;
    const SELF_SIZE_IN_BYTES: i32 = 48;

    pub fn new() -> CompoundTag {
        CompoundTag {
            tags: HashMap::new(),
        }
    }

    /// Each tag with its id and name, in the map's order, then an end.
    pub fn write(&self, output: &mut dyn DataOutput) {
        for (key, tag) in self.tags.iter() {
            CompoundTag::write_named_tag(key, tag, output);
        }

        output.write_byte(0);
    }

    /// 48, and per tag 28 and 2 per UTF-16 unit of its name, 36, and the tag's.
    pub fn size_in_bytes(&self) -> i32 {
        let mut size = CompoundTag::SELF_SIZE_IN_BYTES;

        for (key, tag) in self.tags.iter() {
            size = size
                .wrapping_add(28i32.wrapping_add(2i32.wrapping_mul(string_tag::utf16_length(key))));
            size = size.wrapping_add(36);
            size = size.wrapping_add(tag.size_in_bytes());
        }

        size
    }

    pub fn key_set(&self) -> impl Iterator<Item = &String> {
        self.tags.keys()
    }

    pub fn entry_set(&self) -> impl Iterator<Item = (&String, &Tag)> {
        self.tags.iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &Tag> {
        self.tags.values()
    }

    pub fn get_id(&self) -> i8 {
        10
    }

    pub fn get_type(&self) -> TagType {
        CompoundTag::TYPE
    }

    pub fn size(&self) -> i32 {
        self.tags.size()
    }

    /// Puts the tag under the name, and returns the one that was there.
    pub fn put(&mut self, name: String, tag: Tag) -> Option<Tag> {
        self.tags.put(name, tag)
    }

    pub fn put_byte(&mut self, name: String, value: i8) {
        self.tags.put(name, Tag::Byte(ByteTag::value_of(value)));
    }

    pub fn put_short(&mut self, name: String, value: i16) {
        self.tags.put(name, Tag::Short(ShortTag::value_of(value)));
    }

    pub fn put_int(&mut self, name: String, value: i32) {
        self.tags.put(name, Tag::Int(IntTag::value_of(value)));
    }

    pub fn put_long(&mut self, name: String, value: i64) {
        self.tags.put(name, Tag::Long(LongTag::value_of(value)));
    }

    pub fn put_float(&mut self, name: String, value: f32) {
        self.tags.put(name, Tag::Float(FloatTag::value_of(value)));
    }

    pub fn put_double(&mut self, name: String, value: f64) {
        self.tags.put(name, Tag::Double(DoubleTag::value_of(value)));
    }

    pub fn put_string(&mut self, name: String, value: String) {
        self.tags.put(name, Tag::String(StringTag::value_of(value)));
    }

    pub fn put_byte_array(&mut self, name: String, value: Vec<i8>) {
        self.tags
            .put(name, Tag::ByteArray(ByteArrayTag::new(value)));
    }

    pub fn put_int_array(&mut self, name: String, value: Vec<i32>) {
        self.tags.put(name, Tag::IntArray(IntArrayTag::new(value)));
    }

    pub fn put_long_array(&mut self, name: String, value: Vec<i64>) {
        self.tags
            .put(name, Tag::LongArray(LongArrayTag::new(value)));
    }

    /// A byte of 1 for true, 0 for false.
    pub fn put_boolean(&mut self, name: String, value: bool) {
        self.tags
            .put(name, Tag::Byte(ByteTag::value_of_bool(value)));
    }

    pub fn get(&self, name: &str) -> Option<&Tag> {
        self.tags.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tags.contains_key(name)
    }

    fn get_numeric(&self, name: &str) -> Option<&dyn NumericTag> {
        self.tags.get(name).and_then(Tag::as_numeric_tag)
    }

    pub fn get_byte(&self, name: &str) -> Option<i8> {
        self.get(name).and_then(Tag::as_byte)
    }

    pub fn get_byte_or(&self, name: &str, default_value: i8) -> i8 {
        self.get_numeric(name)
            .map_or(default_value, NumericTag::byte_value)
    }

    pub fn get_short(&self, name: &str) -> Option<i16> {
        self.get(name).and_then(Tag::as_short)
    }

    pub fn get_short_or(&self, name: &str, default_value: i16) -> i16 {
        self.get_numeric(name)
            .map_or(default_value, NumericTag::short_value)
    }

    pub fn get_int(&self, name: &str) -> Option<i32> {
        self.get(name).and_then(Tag::as_int)
    }

    pub fn get_int_or(&self, name: &str, default_value: i32) -> i32 {
        self.get_numeric(name)
            .map_or(default_value, NumericTag::int_value)
    }

    pub fn get_long(&self, name: &str) -> Option<i64> {
        self.get(name).and_then(Tag::as_long)
    }

    pub fn get_long_or(&self, name: &str, default_value: i64) -> i64 {
        self.get_numeric(name)
            .map_or(default_value, NumericTag::long_value)
    }

    pub fn get_float(&self, name: &str) -> Option<f32> {
        self.get(name).and_then(Tag::as_float)
    }

    pub fn get_float_or(&self, name: &str, default_value: f32) -> f32 {
        self.get_numeric(name)
            .map_or(default_value, NumericTag::float_value)
    }

    pub fn get_double(&self, name: &str) -> Option<f64> {
        self.get(name).and_then(Tag::as_double)
    }

    pub fn get_double_or(&self, name: &str, default_value: f64) -> f64 {
        self.get_numeric(name)
            .map_or(default_value, NumericTag::double_value)
    }

    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.get(name).and_then(Tag::as_string)
    }

    pub fn get_string_or<'a>(&'a self, name: &str, default_value: &'a str) -> &'a str {
        self.get_string(name).unwrap_or(default_value)
    }

    pub fn get_byte_array(&self, name: &str) -> Option<&[i8]> {
        self.get(name).and_then(Tag::as_byte_array)
    }

    pub fn get_int_array(&self, name: &str) -> Option<&[i32]> {
        self.get(name).and_then(Tag::as_int_array)
    }

    pub fn get_long_array(&self, name: &str) -> Option<&[i64]> {
        self.get(name).and_then(Tag::as_long_array)
    }

    pub fn get_compound(&self, name: &str) -> Option<&CompoundTag> {
        self.get(name).and_then(Tag::as_compound)
    }

    pub fn get_compound_or_empty(&self, name: &str) -> Cow<'_, CompoundTag> {
        self.get_compound(name)
            .map_or_else(|| Cow::Owned(CompoundTag::new()), Cow::Borrowed)
    }

    pub fn get_list(&self, name: &str) -> Option<&ListTag> {
        self.get(name).and_then(Tag::as_list)
    }

    pub fn get_list_or_empty(&self, name: &str) -> Cow<'_, ListTag> {
        self.get_list(name)
            .map_or_else(|| Cow::Owned(ListTag::new()), Cow::Borrowed)
    }

    /// Whether a number's byte isn't 0.
    pub fn get_boolean(&self, name: &str) -> Option<bool> {
        self.get(name).and_then(Tag::as_boolean)
    }

    pub fn get_boolean_or(&self, string: &str, default_value: bool) -> bool {
        self.get_byte_or(string, i8::from(default_value)) != 0
    }

    /// Takes the tag under the name out, and returns it.
    pub fn remove(&mut self, name: &str) -> Option<Tag> {
        self.tags.remove(name)
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    /// `shallowCopy`: the map copied, `new HashMap<>(tags)`, which sizes its table for the size.
    pub(crate) fn shallow_copy(&self) -> CompoundTag {
        CompoundTag {
            tags: HashMap::from_map(&self.tags),
        }
    }

    /// A copy of each tag, put in this compound's order into a new map.
    pub fn copy(&self) -> CompoundTag {
        let mut new_tags = HashMap::new();
        for (key, tag) in self.tags.iter() {
            new_tags.put(key.clone(), tag.copy());
        }
        CompoundTag { tags: new_tags }
    }

    pub fn as_compound(&self) -> Option<&CompoundTag> {
        Some(self)
    }

    /// `AbstractMap.hashCode`: each tag's name's hash xor the tag's, added up.
    pub fn hash_code(&self) -> i32 {
        self.tags.hash_code()
    }

    // The tag's id, then, unless it's an end, its name and the tag.
    pub(super) fn write_named_tag(name: &str, tag: &Tag, output: &mut dyn DataOutput) {
        output.write_byte(i32::from(tag.get_id()));
        if tag.get_id() != 0 {
            output.write_utf(name);
            tag.write(output);
        }
    }

    /// Puts a copy of each of the other's tags, in its order, except that a compound under a
    /// name that holds a compound here is merged into that one. Returns this compound.
    pub fn merge(&mut self, other: &CompoundTag) -> &mut CompoundTag {
        for (tag_name, other_tag) in other.tags.iter() {
            if let Tag::Compound(other_compound) = other_tag
                && let Some(Tag::Compound(self_compound)) = self.tags.get_mut(tag_name)
            {
                self_compound.merge(other_compound);
            } else {
                self.put(tag_name.clone(), other_tag.copy());
            }
        }

        self
    }

    /// `TYPE.load`: the compound, a level deeper.
    pub(super) fn load(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> CompoundTag {
        let mut accounter = accounter.push_depth_until_dropped();
        CompoundTag::load_compound(input, &mut accounter)
    }

    // Tags until an end, each its id, its name and the tag, accounted with the compound's size
    // and 36 for each name it didn't have yet. A name twice keeps the second tag where the
    // first was.
    fn load_compound(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> CompoundTag {
        accounter.account_bytes(i64::from(CompoundTag::SELF_SIZE_IN_BYTES));
        let mut values = HashMap::new();

        loop {
            let tag_type = input.read_byte();
            if tag_type == 0 {
                break;
            }
            let key = CompoundTag::read_string(input, accounter);
            let tag = TagTypes::get_type(i32::from(tag_type)).load(input, accounter);
            if values.put(key, tag).is_none() {
                accounter.account_bytes(36);
            }
        }

        CompoundTag { tags: values }
    }

    // A name, accounted as 28 and 2 per UTF-16 unit.
    fn read_string(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> String {
        let key = input.read_utf();
        accounter.account_bytes(28);
        accounter.account_bytes_count(2, i64::from(string_tag::utf16_length(&key)));
        key
    }

    /// `TYPE.skip`: tags until an end, each its id, its name and the tag, a level deeper.
    pub(super) fn skip(input: &mut dyn DataInput, accounter: &mut NbtAccounter) {
        let mut accounter = accounter.push_depth_until_dropped();
        loop {
            let tag_type_id = input.read_byte();
            if tag_type_id == 0 {
                break;
            }
            StringTag::skip_string(input);
            TagTypes::get_type(i32::from(tag_type_id)).skip(input, &mut accounter);
        }
    }
}

class!(CompoundTag, Compound);

/// `toString()`: its SNBT (`StringTagVisitor`).
impl fmt::Display for CompoundTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut visitor = StringTagVisitor::new();
        visitor.visit_compound(self);
        f.write_str(&visitor.build())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::CompoundTag;
    use crate::golden::{Decode, Encode, Golden, elements};
    use crate::java::util::hash_map::HashMap;
    use crate::net::minecraft::nbt::list_tag::ListTag;
    use crate::net::minecraft::nbt::tag::Tag;
    use crate::net::minecraft::nbt::tag::tests::{check_class, payload};

    // [10, [capacity, [[key, tag], …]]]: its map's table length, and its tags in order.
    impl Decode for CompoundTag {
        fn decode(value: &Value) -> Self {
            let [capacity, entries] = elements(payload(value, 10));
            CompoundTag {
                tags: HashMap::from_table(i32::decode(capacity), Vec::decode(entries)),
            }
        }
    }

    impl Encode for CompoundTag {
        fn encode(&self) -> Value {
            let entries: Vec<(String, Tag)> = self
                .tags
                .iter()
                .map(|(key, tag)| (key.clone(), tag.clone()))
                .collect();
            Value::from(vec![
                Value::from(10),
                (self.tags.capacity(), entries).encode(),
            ])
        }

        fn matches(&self, expected: &Value) -> bool {
            let Some([id, map]) = expected.as_array().map(Vec::as_slice) else {
                return false;
            };
            let Some([capacity, entries]) = map.as_array().map(Vec::as_slice) else {
                return false;
            };
            let Some(entries) = entries.as_array() else {
                return false;
            };
            id.as_i64() == Some(10)
                && self.tags.capacity().matches(capacity)
                && entries.len() == self.tags.size() as usize
                && self.tags.iter().zip(entries).all(|((key, tag), entry)| {
                    matches!(entry.as_array().map(Vec::as_slice), Some([expected_key, expected_tag])
                        if key.matches(expected_key) && tag.matches(expected_tag))
                })
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.CompoundTag");
        golden.check_method("toString()Ljava/lang/String;", |this: &mut CompoundTag| {
            this.to_string()
        });
        for name in [
            "accept(Lnet/minecraft/nbt/StreamTagVisitor;)Lnet/minecraft/nbt/StreamTagVisitor$ValueResult;",
            "accept(Lnet/minecraft/nbt/TagVisitor;)V",
        ] {
            golden.pending(name, "comes with the visitors");
        }
        golden.pending(
            "CODEC",
            "an instance suite can't encode a codec: TagParser's LENIENT_CODEC goldens check it",
        );
        for name in [
            "read(Lcom/mojang/serialization/MapCodec;)Ljava/util/Optional;",
            "read(Lcom/mojang/serialization/MapCodec;Lcom/mojang/serialization/DynamicOps;)Ljava/util/Optional;",
            "read(Ljava/lang/String;Lcom/mojang/serialization/Codec;)Ljava/util/Optional;",
            "read(Ljava/lang/String;Lcom/mojang/serialization/Codec;Lcom/mojang/serialization/DynamicOps;)Ljava/util/Optional;",
            "store(Lcom/mojang/serialization/MapCodec;Lcom/mojang/serialization/DynamicOps;Ljava/lang/Object;)V",
            "store(Lcom/mojang/serialization/MapCodec;Ljava/lang/Object;)V",
            "store(Ljava/lang/String;Lcom/mojang/serialization/Codec;Lcom/mojang/serialization/DynamicOps;Ljava/lang/Object;)V",
            "store(Ljava/lang/String;Lcom/mojang/serialization/Codec;Ljava/lang/Object;)V",
            "storeNullable(Ljava/lang/String;Lcom/mojang/serialization/Codec;Lcom/mojang/serialization/DynamicOps;Ljava/lang/Object;)V",
            "storeNullable(Ljava/lang/String;Lcom/mojang/serialization/Codec;Ljava/lang/Object;)V",
        ] {
            golden.pending(name, "waits for DataFixerUpper's codecs");
        }
        golden.pending(
            "forEach(Ljava/util/function/BiConsumer;)V",
            "the harness can't generate a BiConsumer; entry_set iterates the same",
        );
        golden.field("TYPE", CompoundTag::TYPE);
        golden.check("<init>()V", CompoundTag::new);
        check_class::<CompoundTag>(&golden, "copy()Lnet/minecraft/nbt/CompoundTag;");
        golden.check_method("keySet()Ljava/util/Set;", |this: &mut CompoundTag| {
            this.key_set().cloned().collect::<Vec<_>>()
        });
        golden.check_method("entrySet()Ljava/util/Set;", |this: &mut CompoundTag| {
            this.entry_set()
                .map(|(key, tag)| (key.clone(), tag.clone()))
                .collect::<Vec<_>>()
        });
        golden.check_method(
            "values()Ljava/util/Collection;",
            |this: &mut CompoundTag| this.values().cloned().collect::<Vec<_>>(),
        );
        golden.check_method("size()I", |this: &mut CompoundTag| this.size());
        golden.check_method(
            "put(Ljava/lang/String;Lnet/minecraft/nbt/Tag;)Lnet/minecraft/nbt/Tag;",
            |this: &mut CompoundTag, name: String, tag: Tag| this.put(name, tag),
        );
        golden.check_method(
            "putByte(Ljava/lang/String;B)V",
            |this: &mut CompoundTag, name: String, value: i8| this.put_byte(name, value),
        );
        golden.check_method(
            "putShort(Ljava/lang/String;S)V",
            |this: &mut CompoundTag, name: String, value: i16| this.put_short(name, value),
        );
        golden.check_method(
            "putInt(Ljava/lang/String;I)V",
            |this: &mut CompoundTag, name: String, value: i32| this.put_int(name, value),
        );
        golden.check_method(
            "putLong(Ljava/lang/String;J)V",
            |this: &mut CompoundTag, name: String, value: i64| this.put_long(name, value),
        );
        golden.check_method(
            "putFloat(Ljava/lang/String;F)V",
            |this: &mut CompoundTag, name: String, value: f32| this.put_float(name, value),
        );
        golden.check_method(
            "putDouble(Ljava/lang/String;D)V",
            |this: &mut CompoundTag, name: String, value: f64| this.put_double(name, value),
        );
        golden.check_method(
            "putString(Ljava/lang/String;Ljava/lang/String;)V",
            |this: &mut CompoundTag, name: String, value: String| this.put_string(name, value),
        );
        golden.check_method(
            "putByteArray(Ljava/lang/String;[B)V",
            |this: &mut CompoundTag, name: String, value: Vec<i8>| this.put_byte_array(name, value),
        );
        golden.check_method(
            "putIntArray(Ljava/lang/String;[I)V",
            |this: &mut CompoundTag, name: String, value: Vec<i32>| this.put_int_array(name, value),
        );
        golden.check_method(
            "putLongArray(Ljava/lang/String;[J)V",
            |this: &mut CompoundTag, name: String, value: Vec<i64>| {
                this.put_long_array(name, value)
            },
        );
        golden.check_method(
            "putBoolean(Ljava/lang/String;Z)V",
            |this: &mut CompoundTag, name: String, value: bool| this.put_boolean(name, value),
        );
        golden.check_method(
            "get(Ljava/lang/String;)Lnet/minecraft/nbt/Tag;",
            |this: &mut CompoundTag, name: String| this.get(&name).cloned(),
        );
        golden.check_method(
            "contains(Ljava/lang/String;)Z",
            |this: &mut CompoundTag, name: String| this.contains(&name),
        );
        golden.check_method(
            "getByte(Ljava/lang/String;)Ljava/util/Optional;",
            |this: &mut CompoundTag, name: String| this.get_byte(&name),
        );
        golden.check_method(
            "getByteOr(Ljava/lang/String;B)B",
            |this: &mut CompoundTag, name: String, default_value: i8| {
                this.get_byte_or(&name, default_value)
            },
        );
        golden.check_method(
            "getShort(Ljava/lang/String;)Ljava/util/Optional;",
            |this: &mut CompoundTag, name: String| this.get_short(&name),
        );
        golden.check_method(
            "getShortOr(Ljava/lang/String;S)S",
            |this: &mut CompoundTag, name: String, default_value: i16| {
                this.get_short_or(&name, default_value)
            },
        );
        golden.check_method(
            "getInt(Ljava/lang/String;)Ljava/util/Optional;",
            |this: &mut CompoundTag, name: String| this.get_int(&name),
        );
        golden.check_method(
            "getIntOr(Ljava/lang/String;I)I",
            |this: &mut CompoundTag, name: String, default_value: i32| {
                this.get_int_or(&name, default_value)
            },
        );
        golden.check_method(
            "getLong(Ljava/lang/String;)Ljava/util/Optional;",
            |this: &mut CompoundTag, name: String| this.get_long(&name),
        );
        golden.check_method(
            "getLongOr(Ljava/lang/String;J)J",
            |this: &mut CompoundTag, name: String, default_value: i64| {
                this.get_long_or(&name, default_value)
            },
        );
        golden.check_method(
            "getFloat(Ljava/lang/String;)Ljava/util/Optional;",
            |this: &mut CompoundTag, name: String| this.get_float(&name),
        );
        golden.check_method(
            "getFloatOr(Ljava/lang/String;F)F",
            |this: &mut CompoundTag, name: String, default_value: f32| {
                this.get_float_or(&name, default_value)
            },
        );
        golden.check_method(
            "getDouble(Ljava/lang/String;)Ljava/util/Optional;",
            |this: &mut CompoundTag, name: String| this.get_double(&name),
        );
        golden.check_method(
            "getDoubleOr(Ljava/lang/String;D)D",
            |this: &mut CompoundTag, name: String, default_value: f64| {
                this.get_double_or(&name, default_value)
            },
        );
        golden.check_method(
            "getString(Ljava/lang/String;)Ljava/util/Optional;",
            |this: &mut CompoundTag, name: String| this.get_string(&name).map(str::to_owned),
        );
        golden.check_method(
            "getStringOr(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            |this: &mut CompoundTag, name: String, default_value: String| {
                this.get_string_or(&name, &default_value).to_owned()
            },
        );
        golden.check_method(
            "getByteArray(Ljava/lang/String;)Ljava/util/Optional;",
            |this: &mut CompoundTag, name: String| this.get_byte_array(&name).map(<[i8]>::to_vec),
        );
        golden.check_method(
            "getIntArray(Ljava/lang/String;)Ljava/util/Optional;",
            |this: &mut CompoundTag, name: String| this.get_int_array(&name).map(<[i32]>::to_vec),
        );
        golden.check_method(
            "getLongArray(Ljava/lang/String;)Ljava/util/Optional;",
            |this: &mut CompoundTag, name: String| this.get_long_array(&name).map(<[i64]>::to_vec),
        );
        golden.check_method(
            "getCompound(Ljava/lang/String;)Ljava/util/Optional;",
            |this: &mut CompoundTag, name: String| this.get_compound(&name).cloned(),
        );
        golden.check_method(
            "getCompoundOrEmpty(Ljava/lang/String;)Lnet/minecraft/nbt/CompoundTag;",
            |this: &mut CompoundTag, name: String| this.get_compound_or_empty(&name).into_owned(),
        );
        golden.check_method(
            "getList(Ljava/lang/String;)Ljava/util/Optional;",
            |this: &mut CompoundTag, name: String| this.get_list(&name).cloned(),
        );
        golden.check_method(
            "getListOrEmpty(Ljava/lang/String;)Lnet/minecraft/nbt/ListTag;",
            |this: &mut CompoundTag, name: String| -> ListTag {
                this.get_list_or_empty(&name).into_owned()
            },
        );
        golden.check_method(
            "getBoolean(Ljava/lang/String;)Ljava/util/Optional;",
            |this: &mut CompoundTag, name: String| this.get_boolean(&name),
        );
        golden.check_method(
            "getBooleanOr(Ljava/lang/String;Z)Z",
            |this: &mut CompoundTag, name: String, default_value: bool| {
                this.get_boolean_or(&name, default_value)
            },
        );
        golden.check_method(
            "remove(Ljava/lang/String;)Lnet/minecraft/nbt/Tag;",
            |this: &mut CompoundTag, name: String| this.remove(&name),
        );
        golden.check_method("isEmpty()Z", |this: &mut CompoundTag| this.is_empty());
        golden.check_method(
            "asCompound()Ljava/util/Optional;",
            |this: &mut CompoundTag| this.as_compound().cloned(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut CompoundTag, other: CompoundTag| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut CompoundTag| this.hash_code());
        golden.check_method(
            "merge(Lnet/minecraft/nbt/CompoundTag;)Lnet/minecraft/nbt/CompoundTag;",
            |this: &mut CompoundTag, other: CompoundTag| this.merge(&other).clone(),
        );
        golden.finish();
    }
}
