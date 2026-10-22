//! net/minecraft/world/level/block/state/properties/Property.java
//!
//! Not ported yet: `codec`, `valueCodec` and `parseValue`, which wait for DataFixerUpper's codecs,
//! and `Property.Value` (`value`, `getAllValues`), which only they use. `hashCode` hashes the value
//! class's identity hash, which changes from run to run, so it isn't ported.
//!
//! Java's `Property<T>` is two traits: `Property` is `Property<?>`, what a state definition holds
//! whatever the values, with the values as `Comparable`s, and `TypedProperty` adds the type, so a
//! state's `get_value(BlockStateProperties::FACING)` is a `Direction`. A state definition and a
//! state compare properties by reference, like Java: each property is a `static`, and two are the
//! same when their addresses are ([`same`]).

use std::any::Any;
use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;

/// A property's value whatever its type, Java's `Comparable<?>` in a state's values: a `Boolean`,
/// an `Integer` or an enum constant.
#[derive(Clone, Copy)]
pub enum Comparable {
    Boolean(bool),
    Integer(i32),
    Enum(&'static dyn EnumConstant),
}

/// An enum constant whatever its enum, as `Comparable` holds it.
pub trait EnumConstant: Send + Sync + 'static {
    /// `getDeclaringClass().getName()`.
    fn class_name(&self) -> &'static str;
    fn ordinal(&self) -> i32;
    /// `name()`: the constant's name in the source, `NORTH`.
    fn name(&self) -> String;
    /// `toString()`, which some enums make their serialized name.
    fn to_java_string(&self) -> String;
    fn serialized_name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
}

/// An enum a property can take: `Enum<T> & StringRepresentable` in Java, with its class's name
/// for `getValueClass` and its `toString` as `Display`.
pub trait PropertyEnum:
    Enum + StringRepresentable + fmt::Display + fmt::Debug + PartialEq + Send + Sync
{
    /// The class's binary name, `net.minecraft.core.Direction`.
    const CLASS: &'static str;
}

impl<T: PropertyEnum> EnumConstant for T {
    fn class_name(&self) -> &'static str {
        T::CLASS
    }

    fn ordinal(&self) -> i32 {
        Enum::ordinal(self)
    }

    fn name(&self) -> String {
        format!("{self:?}")
    }

    fn to_java_string(&self) -> String {
        self.to_string()
    }

    fn serialized_name(&self) -> &'static str {
        self.get_serialized_name()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Comparable {
    /// The enum constant this is, when it's one of `T`'s.
    pub fn as_enum<T: PropertyEnum>(&self) -> Option<T> {
        match self {
            Comparable::Enum(constant) => constant.as_any().downcast_ref::<T>().copied(),
            _ => None,
        }
    }
}

/// `equals`: values of different classes are never equal, and enum constants are compared by
/// their class and ordinal, which is comparing references.
impl PartialEq for Comparable {
    fn eq(&self, other: &Comparable) -> bool {
        match (self, other) {
            (Comparable::Boolean(a), Comparable::Boolean(b)) => a == b,
            (Comparable::Integer(a), Comparable::Integer(b)) => a == b,
            (Comparable::Enum(a), Comparable::Enum(b)) => {
                a.class_name() == b.class_name() && a.ordinal() == b.ordinal()
            }
            _ => false,
        }
    }
}

impl Eq for Comparable {}

/// `toString()`: `true`, `7`, or the constant's own `toString`.
impl fmt::Display for Comparable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Comparable::Boolean(value) => write!(f, "{value}"),
            Comparable::Integer(value) => write!(f, "{value}"),
            Comparable::Enum(constant) => f.write_str(&constant.to_java_string()),
        }
    }
}

impl fmt::Debug for Comparable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Comparable::Enum(constant) => f.write_str(&constant.name()),
            _ => fmt::Display::fmt(self, f),
        }
    }
}

/// A type a property's values have, which converts to and from `Comparable`.
pub trait PropertyValue: Copy + 'static {
    fn to_comparable(self) -> Comparable;

    /// Panics on a value of another type, where Java's cast throws `ClassCastException`.
    fn from_comparable(value: Comparable) -> Self;
}

impl PropertyValue for bool {
    fn to_comparable(self) -> Comparable {
        Comparable::Boolean(self)
    }

