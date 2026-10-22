//! net/minecraft/nbt/FloatTag.java
//!
//! Not ported yet: `toString` and `accept`, like every tag's (see `Tag`).
//!
//! A record, which `equals` with `Float.compare`: -0.0 isn't 0.0, and NaN is NaN. `valueOf` makes
//! both zeros the `ZERO` constant, which is 0.0, so a -0.0 read from NBT comes back as 0.0.

use crate::java::io::data_input::DataInput;
use crate::java::io::data_output::DataOutput;
use crate::java::lang::float::Float;
use crate::java::lang::number::Number;
use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
use crate::net::minecraft::nbt::numeric_tag::NumericTag;
use crate::net::minecraft::nbt::tag::class;
use crate::net::minecraft::nbt::tag_type::TagType;
use crate::net::minecraft::util::mth::Mth;

/// A f32 in NBT.
#[derive(Clone, Copy, Debug)]
pub struct FloatTag {
    value: f32,
}

impl FloatTag {
    pub const TYPE: TagType = TagType::Float;
    pub const ZERO: FloatTag = FloatTag { value: 0.0 };
    const SELF_SIZE_IN_BYTES: i32 = 12;

    /// The record's constructor, which Java deprecates for `valueOf`.
    pub fn new(value: f32) -> FloatTag {
        FloatTag { value }
    }

    /// `ZERO` for either zero, since -0.0 == 0.0.
    pub fn value_of(data: f32) -> FloatTag {
        if data == 0.0 {
            FloatTag::ZERO
        } else {
            FloatTag::new(data)
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn write(&self, output: &mut dyn DataOutput) {
        output.write_float(self.value);
    }

    pub fn size_in_bytes(&self) -> i32 {
        FloatTag::SELF_SIZE_IN_BYTES
    }

    pub fn get_id(&self) -> i8 {
        5
    }

    pub fn get_type(&self) -> TagType {
        FloatTag::TYPE
    }

    pub fn copy(&self) -> FloatTag {
        *self
    }

    /// `Float.hashCode`: its bits, every NaN the same.
    pub fn hash_code(&self) -> i32 {
        Float::hash_code(self.value)
    }

    /// `TYPE.load`: the f32, accounted as the tag's size.
    pub(super) fn load(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> FloatTag {
        FloatTag::value_of(read_accounted(input, accounter))
    }
}

fn read_accounted(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> f32 {
    accounter.account_bytes(i64::from(FloatTag::SELF_SIZE_IN_BYTES));
    input.read_float()
}

/// A record's, which compares its float with `Float.compare`: by its bits, every NaN the same.
impl PartialEq for FloatTag {
    fn eq(&self, other: &FloatTag) -> bool {
        Float::float_to_int_bits(self.value) == Float::float_to_int_bits(other.value)
    }
}

impl NumericTag for FloatTag {
    fn byte_value(&self) -> i8 {
        (Mth::floor_f32(self.value) & 0xFF) as i8
    }

    fn short_value(&self) -> i16 {
        (Mth::floor_f32(self.value) & 65535) as i16
    }

    fn int_value(&self) -> i32 {
        Mth::floor_f32(self.value)
    }

    fn long_value(&self) -> i64 {
        self.value as i64
    }

    fn float_value(&self) -> f32 {
        self.value
    }

    fn double_value(&self) -> f64 {
        f64::from(self.value)
    }

    fn r#box(&self) -> Number {
        Number::Float(self.value)
    }
}

class!(FloatTag, Float);

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::FloatTag;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::nbt::numeric_tag::NumericTag;
    use crate::net::minecraft::nbt::tag::tests::{
        check_class, encode_payload, matches_payload, payload,
    };

    // [5, value].
    impl Decode for FloatTag {
        fn decode(value: &Value) -> Self {
            FloatTag::new(f32::decode(payload(value, 5)))
        }
    }

    impl Encode for FloatTag {
        fn encode(&self) -> Value {
            encode_payload(5, &self.value)
        }

        fn matches(&self, expected: &Value) -> bool {
            matches_payload(5, &self.value, expected)
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.FloatTag");
        golden.pending(
            "toString()Ljava/lang/String;",
            "needs StringTagVisitor's SNBT, which prints floats with Java's Float.toString and Double.toString",
        );
        for name in [
            "accept(Lnet/minecraft/nbt/StreamTagVisitor;)Lnet/minecraft/nbt/StreamTagVisitor$ValueResult;",
            "accept(Lnet/minecraft/nbt/TagVisitor;)V",
        ] {
            golden.pending(name, "comes with the visitors");
        }
        golden.field("TYPE", FloatTag::TYPE);
        golden.field("ZERO", FloatTag::ZERO);
        golden.check("<init>(F)V", FloatTag::new);
        golden.check("valueOf(F)Lnet/minecraft/nbt/FloatTag;", FloatTag::value_of);
        golden.check_method("value()F", |this: &mut FloatTag| this.value());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut FloatTag, other: FloatTag| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut FloatTag| this.hash_code());
        check_class::<FloatTag>(&golden, "copy()Lnet/minecraft/nbt/FloatTag;");
        golden.check_method("byteValue()B", |this: &mut FloatTag| this.byte_value());
        golden.check_method("shortValue()S", |this: &mut FloatTag| this.short_value());
        golden.check_method("intValue()I", |this: &mut FloatTag| this.int_value());
        golden.check_method("longValue()J", |this: &mut FloatTag| this.long_value());
        golden.check_method("floatValue()F", |this: &mut FloatTag| this.float_value());
        golden.check_method("doubleValue()D", |this: &mut FloatTag| this.double_value());
        golden.check_method("box()Ljava/lang/Number;", |this: &mut FloatTag| {
            this.r#box()
        });
        golden.finish();
    }
}
