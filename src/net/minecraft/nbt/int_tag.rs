//! net/minecraft/nbt/IntTag.java
//!
//! Not ported yet: `toString` and `accept`, like every tag's (see `Tag`).
//!
//! A record. Java keeps tags of -128 to 1024, which `valueOf` hands out, so a read makes none of
//! those: the port's tags are values, so it has no cache.

use std::fmt;

use crate::java::io::data_input::DataInput;
use crate::java::io::data_output::DataOutput;
use crate::java::lang::number::Number;
use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
use crate::net::minecraft::nbt::numeric_tag::NumericTag;
use crate::net::minecraft::nbt::string_tag_visitor::StringTagVisitor;
use crate::net::minecraft::nbt::tag::class;
use crate::net::minecraft::nbt::tag_type::TagType;

/// A i32 in NBT.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntTag {
    value: i32,
}

impl IntTag {
    pub const TYPE: TagType = TagType::Int;
    const SELF_SIZE_IN_BYTES: i32 = 12;

    /// The record's constructor, which Java deprecates for `valueOf`.
    pub fn new(value: i32) -> IntTag {
        IntTag { value }
    }

    pub fn value_of(i: i32) -> IntTag {
        IntTag::new(i)
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn write(&self, output: &mut dyn DataOutput) {
        output.write_int(self.value);
    }

    pub fn size_in_bytes(&self) -> i32 {
        IntTag::SELF_SIZE_IN_BYTES
    }

    pub fn get_id(&self) -> i8 {
        3
    }

    pub fn get_type(&self) -> TagType {
        IntTag::TYPE
    }

    pub fn copy(&self) -> IntTag {
        *self
    }

    /// `Integer.hashCode`: the int.
    pub fn hash_code(&self) -> i32 {
        self.value
    }

    /// `TYPE.load`: the i32, accounted as the tag's size.
    pub(super) fn load(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> IntTag {
        IntTag::value_of(read_accounted(input, accounter))
    }
}

fn read_accounted(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> i32 {
    accounter.account_bytes(i64::from(IntTag::SELF_SIZE_IN_BYTES));
    input.read_int()
}

impl NumericTag for IntTag {
    fn byte_value(&self) -> i8 {
        (self.value & 0xFF) as i8
    }

    fn short_value(&self) -> i16 {
        (self.value & 65535) as i16
    }

    fn int_value(&self) -> i32 {
        self.value
    }

    fn long_value(&self) -> i64 {
        i64::from(self.value)
    }

    fn float_value(&self) -> f32 {
        self.value as f32
    }

    fn double_value(&self) -> f64 {
        f64::from(self.value)
    }

    fn r#box(&self) -> Number {
        Number::Integer(self.value)
    }
}

class!(IntTag, Int);

/// `toString()`: its SNBT (`StringTagVisitor`).
impl fmt::Display for IntTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut visitor = StringTagVisitor::new();
        visitor.visit_int(self);
        f.write_str(&visitor.build())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::IntTag;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::nbt::numeric_tag::NumericTag;
    use crate::net::minecraft::nbt::tag::tests::{
        check_class, encode_payload, matches_payload, payload,
    };

    // [3, value].
    impl Decode for IntTag {
        fn decode(value: &Value) -> Self {
            IntTag::new(i32::decode(payload(value, 3)))
        }
    }

    impl Encode for IntTag {
        fn encode(&self) -> Value {
            encode_payload(3, &self.value)
        }

        fn matches(&self, expected: &Value) -> bool {
            matches_payload(3, &self.value, expected)
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.IntTag");
        golden.check_method("toString()Ljava/lang/String;", |this: &mut IntTag| {
            this.to_string()
        });
        for name in [
            "accept(Lnet/minecraft/nbt/StreamTagVisitor;)Lnet/minecraft/nbt/StreamTagVisitor$ValueResult;",
            "accept(Lnet/minecraft/nbt/TagVisitor;)V",
        ] {
            golden.pending(name, "comes with the visitors");
        }
        golden.field("TYPE", IntTag::TYPE);
        golden.check("<init>(I)V", IntTag::new);
        golden.check("valueOf(I)Lnet/minecraft/nbt/IntTag;", IntTag::value_of);
        golden.check_method("value()I", |this: &mut IntTag| this.value());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut IntTag, other: IntTag| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut IntTag| this.hash_code());
        check_class::<IntTag>(&golden, "copy()Lnet/minecraft/nbt/IntTag;");
        golden.check_method("byteValue()B", |this: &mut IntTag| this.byte_value());
        golden.check_method("shortValue()S", |this: &mut IntTag| this.short_value());
        golden.check_method("intValue()I", |this: &mut IntTag| this.int_value());
        golden.check_method("longValue()J", |this: &mut IntTag| this.long_value());
        golden.check_method("floatValue()F", |this: &mut IntTag| this.float_value());
        golden.check_method("doubleValue()D", |this: &mut IntTag| this.double_value());
        golden.check_method("box()Ljava/lang/Number;", |this: &mut IntTag| this.r#box());
        golden.finish();
    }
}
