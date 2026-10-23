//! com/mojang/serialization/DynamicOps.java (DataFixerUpper 10.0.21)
//!
//! How a codec reads and writes one kind of data: NBT's tags (`NbtOps`) or Gson's JSON
//! (`JsonOps`). Java's interface is generic in the data's type `T`; here it's an object-safe trait,
//! so a codec takes `&dyn DynamicOps<T>`, and the methods generic in another ops' type are free
//! functions: `convert_to`, `convert_list` and `convert_map`, which dispatch on the input's kind
//! through `OpsValue`.
//!
//! Java's streams are `Vec`s (`getStream`, `getMapValues`, `createList`, `createMap`), and its
//! consumers of them (`getList`, `getMapEntries`) too: what the game's codecs reach only ever
//! collects them. A `ByteBuffer` is the bytes from its start to its capacity, which is all
//! `createByteList` and `getByteBuffer` read and write. `getMap`'s `MapLike` borrows the input.
//! `withEncoder`, `withDecoder`, `withParser` and the `update` overloads aren't ported: nothing
//! the game's codecs reach calls them. `mapBuilder()`'s default `RecordBuilder.MapBuilder` isn't
//! either, since every ops overrides it: each ops makes its own list and record builders.

use std::fmt;

use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::codec::{BoxedCodec, Codec};
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::list_builder::ListBuilder;
use crate::com::mojang::serialization::map_codec::{BoxedMapCodec, MapCodec};
use crate::com::mojang::serialization::map_like::{self, MapLike};
use crate::com::mojang::serialization::record_builder::RecordBuilder;
use crate::java::lang::number::Number;

/// What a `DynamicOps` reads and writes (Java's `T`): a value that clones, compares and prints
/// (errors print it), and whose kinds `convert_to` knows. Implemented by NBT's `Tag` and Gson's
/// `JsonElement`.
pub trait OpsValue: Clone + PartialEq + fmt::Display + fmt::Debug + 'static {
    /// The value's ops' `convertTo`: the same data written with `out_ops`.
    fn convert_to<U: OpsValue>(out_ops: &dyn DynamicOps<U>, input: &Self) -> U;

    /// A boxed codec's codec for this kind of data.
    fn boxed_codec<A>(codec: &BoxedCodec<A>) -> &(dyn Codec<Self, Value = A> + Send + Sync);

    /// A boxed map codec's map codec for this kind of data.
    fn boxed_map_codec<A>(
        codec: &BoxedMapCodec<A>,
    ) -> &(dyn MapCodec<Self, Value = A> + Send + Sync);
}

/// Which ops an ops is, where Java compares ops by reference (`Dynamic.convert`): the same kind
/// is the same ops, except a `RegistryOps`, which each is its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpsKind {
    Nbt,
    Json,
    JsonCompressed,
    Java,
    Registry,
}

pub trait DynamicOps<T: OpsValue>: fmt::Display {
    /// Which ops this is.
    fn kind(&self) -> OpsKind;

    /// Which ops a delegating ops forwards to (`DelegatingOps.convertTo` compares it), else which
    /// this is.
    fn delegate_kind(&self) -> OpsKind {
        self.kind()
    }

    fn empty(&self) -> T;

    fn empty_map(&self) -> T {
        self.create_map(Vec::new())
    }

    fn empty_list(&self) -> T {
        self.create_list(Vec::new())
    }

    fn get_number_value(&self, input: &T) -> DataResult<Number>;

    /// `getNumberValue(T, Number)`: the number, or the default.
    fn get_number_value_or(&self, input: &T, default_value: Number) -> Number {
        self.get_number_value(input)
            .into_result()
            .unwrap_or(default_value)
    }

    fn create_numeric(&self, i: Number) -> T;

    fn create_byte(&self, value: i8) -> T {
        self.create_numeric(Number::Byte(value))
    }

    fn create_short(&self, value: i16) -> T {
        self.create_numeric(Number::Short(value))
    }

    fn create_int(&self, value: i32) -> T {
        self.create_numeric(Number::Integer(value))
    }

    fn create_long(&self, value: i64) -> T {
        self.create_numeric(Number::Long(value))
    }

