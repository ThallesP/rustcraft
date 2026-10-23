//! net/minecraft/nbt/NbtOps.java
//!
//! `DynamicOps` over NBT's tags, which the game's codecs read and write text components, item
//! components and the rest of what play's packets carry as NBT with. A map is a `CompoundTag`
//! with string keys, a list a `ListTag` or one of the arrays, and a number keeps its tag's kind.
//!
//! Merging into a list goes through a collector that keeps a byte, int or long array as one
//! while what's merged is a tag of its kind, and makes it a list otherwise (`createCollector`).
//! Java's `ListCollector`s are an enum of the four here.

use std::fmt;

use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::codec::{BoxedCodec, Codec};
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{self, DynamicOps, OpsKind, OpsValue};
use crate::com::mojang::serialization::list_builder::{self, ListBuilder};
use crate::com::mojang::serialization::map_codec::{BoxedMapCodec, MapCodec};
use crate::com::mojang::serialization::map_like::MapLike;
use crate::com::mojang::serialization::record_builder::{
    RecordBuilder, StringBuilder, StringBuilderOps,
};
use crate::java::lang::number::Number;
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
use crate::net::minecraft::nbt::tag::Tag;

pub struct NbtOps;

impl NbtOps {
    pub const INSTANCE: NbtOps = NbtOps;

    // `createCollector`: a list collector starting from the tag, for an end or a collection.
    fn create_collector(tag: &Tag) -> Option<ListCollector> {
        if let Tag::End(_) = tag {
            return Some(ListCollector::Generic(ListTag::new()));
        }

        let collection = tag.as_collection_tag()?;
        if collection.is_empty() {
            return Some(ListCollector::Generic(ListTag::new()));
        }

        Some(match tag {
            Tag::List(list) => {
                let mut result = ListTag::new();
                for element in list.stream() {
                    result.add(element.clone());
                }
                ListCollector::Generic(result)
            }
            Tag::ByteArray(array) => ListCollector::Byte(array.get_as_byte_array().to_vec()),
            Tag::IntArray(array) => ListCollector::Int(array.get_as_int_array().to_vec()),
            Tag::LongArray(array) => ListCollector::Long(array.get_as_long_array().to_vec()),
            _ => unreachable!("a collection is a list or an array"),
        })
    }

    // A compound, or an end, which merges as the empty map.
    fn is_map_or_empty(map: &Tag) -> bool {
        matches!(map, Tag::Compound(_) | Tag::End(_))
    }

    // `map instanceof CompoundTag tag ? tag.shallowCopy() : new CompoundTag()`.
    fn copy_or_new(map: &Tag) -> CompoundTag {
        match map {
            Tag::Compound(tag) => tag.shallow_copy(),
            _ => CompoundTag::new(),
        }
    }

    // The compound's entries, each key a `StringTag`.
    fn entries(&self, tag: &CompoundTag) -> Vec<Pair<Tag, Tag>> {
        tag.entry_set()
            .map(|(key, value)| Pair::new(self.create_string(key), value.clone()))
            .collect()
    }
}

/// `NBT`.
impl fmt::Display for NbtOps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NBT")
    }
}

impl OpsValue for Tag {
    /// `NbtOps.convertTo`: each kind of tag as the other ops' kind of value.
    fn convert_to<U: OpsValue>(out_ops: &dyn DynamicOps<U>, input: &Tag) -> U {
        match input {
            Tag::End(_) => out_ops.empty(),
            Tag::Byte(tag) => out_ops.create_byte(tag.value()),
            Tag::Short(tag) => out_ops.create_short(tag.value()),
            Tag::Int(tag) => out_ops.create_int(tag.value()),
            Tag::Long(tag) => out_ops.create_long(tag.value()),
            Tag::Float(tag) => out_ops.create_float(tag.value()),
            Tag::Double(tag) => out_ops.create_double(tag.value()),
            Tag::ByteArray(tag) => out_ops.create_byte_list(tag.get_as_byte_array()),
            Tag::String(tag) => out_ops.create_string(tag.value()),
            Tag::List(_) => dynamic_ops::convert_list(&NbtOps::INSTANCE, out_ops, input),
            Tag::Compound(_) => dynamic_ops::convert_map(&NbtOps::INSTANCE, out_ops, input),
            Tag::IntArray(tag) => out_ops.create_int_list(tag.get_as_int_array()),
            Tag::LongArray(tag) => out_ops.create_long_list(tag.get_as_long_array()),
        }
    }

