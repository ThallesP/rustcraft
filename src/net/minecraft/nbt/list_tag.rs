//! net/minecraft/nbt/ListTag.java
//!
//! Not ported yet: `toString` and `accept`, like every tag's (see `Tag`).
//!
//! Java's `ListTag` is an `AbstractList` over an `ArrayList`, which is a `Vec` here. Of the
//! `List` methods it inherits, `add(Tag)` is the one the game calls, as `add` here; `add(int,
//! Tag)` is `add_at`. A list holds tags of any kinds, but NBT writes one kind per list, so a list
//! of several is written as compounds, each holding its tag under the empty key, and reading
//! unwraps them (`addAndUnwrap`).

use std::fmt;

use std::borrow::Cow;

use crate::java::io::data_input::DataInput;
use crate::java::io::data_output::DataOutput;
use crate::java::lang::object::HashCode;
use crate::java::util::array_list::ArrayList;
use crate::net::minecraft::nbt::collection_tag::CollectionTag;
use crate::net::minecraft::nbt::compound_tag::CompoundTag;
use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
use crate::net::minecraft::nbt::numeric_tag::NumericTag;
use crate::net::minecraft::nbt::string_tag_visitor::StringTagVisitor;
use crate::net::minecraft::nbt::tag::{Tag, class};
use crate::net::minecraft::nbt::tag_type::TagType;
use crate::net::minecraft::nbt::tag_types::TagTypes;

/// A list of tags in NBT.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListTag {
    list: Vec<Tag>,
}

impl ListTag {
    pub const TYPE: TagType = TagType::List;
    const WRAPPER_MARKER: &'static str = "";
    const SELF_SIZE_IN_BYTES: i32 = 36;

    pub fn new() -> ListTag {
        ListTag { list: Vec::new() }
    }

    // A compound of one tag under the empty key is the tag it wraps.
    fn try_unwrap(mut tag: CompoundTag) -> Tag {
        if tag.size() == 1
            && let Some(value) = tag.remove(ListTag::WRAPPER_MARKER)
        {
            return value;
        }
        Tag::Compound(tag)
    }

    fn is_wrapper(tag: &CompoundTag) -> bool {
        tag.size() == 1 && tag.contains(ListTag::WRAPPER_MARKER)
    }

    // `wrapIfNeeded(elementType, tag).write(output)`: in a list written as compounds, a tag
    // that isn't a compound, or is one that looks like a wrapper, is written wrapped.
    fn write_wrapped_if_needed(element_type: i8, tag: &Tag, output: &mut dyn DataOutput) {
        if element_type != Tag::TAG_COMPOUND {
            tag.write(output);
        } else if let Tag::Compound(compound) = tag
            && !ListTag::is_wrapper(compound)
        {
            compound.write(output);
        } else {
            ListTag::write_wrapped(tag, output);
        }
    }

    // `wrapElement(tag).write(output)`: a compound of the tag under the empty key, written.
    fn write_wrapped(tag: &Tag, output: &mut dyn DataOutput) {
        CompoundTag::write_named_tag(ListTag::WRAPPER_MARKER, tag, output);
        output.write_byte(0);
    }

    /// The elements' kind, their count, then each element, wrapped if the list is of several
    /// kinds.
    pub fn write(&self, output: &mut dyn DataOutput) {
        let element_type = self.identify_raw_element_type();
        output.write_byte(i32::from(element_type));
        output.write_int(self.list.len() as i32);

        for element in &self.list {
            ListTag::write_wrapped_if_needed(element_type, element, output);
        }
    }

    /// The elements' id when they're all of one kind, the compound's when they aren't, and the
    /// end's for none. An `EndTag` (id 0) counts as no kind at all.
    pub(crate) fn identify_raw_element_type(&self) -> i8 {
        let mut homogenous_type = 0;

        for element in &self.list {
            let element_type = element.get_id();
            if homogenous_type == 0 {
                homogenous_type = element_type;
            } else if homogenous_type != element_type {
                return Tag::TAG_COMPOUND;
            }
        }

        homogenous_type
    }

    /// Adds the tag, or the tag a compound wraps.
    pub fn add_and_unwrap(&mut self, tag: Tag) {
        match tag {
            Tag::Compound(compound) => self.add(ListTag::try_unwrap(compound)),
            tag => self.add(tag),
        }
    }

    /// 36, 4 per element, and the elements'.
    pub fn size_in_bytes(&self) -> i32 {
        let mut size = ListTag::SELF_SIZE_IN_BYTES;
        size = size.wrapping_add(4i32.wrapping_mul(self.list.len() as i32));

        for child in &self.list {
            size = size.wrapping_add(child.size_in_bytes());
        }

        size
    }

