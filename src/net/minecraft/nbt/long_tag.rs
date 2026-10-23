//! net/minecraft/nbt/LongTag.java
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

/// A i64 in NBT.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LongTag {
    value: i64,
}

impl LongTag {
    pub const TYPE: TagType = TagType::Long;
    const SELF_SIZE_IN_BYTES: i32 = 16;

    /// The record's constructor, which Java deprecates for `valueOf`.
    pub fn new(value: i64) -> LongTag {
        LongTag { value }
    }

    pub fn value_of(i: i64) -> LongTag {
        LongTag::new(i)
    }

    pub fn value(&self) -> i64 {
        self.value
    }

    pub fn write(&self, output: &mut dyn DataOutput) {
        output.write_long(self.value);
    }

    pub fn size_in_bytes(&self) -> i32 {
        LongTag::SELF_SIZE_IN_BYTES
    }

    pub fn get_id(&self) -> i8 {
        4
    }

    pub fn get_type(&self) -> TagType {
        LongTag::TYPE
    }

    pub fn copy(&self) -> LongTag {
        *self
    }

    /// `Long.hashCode`: its two halves, XORed.
    pub fn hash_code(&self) -> i32 {
        (self.value ^ ((self.value as u64) >> 32) as i64) as i32
    }

    /// `TYPE.load`: the i64, accounted as the tag's size.
    pub(super) fn load(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> LongTag {
        LongTag::value_of(read_accounted(input, accounter))
    }
}

fn read_accounted(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> i64 {
    accounter.account_bytes(i64::from(LongTag::SELF_SIZE_IN_BYTES));
    input.read_long()
}

impl NumericTag for LongTag {
    fn byte_value(&self) -> i8 {
        (self.value & 255) as i8
    }

    fn short_value(&self) -> i16 {
        (self.value & 65535) as i16
    }

    fn int_value(&self) -> i32 {
        self.value as i32
    }

    fn long_value(&self) -> i64 {
        self.value
    }

    fn float_value(&self) -> f32 {
        self.value as f32
    }

    fn double_value(&self) -> f64 {
        self.value as f64
    }

    fn r#box(&self) -> Number {
        Number::Long(self.value)
    }
}

class!(LongTag, Long);

/// `toString()`: its SNBT (`StringTagVisitor`).
impl fmt::Display for LongTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut visitor = StringTagVisitor::new();
        visitor.visit_long(self);
        f.write_str(&visitor.build())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::LongTag;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::nbt::numeric_tag::NumericTag;
    use crate::net::minecraft::nbt::tag::tests::{
        check_class, encode_payload, matches_payload, payload,
    };

    // [4, value].
    impl Decode for LongTag {
        fn decode(value: &Value) -> Self {
            LongTag::new(i64::decode(payload(value, 4)))
        }
    }

    impl Encode for LongTag {
        fn encode(&self) -> Value {
            encode_payload(4, &self.value)
        }

        fn matches(&self, expected: &Value) -> bool {
            matches_payload(4, &self.value, expected)
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.LongTag");
        golden.check_method("toString()Ljava/lang/String;", |this: &mut LongTag| {
            this.to_string()
        });
        for name in [
            "accept(Lnet/minecraft/nbt/StreamTagVisitor;)Lnet/minecraft/nbt/StreamTagVisitor$ValueResult;",
            "accept(Lnet/minecraft/nbt/TagVisitor;)V",
        ] {
            golden.pending(name, "comes with the visitors");
        }
        golden.field("TYPE", LongTag::TYPE);
        golden.check("<init>(J)V", LongTag::new);
        golden.check("valueOf(J)Lnet/minecraft/nbt/LongTag;", LongTag::value_of);
        golden.check_method("value()J", |this: &mut LongTag| this.value());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut LongTag, other: LongTag| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut LongTag| this.hash_code());
        check_class::<LongTag>(&golden, "copy()Lnet/minecraft/nbt/LongTag;");
        golden.check_method("byteValue()B", |this: &mut LongTag| this.byte_value());
        golden.check_method("shortValue()S", |this: &mut LongTag| this.short_value());
        golden.check_method("intValue()I", |this: &mut LongTag| this.int_value());
        golden.check_method("longValue()J", |this: &mut LongTag| this.long_value());
        golden.check_method("floatValue()F", |this: &mut LongTag| this.float_value());
        golden.check_method("doubleValue()D", |this: &mut LongTag| this.double_value());
        golden.check_method("box()Ljava/lang/Number;", |this: &mut LongTag| this.r#box());
        golden.finish();
    }
}