    fn boxed_codec<A>(codec: &BoxedCodec<A>) -> &(dyn Codec<Tag, Value = A> + Send + Sync) {
        codec.nbt()
    }

    fn boxed_map_codec<A>(
        codec: &BoxedMapCodec<A>,
    ) -> &(dyn MapCodec<Tag, Value = A> + Send + Sync) {
        codec.nbt()
    }
}

impl DynamicOps<Tag> for NbtOps {
    fn kind(&self) -> OpsKind {
        OpsKind::Nbt
    }

    fn empty(&self) -> Tag {
        Tag::End(EndTag::INSTANCE)
    }

    fn empty_list(&self) -> Tag {
        Tag::List(ListTag::new())
    }

    fn empty_map(&self) -> Tag {
        Tag::Compound(CompoundTag::new())
    }

    fn get_number_value(&self, input: &Tag) -> DataResult<Number> {
        match input.as_number() {
            Some(number) => DataResult::success(number),
            None => DataResult::error(|| "Not a number".to_owned()),
        }
    }

    /// A double, whatever the box.
    fn create_numeric(&self, i: Number) -> Tag {
        Tag::Double(DoubleTag::value_of(i.double_value()))
    }

    fn create_byte(&self, value: i8) -> Tag {
        Tag::Byte(ByteTag::value_of(value))
    }

    fn create_short(&self, value: i16) -> Tag {
        Tag::Short(ShortTag::value_of(value))
    }

    fn create_int(&self, value: i32) -> Tag {
        Tag::Int(IntTag::value_of(value))
    }

    fn create_long(&self, value: i64) -> Tag {
        Tag::Long(LongTag::value_of(value))
    }

    fn create_float(&self, value: f32) -> Tag {
        Tag::Float(FloatTag::value_of(value))
    }

    fn create_double(&self, value: f64) -> Tag {
        Tag::Double(DoubleTag::value_of(value))
    }

    /// Whether a number's double isn't 0.
    fn get_boolean_value(&self, input: &Tag) -> DataResult<bool> {
        self.get_number_value(input)
            .map(|value| value.double_value() != 0.0)
    }

    fn create_boolean(&self, value: bool) -> Tag {
        Tag::Byte(ByteTag::value_of_bool(value))
    }

    fn get_string_value(&self, input: &Tag) -> DataResult<String> {
        match input {
            Tag::String(tag) => DataResult::success(tag.value().to_owned()),
            _ => DataResult::error(|| "Not a string".to_owned()),
        }
    }

    fn create_string(&self, value: &str) -> Tag {
        Tag::String(StringTag::value_of(value.to_owned()))
    }

    fn merge_to_list(&self, list: &Tag, value: Tag) -> DataResult<Tag> {
        match NbtOps::create_collector(list) {
            Some(collector) => DataResult::success(collector.accept(value).result()),
            None => DataResult::error_partial(
                || format!("mergeToList called with not a list: {list}"),
                list.clone(),
            ),
        }
    }

    fn merge_to_list_values(&self, list: &Tag, values: Vec<Tag>) -> DataResult<Tag> {
        match NbtOps::create_collector(list) {
            Some(collector) => DataResult::success(collector.accept_all(values).result()),
            None => DataResult::error_partial(
                || format!("mergeToList called with not a list: {list}"),
                list.clone(),
            ),
        }
    }

    fn merge_to_map(&self, map: &Tag, key: Tag, value: Tag) -> DataResult<Tag> {
        if !NbtOps::is_map_or_empty(map) {
            return DataResult::error_partial(
                || format!("mergeToMap called with not a map: {map}"),
                map.clone(),
            );
        }
        match key {
            Tag::String(string_key) => {
                let mut output = NbtOps::copy_or_new(map);
                output.put(string_key.value().to_owned(), value);
                DataResult::success(Tag::Compound(output))
            }
            key => DataResult::error_partial(|| format!("key is not a string: {key}"), map.clone()),
        }
    }

    fn merge_to_map_like(&self, map: &Tag, values: &dyn MapLike<Tag>) -> DataResult<Tag> {
        if !NbtOps::is_map_or_empty(map) {
            return DataResult::error_partial(
                || format!("mergeToMap called with not a map: {map}"),
                map.clone(),
            );
        }
        let entries = values.entries();
        self.merge_entries(map, entries)
    }