    pub fn get_id(&self) -> i8 {
        9
    }

    pub fn get_type(&self) -> TagType {
        ListTag::TYPE
    }

    pub fn remove(&mut self, index: i32) -> Tag {
        self.list.remove(index_of(index))
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn get_compound(&self, index: i32) -> Option<&CompoundTag> {
        self.get_nullable(index).and_then(Tag::as_compound)
    }

    pub fn get_compound_or_empty(&self, index: i32) -> Cow<'_, CompoundTag> {
        self.get_compound(index)
            .map_or_else(|| Cow::Owned(CompoundTag::new()), Cow::Borrowed)
    }

    pub fn get_list(&self, index: i32) -> Option<&ListTag> {
        self.get_nullable(index).and_then(Tag::as_list)
    }

    pub fn get_list_or_empty(&self, index: i32) -> Cow<'_, ListTag> {
        self.get_list(index)
            .map_or_else(|| Cow::Owned(ListTag::new()), Cow::Borrowed)
    }

    pub fn get_short(&self, index: i32) -> Option<i16> {
        self.get_nullable(index).and_then(Tag::as_short)
    }

    pub fn get_short_or(&self, index: i32, default_value: i16) -> i16 {
        self.get_numeric(index)
            .map_or(default_value, NumericTag::short_value)
    }

    pub fn get_int(&self, index: i32) -> Option<i32> {
        self.get_nullable(index).and_then(Tag::as_int)
    }

    pub fn get_int_or(&self, index: i32, default_value: i32) -> i32 {
        self.get_numeric(index)
            .map_or(default_value, NumericTag::int_value)
    }

    pub fn get_int_array(&self, index: i32) -> Option<&[i32]> {
        self.get_nullable(index).and_then(Tag::as_int_array)
    }

    pub fn get_long_array(&self, index: i32) -> Option<&[i64]> {
        self.get_nullable(index).and_then(Tag::as_long_array)
    }

    pub fn get_double(&self, index: i32) -> Option<f64> {
        self.get_nullable(index).and_then(Tag::as_double)
    }

    pub fn get_double_or(&self, index: i32, default_value: f64) -> f64 {
        self.get_numeric(index)
            .map_or(default_value, NumericTag::double_value)
    }

    pub fn get_float(&self, index: i32) -> Option<f32> {
        self.get_nullable(index).and_then(Tag::as_float)
    }

    pub fn get_float_or(&self, index: i32, default_value: f32) -> f32 {
        self.get_numeric(index)
            .map_or(default_value, NumericTag::float_value)
    }

    pub fn get_string(&self, index: i32) -> Option<&str> {
        self.get_nullable(index).and_then(Tag::as_string)
    }

    pub fn get_string_or<'a>(&'a self, index: i32, default_value: &'a str) -> &'a str {
        self.get_string(index).unwrap_or(default_value)
    }

    // The tag at the index, or none for an index out of range.
    fn get_nullable(&self, index: i32) -> Option<&Tag> {
        usize::try_from(index)
            .ok()
            .and_then(|index| self.list.get(index))
    }

    fn get_numeric(&self, index: i32) -> Option<&dyn NumericTag> {
        self.get_nullable(index).and_then(Tag::as_numeric_tag)
    }

    pub fn size(&self) -> i32 {
        self.list.len() as i32
    }

    pub fn get(&self, index: i32) -> &Tag {
        &self.list[index_of(index)]
    }

    /// Puts the tag at the index, and returns the one that was there.
    pub fn set(&mut self, index: i32, tag: Tag) -> Tag {
        std::mem::replace(&mut self.list[index_of(index)], tag)
    }

    /// `add(int, Tag)`: inserts the tag at the index, which may be one past the end.
    pub fn add_at(&mut self, index: i32, tag: Tag) {
        let index = index_of(index);
        assert!(
            index <= self.list.len(),
            "Index: {index}, Size: {}",
            self.list.len()
        );
        self.list.insert(index, tag);
    }

    /// `add(Tag)`, `AbstractList`'s: adds the tag at the end.
    pub fn add(&mut self, tag: Tag) {
        self.list.push(tag);
    }

    pub fn copy(&self) -> ListTag {
        let mut copy = ArrayList::new(self.list.len() as i32);

        for tag in &self.list {
            copy.push(tag.copy());
        }

        ListTag { list: copy }
    }

    pub fn as_list(&self) -> Option<&ListTag> {
        Some(self)
    }

    /// `ArrayList.hashCode`: each element's hash added to 31 times the ones before, from 1.
    pub fn hash_code(&self) -> i32 {
        self.list.iter().fold(1, |hash: i32, element| {
            hash.wrapping_mul(31).wrapping_add(element.hash_code())
        })
    }

    /// The elements in order.
    pub fn stream(&self) -> impl Iterator<Item = &Tag> {
        self.list.iter()
    }

    /// The elements that are compounds, in order.
    pub fn compound_stream(&self) -> impl Iterator<Item = &CompoundTag> {
        self.stream().filter_map(Tag::as_compound)
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }

    /// `TYPE.load`: the list, a level deeper.
    pub(super) fn load(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> ListTag {
        let mut accounter = accounter.push_depth_until_dropped();
        ListTag::load_list(input, &mut accounter)
    }

    // The elements' type and count, accounted with the list's size and 4 per element before
    // they're read. Elements that are compounds wrapping a tag are that tag.
    fn load_list(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> ListTag {
        accounter.account_bytes(i64::from(ListTag::SELF_SIZE_IN_BYTES));
        let type_id = input.read_byte();
        let count = ListTag::read_list_count(input);
        assert!(type_id != 0 || count <= 0, "Missing type on ListTag");

        accounter.account_bytes_count(4, i64::from(count));
        let r#type = TagTypes::get_type(i32::from(type_id));
        let mut list = ListTag {
            list: ArrayList::new(count),
        };

        for _ in 0..count {
            list.add_and_unwrap(r#type.load(input, accounter));
        }

        list
    }

    fn read_list_count(input: &mut dyn DataInput) -> i32 {
        let count = input.read_int();
        assert!(count >= 0, "ListTag length cannot be negative: {count}");
        count
    }

    /// `TYPE.skip`: the elements' type and count, then that many elements, a level deeper.
    pub(super) fn skip(input: &mut dyn DataInput, accounter: &mut NbtAccounter) {
        let mut accounter = accounter.push_depth_until_dropped();
        let r#type = TagTypes::get_type(i32::from(input.read_byte()));
        let count = input.read_int();
        r#type.skip_count(input, count, &mut accounter);
    }
}

// A list index, which panics when it's negative, like Java's.
fn index_of(index: i32) -> usize {
    usize::try_from(index).unwrap_or_else(|_| panic!("Index {index} out of bounds"))
}

impl CollectionTag for ListTag {
    fn clear(&mut self) {
        ListTag::clear(self);
    }

    fn set_tag(&mut self, index: i32, tag: Tag) -> bool {
        self.list[index_of(index)] = tag;
        true
    }

    fn add_tag(&mut self, index: i32, tag: Tag) -> bool {
        self.add_at(index, tag);
        true
    }

    fn remove(&mut self, index: i32) -> Tag {
        ListTag::remove(self, index)
    }

    fn get(&self, index: i32) -> Tag {
        ListTag::get(self, index).clone()
    }

    fn size(&self) -> i32 {
        ListTag::size(self)
    }

    fn is_empty(&self) -> bool {
        ListTag::is_empty(self)
    }
}

class!(ListTag, List);

/// `toString()`: its SNBT (`StringTagVisitor`).
impl fmt::Display for ListTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut visitor = StringTagVisitor::new();
        visitor.visit_list(self);
        f.write_str(&visitor.build())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ListTag;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::nbt::collection_tag::CollectionTag;
    use crate::net::minecraft::nbt::tag::Tag;
    use crate::net::minecraft::nbt::tag::tests::{
        check_class, encode_payload, matches_payload, payload,
    };

    // [9, [tag, …]].
    impl Decode for ListTag {
        fn decode(value: &Value) -> Self {
            ListTag {
                list: Vec::decode(payload(value, 9)),
            }
        }
    }

    impl Encode for ListTag {
        fn encode(&self) -> Value {
            encode_payload(9, &self.list)
        }

        fn matches(&self, expected: &Value) -> bool {
            matches_payload(9, &self.list, expected)
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.ListTag");
        golden.check_method("toString()Ljava/lang/String;", |this: &mut ListTag| {
            this.to_string()
        });
        for name in [
            "accept(Lnet/minecraft/nbt/StreamTagVisitor;)Lnet/minecraft/nbt/StreamTagVisitor$ValueResult;",
            "accept(Lnet/minecraft/nbt/TagVisitor;)V",
        ] {
            golden.pending(name, "comes with the visitors");
        }
        golden.field("TYPE", ListTag::TYPE);
        golden.check("<init>()V", ListTag::new);
        check_class::<ListTag>(&golden, "copy()Lnet/minecraft/nbt/ListTag;");
        golden.check_method(
            "addAndUnwrap(Lnet/minecraft/nbt/Tag;)V",
            |this: &mut ListTag, tag: Tag| this.add_and_unwrap(tag),
        );
        golden.check_method(
            "remove(I)Lnet/minecraft/nbt/Tag;",
            |this: &mut ListTag, index: i32| this.remove(index),
        );
        golden.check_method("isEmpty()Z", |this: &mut ListTag| this.is_empty());
        golden.check_method(
            "getCompound(I)Ljava/util/Optional;",
            |this: &mut ListTag, index: i32| this.get_compound(index).cloned(),
        );
        golden.check_method(
            "getCompoundOrEmpty(I)Lnet/minecraft/nbt/CompoundTag;",
            |this: &mut ListTag, index: i32| this.get_compound_or_empty(index).into_owned(),
        );
        golden.check_method(
            "getList(I)Ljava/util/Optional;",
            |this: &mut ListTag, index: i32| this.get_list(index).cloned(),
        );
        golden.check_method(
            "getListOrEmpty(I)Lnet/minecraft/nbt/ListTag;",
            |this: &mut ListTag, index: i32| this.get_list_or_empty(index).into_owned(),
        );
        golden.check_method(
            "getShort(I)Ljava/util/Optional;",
            |this: &mut ListTag, index: i32| this.get_short(index),
        );
        golden.check_method(
            "getShortOr(IS)S",
            |this: &mut ListTag, index: i32, default_value: i16| {
                this.get_short_or(index, default_value)
            },
        );
        golden.check_method(
            "getInt(I)Ljava/util/Optional;",
            |this: &mut ListTag, index: i32| this.get_int(index),
        );
        golden.check_method(
            "getIntOr(II)I",
            |this: &mut ListTag, index: i32, default_value: i32| {
                this.get_int_or(index, default_value)
            },
        );
        golden.check_method(
            "getIntArray(I)Ljava/util/Optional;",
            |this: &mut ListTag, index: i32| this.get_int_array(index).map(<[i32]>::to_vec),
        );
        golden.check_method(
            "getLongArray(I)Ljava/util/Optional;",
            |this: &mut ListTag, index: i32| this.get_long_array(index).map(<[i64]>::to_vec),
        );
        golden.check_method(
            "getDouble(I)Ljava/util/Optional;",
            |this: &mut ListTag, index: i32| this.get_double(index),
        );
        golden.check_method(
            "getDoubleOr(ID)D",
            |this: &mut ListTag, index: i32, default_value: f64| {
                this.get_double_or(index, default_value)
            },
        );
        golden.check_method(
            "getFloat(I)Ljava/util/Optional;",
            |this: &mut ListTag, index: i32| this.get_float(index),
        );
        golden.check_method(
            "getFloatOr(IF)F",
            |this: &mut ListTag, index: i32, default_value: f32| {
                this.get_float_or(index, default_value)
            },
        );
        golden.check_method(
            "getString(I)Ljava/util/Optional;",
            |this: &mut ListTag, index: i32| this.get_string(index).map(str::to_owned),
        );
        golden.check_method(
            "getStringOr(ILjava/lang/String;)Ljava/lang/String;",
            |this: &mut ListTag, index: i32, default_value: String| {
                this.get_string_or(index, &default_value).to_owned()
            },
        );
        golden.check_method("size()I", |this: &mut ListTag| this.size());
        golden.check_method(
            "get(I)Lnet/minecraft/nbt/Tag;",
            |this: &mut ListTag, index: i32| this.get(index).clone(),
        );
        golden.check_method(
            "set(ILnet/minecraft/nbt/Tag;)Lnet/minecraft/nbt/Tag;",
            |this: &mut ListTag, index: i32, tag: Tag| this.set(index, tag),
        );
        golden.check_method(
            "add(ILnet/minecraft/nbt/Tag;)V",
            |this: &mut ListTag, index: i32, tag: Tag| this.add_at(index, tag),
        );
        golden.check_method(
            "setTag(ILnet/minecraft/nbt/Tag;)Z",
            |this: &mut ListTag, index: i32, tag: Tag| this.set_tag(index, tag),
        );
        golden.check_method(
            "addTag(ILnet/minecraft/nbt/Tag;)Z",
            |this: &mut ListTag, index: i32, tag: Tag| this.add_tag(index, tag),
        );
        golden.check_method("asList()Ljava/util/Optional;", |this: &mut ListTag| {
            this.as_list().cloned()
        });
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ListTag, other: ListTag| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut ListTag| this.hash_code());
        golden.check_method("stream()Ljava/util/stream/Stream;", |this: &mut ListTag| {
            this.stream().cloned().collect::<Vec<_>>()
        });
        golden.check_method(
            "compoundStream()Ljava/util/stream/Stream;",
            |this: &mut ListTag| this.compound_stream().cloned().collect::<Vec<_>>(),
        );
        golden.check_method("clear()V", |this: &mut ListTag| this.clear());
        golden.finish();
    }
}
