//! net/minecraft/nbt/ByteTag.java
//!
//! Not ported yet: `toString` and `accept`, like every tag's (see `Tag`).
//!
//! A record. Java keeps a tag of each byte, which `valueOf` hands out, so a read makes none: the
//! port's tags are values, so it has no cache.

use std::fmt;

use crate::java::io::data_input::DataInput;
use crate::java::io::data_output::DataOutput;
use crate::java::lang::number::Number;
use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
use crate::net::minecraft::nbt::numeric_tag::NumericTag;
use crate::net::minecraft::nbt::string_tag_visitor::StringTagVisitor;
use crate::net::minecraft::nbt::tag::class;
use crate::net::minecraft::nbt::tag_type::TagType;

/// A i8 in NBT.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ByteTag {
    value: i8,
}

impl ByteTag {
    pub const TYPE: TagType = TagType::Byte;
    pub const ZERO: ByteTag = ByteTag { value: 0 };
    pub const ONE: ByteTag = ByteTag { value: 1 };
    const SELF_SIZE_IN_BYTES: i32 = 9;

    /// The record's constructor, which Java deprecates for `valueOf`.
    pub fn new(value: i8) -> ByteTag {
        ByteTag { value }
    }

    pub fn value_of(data: i8) -> ByteTag {
        ByteTag::new(data)
    }

    /// 1 for true, 0 for false.
    pub fn value_of_bool(data: bool) -> ByteTag {
        if data { ByteTag::ONE } else { ByteTag::ZERO }
    }

    pub fn value(&self) -> i8 {
        self.value
    }

    pub fn write(&self, output: &mut dyn DataOutput) {
        output.write_byte(i32::from(self.value));
    }

    pub fn size_in_bytes(&self) -> i32 {
        ByteTag::SELF_SIZE_IN_BYTES
    }

    pub fn get_id(&self) -> i8 {
        1
    }

    pub fn get_type(&self) -> TagType {
        ByteTag::TYPE
    }

    pub fn copy(&self) -> ByteTag {
        *self
    }

    /// `Byte.hashCode`: the byte.
    pub fn hash_code(&self) -> i32 {
        i32::from(self.value)
    }

    /// `TYPE.load`: the i8, accounted as the tag's size.
    pub(super) fn load(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> ByteTag {
        ByteTag::value_of(read_accounted(input, accounter))
    }
}

fn read_accounted(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> i8 {
    accounter.account_bytes(i64::from(ByteTag::SELF_SIZE_IN_BYTES));
    input.read_byte()
}

impl NumericTag for ByteTag {
    fn byte_value(&self) -> i8 {
        self.value
    }

    fn short_value(&self) -> i16 {
        i16::from(self.value)
    }

    fn int_value(&self) -> i32 {
        i32::from(self.value)
    }

    fn long_value(&self) -> i64 {
        i64::from(self.value)
    }

    fn float_value(&self) -> f32 {
        f32::from(self.value)
    }

    fn double_value(&self) -> f64 {
        f64::from(self.value)
    }

    fn r#box(&self) -> Number {
        Number::Byte(self.value)
    }
}

class!(ByteTag, Byte);

/// `toString()`: its SNBT (`StringTagVisitor`).
impl fmt::Display for ByteTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut visitor = StringTagVisitor::new();
        visitor.visit_byte(self);
        f.write_str(&visitor.build())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ByteTag;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::nbt::numeric_tag::NumericTag;
    use crate::net::minecraft::nbt::tag::tests::{
        check_class, encode_payload, matches_payload, payload,
    };

    // [1, value].
    impl Decode for ByteTag {
        fn decode(value: &Value) -> Self {
            ByteTag::new(i8::decode(payload(value, 1)))
        }
    }

    impl Encode for ByteTag {
        fn encode(&self) -> Value {
            encode_payload(1, &self.value)
        }

        fn matches(&self, expected: &Value) -> bool {
            matches_payload(1, &self.value, expected)
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.ByteTag");
        golden.check_method("toString()Ljava/lang/String;", |this: &mut ByteTag| {
            this.to_string()
        });
        for name in [
            "accept(Lnet/minecraft/nbt/StreamTagVisitor;)Lnet/minecraft/nbt/StreamTagVisitor$ValueResult;",
            "accept(Lnet/minecraft/nbt/TagVisitor;)V",
        ] {
            golden.pending(name, "comes with the visitors");
        }
        golden.field("TYPE", ByteTag::TYPE);
        golden.field("ZERO", ByteTag::ZERO);
        golden.field("ONE", ByteTag::ONE);
        golden.check("<init>(B)V", ByteTag::new);
        golden.check("valueOf(B)Lnet/minecraft/nbt/ByteTag;", ByteTag::value_of);
        golden.check(
            "valueOf(Z)Lnet/minecraft/nbt/ByteTag;",
            ByteTag::value_of_bool,
        );
        golden.check_method("value()B", |this: &mut ByteTag| this.value());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ByteTag, other: ByteTag| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut ByteTag| this.hash_code());
        check_class::<ByteTag>(&golden, "copy()Lnet/minecraft/nbt/ByteTag;");
        golden.check_method("byteValue()B", |this: &mut ByteTag| this.byte_value());
        golden.check_method("shortValue()S", |this: &mut ByteTag| this.short_value());
        golden.check_method("intValue()I", |this: &mut ByteTag| this.int_value());
        golden.check_method("longValue()J", |this: &mut ByteTag| this.long_value());
        golden.check_method("floatValue()F", |this: &mut ByteTag| this.float_value());
        golden.check_method("doubleValue()D", |this: &mut ByteTag| this.double_value());
        golden.check_method("box()Ljava/lang/Number;", |this: &mut ByteTag| this.r#box());
        golden.finish();
    }
}
