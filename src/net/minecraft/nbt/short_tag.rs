//! net/minecraft/nbt/ShortTag.java
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

/// A i16 in NBT.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShortTag {
    value: i16,
}

impl ShortTag {
    pub const TYPE: TagType = TagType::Short;
    const SELF_SIZE_IN_BYTES: i32 = 10;

    /// The record's constructor, which Java deprecates for `valueOf`.
    pub fn new(value: i16) -> ShortTag {
        ShortTag { value }
    }

    pub fn value_of(i: i16) -> ShortTag {
        ShortTag::new(i)
    }

    pub fn value(&self) -> i16 {
        self.value
    }

    pub fn write(&self, output: &mut dyn DataOutput) {
        output.write_short(i32::from(self.value));
    }

    pub fn size_in_bytes(&self) -> i32 {
        ShortTag::SELF_SIZE_IN_BYTES
    }

    pub fn get_id(&self) -> i8 {
        2
    }

    pub fn get_type(&self) -> TagType {
        ShortTag::TYPE
    }

    pub fn copy(&self) -> ShortTag {
        *self
    }

    /// `Short.hashCode`: the short.
    pub fn hash_code(&self) -> i32 {
        i32::from(self.value)
    }

    /// `TYPE.load`: the i16, accounted as the tag's size.
    pub(super) fn load(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> ShortTag {
        ShortTag::value_of(read_accounted(input, accounter))
    }
}

fn read_accounted(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> i16 {
    accounter.account_bytes(i64::from(ShortTag::SELF_SIZE_IN_BYTES));
    input.read_short()
}

impl NumericTag for ShortTag {
    fn byte_value(&self) -> i8 {
        (self.value & 0xFF) as i8
    }

    fn short_value(&self) -> i16 {
        self.value
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
        Number::Short(self.value)
    }
}

class!(ShortTag, Short);

/// `toString()`: its SNBT (`StringTagVisitor`).
impl fmt::Display for ShortTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut visitor = StringTagVisitor::new();
        visitor.visit_short(self);
        f.write_str(&visitor.build())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ShortTag;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::nbt::numeric_tag::NumericTag;
    use crate::net::minecraft::nbt::tag::tests::{
        check_class, encode_payload, matches_payload, payload,
    };

    // [2, value].
    impl Decode for ShortTag {
        fn decode(value: &Value) -> Self {
            ShortTag::new(i16::decode(payload(value, 2)))
        }
    }

    impl Encode for ShortTag {
        fn encode(&self) -> Value {
            encode_payload(2, &self.value)
        }

        fn matches(&self, expected: &Value) -> bool {
            matches_payload(2, &self.value, expected)
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.ShortTag");
        golden.check_method("toString()Ljava/lang/String;", |this: &mut ShortTag| {
            this.to_string()
        });
        for name in [
            "accept(Lnet/minecraft/nbt/StreamTagVisitor;)Lnet/minecraft/nbt/StreamTagVisitor$ValueResult;",
            "accept(Lnet/minecraft/nbt/TagVisitor;)V",
        ] {
            golden.pending(name, "comes with the visitors");
        }
        golden.field("TYPE", ShortTag::TYPE);
        golden.check("<init>(S)V", ShortTag::new);
        golden.check("valueOf(S)Lnet/minecraft/nbt/ShortTag;", ShortTag::value_of);
        golden.check_method("value()S", |this: &mut ShortTag| this.value());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ShortTag, other: ShortTag| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut ShortTag| this.hash_code());
        check_class::<ShortTag>(&golden, "copy()Lnet/minecraft/nbt/ShortTag;");
        golden.check_method("byteValue()B", |this: &mut ShortTag| this.byte_value());
        golden.check_method("shortValue()S", |this: &mut ShortTag| this.short_value());
        golden.check_method("intValue()I", |this: &mut ShortTag| this.int_value());
        golden.check_method("longValue()J", |this: &mut ShortTag| this.long_value());
        golden.check_method("floatValue()F", |this: &mut ShortTag| this.float_value());
        golden.check_method("doubleValue()D", |this: &mut ShortTag| this.double_value());
        golden.check_method("box()Ljava/lang/Number;", |this: &mut ShortTag| {
            this.r#box()
        });
        golden.finish();
    }
}
