//! com/mojang/serialization/JavaOps.java (DataFixerUpper 10.0.21)
//!
//! `DynamicOps` over plain Java objects, which `ExtraCodecs.JAVA` converts a translation's
//! arguments to. Java's `Object` is `JavaValue`, of the kinds `convertTo` knows. Only the side the
//! game's codecs reach is ported: making values (`create…`), which other ops' `convertTo` calls
//! when converting to Java objects, and `convertTo` itself. Reading Java objects back (`get…`,
//! the merges, the builders) isn't reached by any codec the port has, and panics.

use std::fmt;

use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::codec::{BoxedCodec, Codec};
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{self, DynamicOps, OpsKind, OpsValue};
use crate::com::mojang::serialization::list_builder::ListBuilder;
use crate::com::mojang::serialization::map_codec::{BoxedMapCodec, MapCodec};
use crate::com::mojang::serialization::map_like::MapLike;
use crate::com::mojang::serialization::record_builder::RecordBuilder;
use crate::java::lang::number::Number;

/// A Java object as `JavaOps` makes one: `null`, a boolean, a number's box, a string, a list, a
/// map, or fastutil's primitive lists.
#[derive(Clone, Debug, PartialEq)]
pub enum JavaValue {
    Null,
    Boolean(bool),
    Number(Number),
    String(String),
    List(Vec<JavaValue>),
    Map(Vec<(JavaValue, JavaValue)>),
    ByteList(Vec<i8>),
    IntList(Vec<i32>),
    LongList(Vec<i64>),
}

pub struct JavaOps;

impl JavaOps {
    pub const INSTANCE: JavaOps = JavaOps;
}

/// `JavaOps`.
impl fmt::Display for JavaOps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Java")
    }
}

/// Java's `toString` of each kind, as far as it's ported: the boxes', a string itself, a list's
/// `[a, b]`.
impl fmt::Display for JavaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JavaValue::Null => f.write_str("null"),
            JavaValue::Boolean(value) => write!(f, "{value}"),
            JavaValue::Number(value) => write!(f, "{value}"),
            JavaValue::String(value) => f.write_str(value),
            JavaValue::List(values) => {
                let values: Vec<String> = values.iter().map(ToString::to_string).collect();
                write!(f, "[{}]", values.join(", "))
            }
            JavaValue::Map(entries) => {
                let entries: Vec<String> = entries
                    .iter()
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect();
                write!(f, "{{{}}}", entries.join(", "))
            }
            JavaValue::ByteList(values) => write!(f, "{values:?}"),
            JavaValue::IntList(values) => write!(f, "{values:?}"),
            JavaValue::LongList(values) => write!(f, "{values:?}"),
        }
    }
}

impl OpsValue for JavaValue {
    /// `JavaOps.convertTo`: each kind as the other ops' kind of value.
    fn convert_to<U: OpsValue>(out_ops: &dyn DynamicOps<U>, input: &JavaValue) -> U {
        match input {
            JavaValue::Null => out_ops.empty(),
            JavaValue::Map(_) => dynamic_ops::convert_map(&JavaOps::INSTANCE, out_ops, input),
            JavaValue::ByteList(values) => out_ops.create_byte_list(values),
            JavaValue::IntList(values) => out_ops.create_int_list(values),
            JavaValue::LongList(values) => out_ops.create_long_list(values),
            JavaValue::List(_) => dynamic_ops::convert_list(&JavaOps::INSTANCE, out_ops, input),
            JavaValue::String(value) => out_ops.create_string(value),
            JavaValue::Boolean(value) => out_ops.create_boolean(*value),
            JavaValue::Number(Number::Byte(value)) => out_ops.create_byte(*value),
            JavaValue::Number(Number::Short(value)) => out_ops.create_short(*value),
            JavaValue::Number(Number::Integer(value)) => out_ops.create_int(*value),
            JavaValue::Number(Number::Long(value)) => out_ops.create_long(*value),
            JavaValue::Number(Number::Float(value)) => out_ops.create_float(*value),
            JavaValue::Number(Number::Double(value)) => out_ops.create_double(*value),
            JavaValue::Number(other) => out_ops.create_numeric(other.clone()),
        }
    }