    fn merge_to_map_values(&self, map: &Tag, values: Vec<Pair<Tag, Tag>>) -> DataResult<Tag> {
        if !NbtOps::is_map_or_empty(map) {
            return DataResult::error_partial(
                || format!("mergeToMap called with not a map: {map}"),
                map.clone(),
            );
        }
        self.merge_entries(map, values)
    }

    fn get_map_values(&self, input: &Tag) -> DataResult<Vec<Pair<Tag, Tag>>> {
        match input {
            Tag::Compound(tag) => DataResult::success(self.entries(tag)),
            _ => DataResult::error(|| format!("Not a map: {input}")),
        }
    }

    fn get_map<'a>(&self, input: &'a Tag) -> DataResult<Box<dyn MapLike<Tag> + 'a>> {
        match input {
            Tag::Compound(tag) => DataResult::success(Box::new(CompoundMapLike { tag })),
            _ => DataResult::error(|| format!("Not a map: {input}")),
        }
    }

    /// A map of string keys; another key panics, where Java throws
    /// `UnsupportedOperationException`.
    fn create_map(&self, map: Vec<Pair<Tag, Tag>>) -> Tag {
        let mut tag = CompoundTag::new();
        for entry in map {
            let (key, value) = entry.into_parts();
            match key {
                Tag::String(string_key) => {
                    tag.put(string_key.value().to_owned(), value);
                }
                key => panic!(
                    "UnsupportedOperationException: Cannot create map with non-string key: {key}"
                ),
            }
        }
        Tag::Compound(tag)
    }

    fn get_stream(&self, input: &Tag) -> DataResult<Vec<Tag>> {
        match input.as_collection_tag() {
            Some(collection) => DataResult::success(collection.stream().collect()),
            None => DataResult::error(|| "Not a list".to_owned()),
        }
    }

    fn get_list(&self, input: &Tag) -> DataResult<Vec<Tag>> {
        match input.as_collection_tag() {
            Some(collection) => DataResult::success(collection.iterator().collect()),
            None => DataResult::error(|| format!("Not a list: {input}")),
        }
    }

    fn get_byte_buffer(&self, input: &Tag) -> DataResult<Vec<i8>> {
        match input {
            Tag::ByteArray(array) => DataResult::success(array.get_as_byte_array().to_vec()),
            _ => dynamic_ops::get_byte_buffer(self, input),
        }
    }

    fn create_byte_list(&self, input: &[i8]) -> Tag {
        Tag::ByteArray(ByteArrayTag::new(input.to_vec()))
    }

    fn get_int_stream(&self, input: &Tag) -> DataResult<Vec<i32>> {
        match input {
            Tag::IntArray(array) => DataResult::success(array.get_as_int_array().to_vec()),
            _ => dynamic_ops::get_int_stream(self, input),
        }
    }

    fn create_int_list(&self, input: &[i32]) -> Tag {
        Tag::IntArray(IntArrayTag::new(input.to_vec()))
    }

    fn get_long_stream(&self, input: &Tag) -> DataResult<Vec<i64>> {
        match input {
            Tag::LongArray(array) => DataResult::success(array.get_as_long_array().to_vec()),
            _ => dynamic_ops::get_long_stream(self, input),
        }
    }

    fn create_long_list(&self, input: &[i64]) -> Tag {
        Tag::LongArray(LongArrayTag::new(input.to_vec()))
    }

    fn create_list(&self, input: Vec<Tag>) -> Tag {
        Tag::List(ListTag::from_list(input))
    }

    fn remove(&self, input: &Tag, key: &str) -> Tag {
        match input {
            Tag::Compound(tag) => {
                let mut result = tag.shallow_copy();
                result.remove(key);
                Tag::Compound(result)
            }
            _ => input.clone(),
        }
    }

    fn list_builder(&self) -> Box<dyn ListBuilder<Tag> + '_> {
        Box::new(list_builder::Builder::new(self))
    }

    fn map_builder(&self) -> Box<dyn RecordBuilder<Tag> + '_> {
        Box::new(StringBuilder::new(self, NbtRecordBuilder))
    }
}