    fn from_comparable(value: Comparable) -> bool {
        match value {
            Comparable::Boolean(value) => value,
            other => panic!("ClassCastException: {other:?} isn't a Boolean"),
        }
    }
}

impl PropertyValue for i32 {
    fn to_comparable(self) -> Comparable {
        Comparable::Integer(self)
    }

    fn from_comparable(value: Comparable) -> i32 {
        match value {
            Comparable::Integer(value) => value,
            other => panic!("ClassCastException: {other:?} isn't an Integer"),
        }
    }
}

impl<T: PropertyEnum> PropertyValue for T {
    fn to_comparable(self) -> Comparable {
        Comparable::Enum(&T::ENUM_CONSTANTS[Enum::ordinal(&self) as usize])
    }

    fn from_comparable(value: Comparable) -> T {
        value
            .as_enum()
            .unwrap_or_else(|| panic!("ClassCastException: {value:?} isn't a {}", T::CLASS))
    }
}

/// `Property<?>`: a named set of values a block's states vary in, `facing` or `waterlogged`.
pub trait Property: fmt::Display + Send + Sync + 'static {
    fn get_name(&self) -> &'static str;

    /// `getValueClass()`: the values' class's binary name, `java.lang.Boolean`.
    fn get_value_class(&self) -> &'static str;

    /// `getPossibleValues()`, in the order a state definition numbers them.
    fn get_possible_values(&self) -> Vec<Comparable>;

    /// `getPossibleValues().size()`.
    fn possible_value_count(&self) -> usize;

    /// `getPossibleValues().get(index)`.
    fn possible_value(&self, index: usize) -> Comparable;

    /// `getName(T)`: the name a value is written by. Panics on a value of another type, where
    /// Java throws `ClassCastException`.
    fn get_name_of(&self, value: Comparable) -> String;

    /// `getValue(String)`: the value written by the name, if it's one of this property's.
    fn get_value(&self, name: &str) -> Option<Comparable>;

    /// `getInternalIndex(T)`: the value's place among the possible values, or -1. Panics on a
    /// value of another type.
    fn get_internal_index(&self, value: Comparable) -> i32;

    /// `equals(Object)`: the same class of values and name, and for an integer or an enum
    /// property the same values.
    fn equals(&self, other: &dyn Property) -> bool;

    /// `Enum.valueOf(getValueClass(), name)` for an enum property: any constant of the values'
    /// class, among the property's values or not. None for a boolean or integer property, or a
    /// name no constant has.
    fn value_of_constant(&self, name: &str) -> Option<Comparable>;

    fn as_any(&self) -> &dyn Any;
}

/// `Property<T>`: what takes or gives a `T`.
pub trait TypedProperty: Property {
    type Value: PropertyValue;
}

/// Whether two properties are the same object, which is how states look them up.
pub fn same(a: &dyn Property, b: &dyn Property) -> bool {
    std::ptr::addr_eq(a, b)
}