    fn boxed_codec<A>(_codec: &BoxedCodec<A>) -> &(dyn Codec<JavaValue, Value = A> + Send + Sync) {
        panic!("no codec the port has reads Java objects")
    }

    fn boxed_map_codec<A>(
        _codec: &BoxedMapCodec<A>,
    ) -> &(dyn MapCodec<JavaValue, Value = A> + Send + Sync) {
        panic!("no codec the port has reads Java objects")
    }
}

// What no codec the port has reaches.
fn unreached() -> ! {
    panic!("JavaOps only makes values here: nothing reads Java objects back")
}

impl DynamicOps<JavaValue> for JavaOps {
    fn kind(&self) -> OpsKind {
        OpsKind::Java
    }

    fn empty(&self) -> JavaValue {
        JavaValue::Null
    }

    fn empty_map(&self) -> JavaValue {
        JavaValue::Map(Vec::new())
    }

    fn empty_list(&self) -> JavaValue {
        JavaValue::List(Vec::new())
    }

    fn get_number_value(&self, input: &JavaValue) -> DataResult<Number> {
        match input {
            JavaValue::Number(value) => DataResult::success(value.clone()),
            _ => DataResult::error(|| format!("Not a number: {input}")),
        }
    }

    fn create_numeric(&self, i: Number) -> JavaValue {
        JavaValue::Number(i)
    }

    fn get_boolean_value(&self, input: &JavaValue) -> DataResult<bool> {
        match input {
            JavaValue::Boolean(value) => DataResult::success(*value),
            _ => DataResult::error(|| format!("Not a boolean: {input}")),
        }
    }

    fn create_boolean(&self, value: bool) -> JavaValue {
        JavaValue::Boolean(value)
    }

    fn get_string_value(&self, input: &JavaValue) -> DataResult<String> {
        match input {
            JavaValue::String(value) => DataResult::success(value.clone()),
            _ => DataResult::error(|| format!("Not a string: {input}")),
        }
    }

    fn create_string(&self, value: &str) -> JavaValue {
        JavaValue::String(value.to_owned())
    }

    fn merge_to_list(&self, _list: &JavaValue, _value: JavaValue) -> DataResult<JavaValue> {
        unreached()
    }

    fn merge_to_map(
        &self,
        _map: &JavaValue,
        _key: JavaValue,
        _value: JavaValue,
    ) -> DataResult<JavaValue> {
        unreached()
    }

    fn get_map_values(&self, _input: &JavaValue) -> DataResult<Vec<Pair<JavaValue, JavaValue>>> {
        unreached()
    }

    /// `ImmutableMap.toImmutableMap`'s entries, in order. A null value (an `EndTag` converted)
    /// panics, where Java's map throws `NullPointerException`.
    fn create_map(&self, map: Vec<Pair<JavaValue, JavaValue>>) -> JavaValue {
        JavaValue::Map(
            map.into_iter()
                .map(|entry| {
                    let (key, value) = entry.into_parts();
                    if value == JavaValue::Null {
                        panic!("NullPointerException: null value in entry: {key}=null");
                    }
                    (key, value)
                })
                .collect(),
        )
    }

    fn get_map<'a>(&self, _input: &'a JavaValue) -> DataResult<Box<dyn MapLike<JavaValue> + 'a>> {
        unreached()
    }

    fn get_stream(&self, _input: &JavaValue) -> DataResult<Vec<JavaValue>> {
        unreached()
    }

    fn create_list(&self, input: Vec<JavaValue>) -> JavaValue {
        JavaValue::List(input)
    }

    fn create_byte_list(&self, input: &[i8]) -> JavaValue {
        JavaValue::ByteList(input.to_vec())
    }

    fn create_int_list(&self, input: &[i32]) -> JavaValue {
        JavaValue::IntList(input.to_vec())
    }

    fn create_long_list(&self, input: &[i64]) -> JavaValue {
        JavaValue::LongList(input.to_vec())
    }

    fn remove(&self, _input: &JavaValue, _key: &str) -> JavaValue {
        unreached()
    }

    fn list_builder(&self) -> Box<dyn ListBuilder<JavaValue> + '_> {
        unreached()
    }

    fn map_builder(&self) -> Box<dyn RecordBuilder<JavaValue> + '_> {
        unreached()
    }
}