impl NbtOps {
    // The rest of `mergeToMap(Tag, MapLike)` and `mergeToMap(Tag, Map)` after the map's check:
    // no entries leave the map as it is (an end becomes the empty map), and keys that aren't
    // strings are left out and named in the error.
    fn merge_entries(&self, map: &Tag, values: Vec<Pair<Tag, Tag>>) -> DataResult<Tag> {
        if values.is_empty() {
            return if let Tag::End(_) = map {
                DataResult::success(self.empty_map())
            } else {
                DataResult::success(map.clone())
            };
        }

        let mut output = NbtOps::copy_or_new(map);
        let mut missed = Vec::new();
        for entry in values {
            let (key, value) = entry.into_parts();
            match key {
                Tag::String(string_key) => {
                    output.put(string_key.value().to_owned(), value);
                }
                key => missed.push(key),
            }
        }
        if !missed.is_empty() {
            let missed = missed
                .iter()
                .map(Tag::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            DataResult::error_partial(
                || format!("some keys are not strings: [{missed}]"),
                Tag::Compound(output),
            )
        } else {
            DataResult::success(Tag::Compound(output))
        }
    }
}

/// `getMap`'s `MapLike` over a compound.
struct CompoundMapLike<'a> {
    tag: &'a CompoundTag,
}

impl MapLike<Tag> for CompoundMapLike<'_> {
    /// The tag under a string key; another key panics, where Java throws
    /// `UnsupportedOperationException`.
    fn get(&self, key: &Tag) -> Option<&Tag> {
        match key {
            Tag::String(string_key) => self.tag.get(string_key.value()),
            key => panic!(
                "UnsupportedOperationException: Cannot get map entry with non-string key: {key}"
            ),
        }
    }

    fn get_str(&self, key: &str) -> Option<&Tag> {
        self.tag.get(key)
    }

    fn entries(&self) -> Vec<Pair<Tag, Tag>> {
        NbtOps::INSTANCE.entries(self.tag)
    }
}

/// `MapLike[{…}]`, the compound's SNBT.
impl fmt::Display for CompoundMapLike<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MapLike[{}]", self.tag)
    }
}

/// `ListCollector`'s four classes: a list, or an array while what's merged is its kind.
enum ListCollector {
    Generic(ListTag),
    Byte(Vec<i8>),
    Int(Vec<i32>),
    Long(Vec<i64>),
}

impl ListCollector {
    // A generic collector of the array's values, each a tag of its kind.
    fn generic_of(values: impl IntoIterator<Item = Tag>) -> ListCollector {
        let mut result = ListTag::new();
        for value in values {
            result.add(value);
        }
        ListCollector::Generic(result)
    }

    fn accept(self, tag: Tag) -> ListCollector {
        match (self, tag) {
            (ListCollector::Generic(mut result), tag) => {
                result.add(tag);
                ListCollector::Generic(result)
            }
            (ListCollector::Byte(mut values), Tag::Byte(byte_tag)) => {
                values.push(byte_tag.value());
                ListCollector::Byte(values)
            }
            (ListCollector::Int(mut values), Tag::Int(int_tag)) => {
                values.push(int_tag.value());
                ListCollector::Int(values)
            }
            (ListCollector::Long(mut values), Tag::Long(long_tag)) => {
                values.push(long_tag.value());
                ListCollector::Long(values)
            }
            (ListCollector::Byte(values), tag) => ListCollector::generic_of(
                values.into_iter().map(|v| Tag::Byte(ByteTag::value_of(v))),
            )
            .accept(tag),
            (ListCollector::Int(values), tag) => {
                ListCollector::generic_of(values.into_iter().map(|v| Tag::Int(IntTag::value_of(v))))
                    .accept(tag)
            }
            (ListCollector::Long(values), tag) => ListCollector::generic_of(
                values.into_iter().map(|v| Tag::Long(LongTag::value_of(v))),
            )
            .accept(tag),
        }
    }

    fn accept_all(self, tags: Vec<Tag>) -> ListCollector {
        let mut collector = self;
        for tag in tags {
            collector = collector.accept(tag);
        }
        collector
    }

    fn result(self) -> Tag {
        match self {
            ListCollector::Generic(result) => Tag::List(result),
            ListCollector::Byte(values) => Tag::ByteArray(ByteArrayTag::new(values)),
            ListCollector::Int(values) => Tag::IntArray(IntArrayTag::new(values)),
            ListCollector::Long(values) => Tag::LongArray(LongArrayTag::new(values)),
        }
    }
}