/// `toString()`: `MoreObjects.toStringHelper` of the name, the value class and the values, under
/// the property's own class name.
pub(crate) fn to_string_helper(
    f: &mut fmt::Formatter<'_>,
    class: &str,
    property: &dyn Property,
) -> fmt::Result {
    let values: Vec<String> = property
        .get_possible_values()
        .iter()
        .map(Comparable::to_string)
        .collect();
    write!(
        f,
        "{class}{{name={}, clazz=class {}, values=[{}]}}",
        property.get_name(),
        property.get_value_class(),
        values.join(", ")
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::Value;

    use super::{Comparable, Property};
    use crate::net::minecraft::core::direction::Direction;
    use crate::golden::{Encode, Golden};
    use crate::net::minecraft::world::level::block::state::properties::block_state_properties::tests::property;
    use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
    use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
    use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;

    /// A boolean, an int or the constant's name, the way vanilla encodes a value.
    impl Encode for Comparable {
        fn encode(&self) -> Value {
            match self {
                Comparable::Boolean(value) => Value::from(*value),
                Comparable::Integer(value) => Value::from(*value),
                Comparable::Enum(constant) => Value::from(constant.name()),
            }
        }
    }

    /// The property's value encoded as vanilla does: a boolean, an int, or the name of a constant
    /// of its enum, which needn't be one of its values.
    pub(crate) fn value(property: &dyn Property, value: &Value) -> Comparable {
        match value {
            Value::Bool(value) => Comparable::Boolean(*value),
            Value::Number(number) => Comparable::Integer(
                number
                    .as_i64()
                    .and_then(|n| i32::try_from(n).ok())
                    .expect("an int"),
            ),
            Value::String(name) => property
                .value_of_constant(name)
                .unwrap_or_else(|| panic!("{name} isn't a constant of {property}'s values")),
            other => panic!("expected a property value, got {other}"),
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.block.state.properties.Property");
        golden.check("getName()Ljava/lang/String;", |field: String| {
            property(&field).get_name().to_owned()
        });
        golden.check("getPossibleValues()Ljava/util/List;", |field: String| {
            property(&field).get_possible_values()
        });
        golden.check("getValueClass()Ljava/lang/Class;", |field: String| {
            property(&field).get_value_class().to_owned()
        });
        golden.check("toString()Ljava/lang/String;", |field: String| {
            property(&field).to_string()
        });
        golden.check(
            "getName(Ljava/lang/Comparable;)Ljava/lang/String;",
            |field: String, encoded: Value| {
                let property = property(&field);
                property.get_name_of(value(property, &encoded))
            },
        );
        golden.check(
            "getInternalIndex(Ljava/lang/Comparable;)I",
            |field: String, encoded: Value| {
                let property = property(&field);
                property.get_internal_index(value(property, &encoded))
            },
        );
        golden.check(
            "getValue(Ljava/lang/String;)Ljava/util/Optional;",
            |field: String, name: String| property(&field).get_value(&name),
        );
        golden.check(
            "equals(Ljava/lang/Object;)Z",
            |field: String, other: String| property(&field).equals(property(&other)),
        );
        golden.pending(
            "hashCode()I",
            "hashes its value class's identity hash, which changes from run to run",
        );
        golden.pending(
            "codec()Lcom/mojang/serialization/Codec;",
            "waits for DataFixerUpper's codecs",
        );
        golden.pending(
            "valueCodec()Lcom/mojang/serialization/Codec;",
            "waits for DataFixerUpper's codecs",
        );
        golden.pending(
            "parseValue(Lcom/mojang/serialization/DynamicOps;Lnet/minecraft/world/level/block/state/StateHolder;Ljava/lang/Object;)Lcom/mojang/serialization/DataResult;",
            "waits for DataFixerUpper's codecs",
        );
        golden.pending(
            "value(Ljava/lang/Comparable;)Lnet/minecraft/world/level/block/state/properties/Property$Value;",
            "Property.Value only exists for the codecs so far",
        );
        golden.pending(
            "value(Lnet/minecraft/world/level/block/state/StateHolder;)Lnet/minecraft/world/level/block/state/properties/Property$Value;",
            "Property.Value only exists for the codecs so far",
        );
        golden.pending(
            "getAllValues()Ljava/util/stream/Stream;",
            "Property.Value only exists for the codecs so far",
        );
        golden.check(
            "IntegerProperty.create(Ljava/lang/String;II)Lnet/minecraft/world/level/block/state/properties/IntegerProperty;",
            |name: String, min: i32, max: i32| {
                let created = IntegerProperty::create(Box::leak(name.into_boxed_str()), min, max);
                (created.get_name().to_owned(), created.possible_value_count() as i32)
            },
        );
        golden.check(
            "EnumProperty.create(Ljava/lang/String;Ljava/lang/Class;Ljava/util/List;)Lnet/minecraft/world/level/block/state/properties/EnumProperty;",
            |name: String, values: Vec<String>| {
                assert!(values.is_empty(), "the case makes an empty one");
                let created = EnumProperty::<Direction>::create(Box::leak(name.into_boxed_str()), &[]);
                created.get_name().to_owned()
            },
        );
        golden.check(
            "BooleanProperty.create(Ljava/lang/String;)Lnet/minecraft/world/level/block/state/properties/BooleanProperty;",
            |name: String| BooleanProperty::create(Box::leak(name.into_boxed_str())).get_possible_values(),
        );
        golden.finish();
    }
}