    fn create_float(&self, value: f32) -> T {
        self.create_numeric(Number::Float(value))
    }

    fn create_double(&self, value: f64) -> T {
        self.create_numeric(Number::Double(value))
    }

    fn get_boolean_value(&self, input: &T) -> DataResult<bool>;

    fn create_boolean(&self, value: bool) -> T;

    fn get_string_value(&self, input: &T) -> DataResult<String>;

    fn create_string(&self, value: &str) -> T;

    fn merge_to_list(&self, list: &T, value: T) -> DataResult<T>;

    /// `mergeToList(T, List)`: each value merged in turn.
    fn merge_to_list_values(&self, list: &T, values: Vec<T>) -> DataResult<T> {
        let mut result = DataResult::success(list.clone());
        for value in values {
            result = result.flat_map(|r| self.merge_to_list(&r, value));
        }
        result
    }

    fn merge_to_map(&self, map: &T, key: T, value: T) -> DataResult<T>;

    /// `mergeToMap(T, Map)`, the map's entries in its order.
    fn merge_to_map_values(&self, map: &T, values: Vec<Pair<T, T>>) -> DataResult<T> {
        let create_string = Box::new(|key: &str| self.create_string(key));
        self.merge_to_map_like(map, &*map_like::for_map(values, create_string))
    }

    /// `mergeToMap(T, MapLike)`: each entry merged in turn.
    fn merge_to_map_like(&self, map: &T, values: &dyn MapLike<T>) -> DataResult<T> {
        let mut result = DataResult::success(map.clone());
        for entry in values.entries() {
            let (key, value) = entry.into_parts();
            result = result.flat_map(|r| self.merge_to_map(&r, key, value));
        }
        result
    }

    fn merge_to_primitive(&self, prefix: &T, value: T) -> DataResult<T> {
        if *prefix != self.empty() {
            let message =
                format!("Do not know how to append a primitive value {value} to {prefix}");
            DataResult::error_partial(|| message, value)
        } else {
            DataResult::success(value)
        }
    }

    fn get_map_values(&self, input: &T) -> DataResult<Vec<Pair<T, T>>>;

    /// `getMapEntries`: the entries `getMapValues` streams, for a consumer.
    fn get_map_entries(&self, input: &T) -> DataResult<Vec<Pair<T, T>>> {
        self.get_map_values(input)
    }

    fn create_map(&self, map: Vec<Pair<T, T>>) -> T;

    fn get_map<'a>(&self, input: &'a T) -> DataResult<Box<dyn MapLike<T> + 'a>>;

    fn get_stream(&self, input: &T) -> DataResult<Vec<T>>;

    /// `getList`: the elements, for a consumer.
    fn get_list(&self, input: &T) -> DataResult<Vec<T>> {
        self.get_stream(input)
    }

    fn create_list(&self, input: Vec<T>) -> T;

    /// The elements' bytes, if every one is a number.
    fn get_byte_buffer(&self, input: &T) -> DataResult<Vec<i8>> {
        get_byte_buffer(self, input)
    }

    fn create_byte_list(&self, input: &[i8]) -> T {
        self.create_list(input.iter().map(|&b| self.create_byte(b)).collect())
    }

    fn get_int_stream(&self, input: &T) -> DataResult<Vec<i32>> {
        get_int_stream(self, input)
    }

    fn create_int_list(&self, input: &[i32]) -> T {
        self.create_list(input.iter().map(|&i| self.create_int(i)).collect())
    }

    fn get_long_stream(&self, input: &T) -> DataResult<Vec<i64>> {
        get_long_stream(self, input)
    }

    fn create_long_list(&self, input: &[i64]) -> T {
        self.create_list(input.iter().map(|&l| self.create_long(l)).collect())
    }

    fn remove(&self, input: &T, key: &str) -> T;

    fn compress_maps(&self) -> bool {
        false
    }

    fn get(&self, input: &T, key: &str) -> DataResult<T> {
        self.get_generic(input, &self.create_string(key))
    }

    fn get_generic(&self, input: &T, key: &T) -> DataResult<T> {
        self.get_map(input).flat_map(|map| match map.get(key) {
            Some(value) => DataResult::success(value.clone()),
            None => DataResult::error(|| format!("No element {key} in the map {input}")),
        })
    }

    fn set(&self, input: &T, key: &str, value: T) -> T {
        self.merge_to_map(input, self.create_string(key), value)
            .into_result()
            .unwrap_or_else(|| input.clone())
    }

    /// `update(T, String, Function)`: the value under the key, changed by the function.
    fn update(&self, input: &T, key: &str, function: &dyn Fn(T) -> T) -> T {
        self.get(input, key)
            .map(|value| self.set(input, key, function(value)))
            .into_result()
            .unwrap_or_else(|| input.clone())
    }

    fn list_builder(&self) -> Box<dyn ListBuilder<T> + '_>;

    fn map_builder(&self) -> Box<dyn RecordBuilder<T> + '_>;
}