/// `NbtRecordBuilder`: fields into a compound, merged into a compound prefix's copy.
struct NbtRecordBuilder;

impl StringBuilderOps<Tag> for NbtRecordBuilder {
    type Builder = CompoundTag;

    fn init_builder(&self) -> CompoundTag {
        CompoundTag::new()
    }

    fn append(&self, key: String, value: Tag, mut builder: CompoundTag) -> CompoundTag {
        builder.put(key, value);
        builder
    }

    fn build(&self, builder: CompoundTag, prefix: Tag) -> DataResult<Tag> {
        match prefix {
            Tag::End(_) => DataResult::success(Tag::Compound(builder)),
            Tag::Compound(compound) => {
                let mut result = compound.shallow_copy();
                for (key, value) in builder.entry_set() {
                    result.put(key.clone(), value.clone());
                }
                DataResult::success(Tag::Compound(result))
            }
            prefix => {
                let message = format!("mergeToMap called with not a map: {prefix}");
                DataResult::error_partial(|| message, prefix)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::NbtOps;
    use crate::com::mojang::datafixers::util::pair::Pair;
    use crate::com::mojang::serialization::data_result::tests::{
        decode_result, encode_result, lifecycle,
    };
    use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
    use crate::com::mojang::serialization::map_like::MapLike;
    use crate::golden::{Decode, Encode, Golden};
    use crate::java::lang::number::tests::{decode_boxed, encode_boxed};
    use crate::net::minecraft::nbt::string_tag::StringTag;
    use crate::net::minecraft::nbt::tag::Tag;

    const KEYS: [&str; 7] = ["a", "b", "text", "color", "", "extra", "é"];

    fn tag(tag: &Tag) -> Box<dyn Encode> {
        Box::new(tag.clone())
    }

    fn pairs(pairs: &[Pair<Tag, Tag>]) -> Box<dyn Encode> {
        Box::new(
            pairs
                .iter()
                .map(|pair| (pair.get_first().clone(), pair.get_second().clone()))
                .collect::<Vec<_>>(),
        )
    }

    fn decode_pairs(value: &Value) -> Vec<Pair<Tag, Tag>> {
        Vec::<(Tag, Tag)>::decode(value)
            .into_iter()
            .map(|(key, value)| Pair::new(key, value))
            .collect()
    }

    // Its toString, its entries, and what it holds under each of KEYS, by string and by tag.
    fn map_like(map: &dyn MapLike<Tag>) -> Box<dyn Encode> {
        let mut lookups = Vec::new();
        for key in KEYS {
            lookups.push(map.get_str(key).cloned());
            lookups.push(
                map.get(&Tag::String(StringTag::value_of(key.to_owned())))
                    .cloned(),
            );
        }
        Box::new((map.to_string(), pairs(&map.entries()), lookups))
    }

    fn tag_result(
        value: &Value,
    ) -> crate::com::mojang::serialization::data_result::DataResult<Tag> {
        decode_result(value, Tag::decode)
    }

    // The ops on a list builder, then its two builds.
    fn list_builder_program(ops_spec: Value, prefix: Tag) -> Vec<Box<dyn Encode>> {
        let ops = &NbtOps::INSTANCE;
        let mut builder = ops.list_builder();
        for op in ops_spec.as_array().unwrap() {
            let op = op.as_array().unwrap();
            match op[0].as_str().unwrap() {
                "add" => builder.add(Tag::decode(&op[1])),
                "addResult" => builder.add_result(tag_result(&op[1])),
                "withErrorsFrom" => builder.with_errors_from(tag_result(&op[1]).map(|_| ())),
                _ => builder.map_error(&|message| format!("<{message}>")),
            }
        }
        vec![
            encode_result(&builder.build(prefix), tag),
            encode_result(&builder.build(ops.empty()), tag),
        ]
    }

    // The ops on a record builder, then its two builds.
    fn map_builder_program(ops_spec: Value, prefix: Tag) -> Vec<Box<dyn Encode>> {
        let ops = &NbtOps::INSTANCE;
        let mut builder = ops.map_builder();
        for op in ops_spec.as_array().unwrap() {
            let op = op.as_array().unwrap();
            match op[0].as_str().unwrap() {
                "addString" => builder.add_str(op[1].as_str().unwrap(), Tag::decode(&op[2])),
                "addStringResult" => {
                    builder.add_str_result(op[1].as_str().unwrap(), tag_result(&op[2]))
                }
                "add" => builder.add(Tag::decode(&op[1]), Tag::decode(&op[2])),
                "addResult" => builder.add_result(Tag::decode(&op[1]), tag_result(&op[2])),
                "addResults" => builder.add_results(tag_result(&op[1]), tag_result(&op[2])),
                "withErrorsFrom" => builder.with_errors_from(tag_result(&op[1]).map(|_| ())),
                "setLifecycle" => builder.set_lifecycle(lifecycle(&op[1])),
                _ => builder.map_error(&|message| format!("<{message}>")),
            }
        }
        vec![
            encode_result(&builder.build(prefix), tag),
            encode_result(&builder.build(ops.empty()), tag),
        ]
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.NbtOps");
        let ops = &NbtOps::INSTANCE;
        golden.check("toString()Ljava/lang/String;", || ops.to_string());
        golden.check("empty()Lnet/minecraft/nbt/Tag;", || ops.empty());
        golden.check("emptyList()Lnet/minecraft/nbt/Tag;", || ops.empty_list());
        golden.check("emptyMap()Lnet/minecraft/nbt/Tag;", || ops.empty_map());

        golden.check(
            "getNumberValue(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;",
            |input: Tag| encode_result(&ops.get_number_value(&input), encode_boxed),
        );
        golden.check(
            "getNumberValue(Lnet/minecraft/nbt/Tag;Ljava/lang/Number;)Ljava/lang/Number;",
            |input: Tag, fallback: Value| {
                encode_boxed(&ops.get_number_value_or(&input, decode_boxed(&fallback)))
            },
        );
        golden.check(
            "getBooleanValue(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;",
            |input: Tag| encode_result(&ops.get_boolean_value(&input), |&b| Box::new(b)),
        );
        golden.check(
            "getStringValue(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;",
            |input: Tag| encode_result(&ops.get_string_value(&input), |s| Box::new(s.clone())),
        );
        golden.check(
            "getMapValues(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;",
            |input: Tag| encode_result(&ops.get_map_values(&input), |values| pairs(values)),
        );
        golden.check(
            "getMapEntries(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;",
            |input: Tag| encode_result(&ops.get_map_entries(&input), |values| pairs(values)),
        );
        golden.check(
            "getMap(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;",
            |input: Tag| encode_result(&ops.get_map(&input), |map| map_like(&**map)),
        );
        golden.check(
            "getStream(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;",
            |input: Tag| encode_result(&ops.get_stream(&input), |tags| Box::new(tags.clone())),
        );
        golden.check(
            "getList(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;",
            |input: Tag| encode_result(&ops.get_list(&input), |tags| Box::new(tags.clone())),
        );
        golden.check(
            "getByteBuffer(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;",
            |input: Tag| {
                encode_result(&ops.get_byte_buffer(&input), |bytes| {
                    Box::new(bytes.clone())
                })
            },
        );
        golden.check(
            "getIntStream(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;",
            |input: Tag| encode_result(&ops.get_int_stream(&input), |ints| Box::new(ints.clone())),
        );
        golden.check(
            "getLongStream(Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;",
            |input: Tag| {
                encode_result(&ops.get_long_stream(&input), |longs| {
                    Box::new(longs.clone())
                })
            },
        );
        golden.check(
            "convertTo(Lcom/mojang/serialization/DynamicOps;Lnet/minecraft/nbt/Tag;)Ljava/lang/Object;",
            |input: Tag| Tag::convert_to(ops, &input),
        );
        golden.check(
            "createString(Ljava/lang/String;)Lnet/minecraft/nbt/Tag;",
            |input: Tag| ops.create_string(&input.to_string()),
        );
        golden.check(
            "remove(Lnet/minecraft/nbt/Tag;Ljava/lang/String;)Lnet/minecraft/nbt/Tag;",
            |input: Tag, key: String| ops.remove(&input, &key),
        );
        golden.check(
            "get(Lnet/minecraft/nbt/Tag;Ljava/lang/String;)Lcom/mojang/serialization/DataResult;",
            |input: Tag, key: String| encode_result(&ops.get(&input, &key), tag),
        );
        golden.check(
            "set(Lnet/minecraft/nbt/Tag;Ljava/lang/String;Lnet/minecraft/nbt/Tag;)Lnet/minecraft/nbt/Tag;",
            |input: Tag, value: Tag, key: String| ops.set(&input, &key, value),
        );
        golden.check(
            "mergeToPrimitive(Lnet/minecraft/nbt/Tag;Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;",
            |prefix: Tag, value: Tag| encode_result(&ops.merge_to_primitive(&prefix, value), tag),
        );
        golden.check(
            "getGeneric(Lnet/minecraft/nbt/Tag;Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;",
            |input: Tag, key: Tag| encode_result(&ops.get_generic(&input, &key), tag),
        );

        golden.check(
            "createNumeric(Ljava/lang/Number;)Lnet/minecraft/nbt/Tag;",
            |number: Value| ops.create_numeric(decode_boxed(&number)),
        );
        golden.check("createByte(B)Lnet/minecraft/nbt/Tag;", |value: i8| {
            ops.create_byte(value)
        });
        golden.check("createShort(S)Lnet/minecraft/nbt/Tag;", |value: i16| {
            ops.create_short(value)
        });
        golden.check("createInt(I)Lnet/minecraft/nbt/Tag;", |value: i32| {
            ops.create_int(value)
        });
        golden.check("createLong(J)Lnet/minecraft/nbt/Tag;", |value: i64| {
            ops.create_long(value)
        });
        golden.check("createFloat(F)Lnet/minecraft/nbt/Tag;", |value: f32| {
            ops.create_float(value)
        });
        golden.check("createDouble(D)Lnet/minecraft/nbt/Tag;", |value: f64| {
            ops.create_double(value)
        });
        golden.check("createBoolean(Z)Lnet/minecraft/nbt/Tag;", |value: bool| {
            ops.create_boolean(value)
        });

        golden.check(
            "mergeToList(Lnet/minecraft/nbt/Tag;Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;",
            |list: Tag, value: Tag| encode_result(&ops.merge_to_list(&list, value), tag),
        );
        golden.check(
            "mergeToList(Lnet/minecraft/nbt/Tag;Ljava/util/List;)Lcom/mojang/serialization/DataResult;",
            |list: Tag, values: Vec<Tag>| {
                encode_result(&ops.merge_to_list_values(&list, values), tag)
            },
        );
        golden.check(
            "createList(Ljava/util/stream/Stream;)Lnet/minecraft/nbt/Tag;",
            |values: Vec<Tag>| ops.create_list(values),
        );
        golden.check(
            "mergeToMap(Lnet/minecraft/nbt/Tag;Lnet/minecraft/nbt/Tag;Lnet/minecraft/nbt/Tag;)Lcom/mojang/serialization/DataResult;",
            |map: Tag, key: Tag, value: Tag| encode_result(&ops.merge_to_map(&map, key, value), tag),
        );
        golden.check(
            "mergeToMap(Lnet/minecraft/nbt/Tag;Ljava/util/Map;)Lcom/mojang/serialization/DataResult;",
            |map: Tag, entries: Value| {
                encode_result(&ops.merge_to_map_values(&map, decode_pairs(&entries)), tag)
            },
        );
        golden.check(
            "mergeToMap(Lnet/minecraft/nbt/Tag;Lcom/mojang/serialization/MapLike;)Lcom/mojang/serialization/DataResult;",
            |map: Tag, source: Tag| {
                let values = ops.get_map(&source).get_or_throw();
                encode_result(&ops.merge_to_map_like(&map, &*values), tag)
            },
        );
        golden.check(
            "createMap(Ljava/util/stream/Stream;)Lnet/minecraft/nbt/Tag;",
            |entries: Value| ops.create_map(decode_pairs(&entries)),
        );
        golden.check(
            "createByteList(Ljava/nio/ByteBuffer;)Lnet/minecraft/nbt/Tag;",
            |bytes: Vec<i8>| ops.create_byte_list(&bytes),
        );
        golden.check(
            "createIntList(Ljava/util/stream/IntStream;)Lnet/minecraft/nbt/Tag;",
            |ints: Vec<i32>| ops.create_int_list(&ints),
        );
        golden.check(
            "createLongList(Ljava/util/stream/LongStream;)Lnet/minecraft/nbt/Tag;",
            |longs: Vec<i64>| ops.create_long_list(&longs),
        );

        golden.check("listBuilder programs", list_builder_program);
        golden.check("mapBuilder programs", map_builder_program);
        golden.finish();
    }
}