/// `convertTo`: the input, read with its ops, written with `out_ops`.
pub fn convert_to<T: OpsValue, U: OpsValue>(
    _in_ops: &dyn DynamicOps<T>,
    out_ops: &dyn DynamicOps<U>,
    input: &T,
) -> U {
    T::convert_to(out_ops, input)
}

/// `convertList`: each element converted, as a list of `out_ops`.
pub fn convert_list<T: OpsValue, U: OpsValue>(
    in_ops: &dyn DynamicOps<T>,
    out_ops: &dyn DynamicOps<U>,
    input: &T,
) -> U {
    out_ops.create_list(
        in_ops
            .get_stream(input)
            .into_result()
            .unwrap_or_default()
            .iter()
            .map(|element| T::convert_to(out_ops, element))
            .collect(),
    )
}

/// `convertMap`: each key and value converted, as a map of `out_ops`.
pub fn convert_map<T: OpsValue, U: OpsValue>(
    in_ops: &dyn DynamicOps<T>,
    out_ops: &dyn DynamicOps<U>,
    input: &T,
) -> U {
    out_ops.create_map(
        in_ops
            .get_map_values(input)
            .into_result()
            .unwrap_or_default()
            .into_iter()
            .map(|entry| {
                Pair::new(
                    T::convert_to(out_ops, entry.get_first()),
                    T::convert_to(out_ops, entry.get_second()),
                )
            })
            .collect(),
    )
}

/// `getByteBuffer`'s default, which an ops overriding it falls back on: the elements' bytes, if
/// every one is a number.
pub fn get_byte_buffer<T: OpsValue, O: DynamicOps<T> + ?Sized>(
    ops: &O,
    input: &T,
) -> DataResult<Vec<i8>> {
    ops.get_stream(input).flat_map(|list| {
        if !list
            .iter()
            .all(|element| ops.get_number_value(element).is_success())
        {
            return DataResult::error(|| format!("Some elements are not bytes: {input}"));
        }
        DataResult::success(
            list.iter()
                .map(|element| ops.get_number_value(element).get_or_throw().byte_value())
                .collect(),
        )
    })
}

/// `getIntStream`'s default: the elements' ints, if every one is a number.
pub fn get_int_stream<T: OpsValue, O: DynamicOps<T> + ?Sized>(
    ops: &O,
    input: &T,
) -> DataResult<Vec<i32>> {
    ops.get_stream(input).flat_map(|list| {
        if list
            .iter()
            .all(|element| ops.get_number_value(element).is_success())
        {
            DataResult::success(
                list.iter()
                    .map(|element| ops.get_number_value(element).get_or_throw().int_value())
                    .collect(),
            )
        } else {
            DataResult::error(|| format!("Some elements are not ints: {input}"))
        }
    })
}

/// `getLongStream`'s default: the elements' longs, if every one is a number.
pub fn get_long_stream<T: OpsValue, O: DynamicOps<T> + ?Sized>(
    ops: &O,
    input: &T,
) -> DataResult<Vec<i64>> {
    ops.get_stream(input).flat_map(|list| {
        if list
            .iter()
            .all(|element| ops.get_number_value(element).is_success())
        {
            DataResult::success(
                list.iter()
                    .map(|element| ops.get_number_value(element).get_or_throw().long_value())
                    .collect(),
            )
        } else {
            DataResult::error(|| format!("Some elements are not longs: {input}"))
        }
    })
}
