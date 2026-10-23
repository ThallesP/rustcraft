//! net/minecraft/nbt/DoubleTag.java
//!
//! Not ported yet: `toString` and `accept`, like every tag's (see `Tag`).
//!
//! A record, which `equals` with `Double.compare`: -0.0 isn't 0.0, and NaN is NaN. `valueOf` makes
//! both zeros the `ZERO` constant, which is 0.0, so a -0.0 read from NBT comes back as 0.0.

use std::fmt;

use crate::java::io::data_input::DataInput;
use crate::java::io::data_output::DataOutput;
use crate::java::lang::double::Double;
use crate::java::lang::number::Number;
use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
use crate::net::minecraft::nbt::numeric_tag::NumericTag;
use crate::net::minecraft::nbt::string_tag_visitor::StringTagVisitor;
use crate::net::minecraft::nbt::tag::class;
use crate::net::minecraft::nbt::tag_type::TagType;
use crate::net::minecraft::util::mth::Mth;

/// A f64 in NBT.
#[derive(Clone, Copy, Debug)]
pub struct DoubleTag {
    value: f64,
}

impl DoubleTag {
    pub const TYPE: TagType = TagType::Double;
    pub const ZERO: DoubleTag = DoubleTag { value: 0.0 };
    const SELF_SIZE_IN_BYTES: i32 = 16;

    /// The record's constructor, which Java deprecates for `valueOf`.
    pub fn new(value: f64) -> DoubleTag {
        DoubleTag { value }
    }

    /// `ZERO` for either zero, since -0.0 == 0.0.
    pub fn value_of(data: f64) -> DoubleTag {
        if data == 0.0 {
            DoubleTag::ZERO
        } else {
            DoubleTag::new(data)
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn write(&self, output: &mut dyn DataOutput) {
        output.write_double(self.value);
    }

    pub fn size_in_bytes(&self) -> i32 {
        DoubleTag::SELF_SIZE_IN_BYTES
    }

    pub fn get_id(&self) -> i8 {
        6
    }

    pub fn get_type(&self) -> TagType {
        DoubleTag::TYPE
    }

    pub fn copy(&self) -> DoubleTag {
        *self
    }

    /// `Double.hashCode`: its bits' two halves XORed, every NaN the same.
    pub fn hash_code(&self) -> i32 {
        Double::hash_code(self.value)
    }

    /// `TYPE.load`: the f64, accounted as the tag's size.
    pub(super) fn load(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> DoubleTag {
        DoubleTag::value_of(read_accounted(input, accounter))
    }
}

fn read_accounted(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> f64 {
    accounter.account_bytes(i64::from(DoubleTag::SELF_SIZE_IN_BYTES));
    input.read_double()
}

/// A record's, which compares its double with `Double.compare`: by its bits, every NaN the same.
impl PartialEq for DoubleTag {
    fn eq(&self, other: &DoubleTag) -> bool {
        Double::double_to_long_bits(self.value) == Double::double_to_long_bits(other.value)
    }
}

impl NumericTag for DoubleTag {
    fn byte_value(&self) -> i8 {
        (Mth::floor_f64(self.value) & 0xFF) as i8
    }

    fn short_value(&self) -> i16 {
        (Mth::floor_f64(self.value) & 65535) as i16
    }

    fn int_value(&self) -> i32 {
        Mth::floor_f64(self.value)
    }

    fn long_value(&self) -> i64 {
        self.value.floor() as i64
    }

    fn float_value(&self) -> f32 {
        self.value as f32
    }

    fn double_value(&self) -> f64 {
        self.value
    }

    fn r#box(&self) -> Number {
        Number::Double(self.value)
    }
}

class!(DoubleTag, Double);

/// `toString()`: its SNBT (`StringTagVisitor`).
impl fmt::Display for DoubleTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut visitor = StringTagVisitor::new();
        visitor.visit_double(self);
        f.write_str(&visitor.build())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::DoubleTag;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::nbt::numeric_tag::NumericTag;
    use crate::net::minecraft::nbt::tag::tests::{
        check_class, encode_payload, matches_payload, payload,
    };

    // [6, value].
    impl Decode for DoubleTag {
        fn decode(value: &Value) -> Self {
            DoubleTag::new(f64::decode(payload(value, 6)))
        }
    }

    impl Encode for DoubleTag {
        fn encode(&self) -> Value {
            encode_payload(6, &self.value)
        }

        fn matches(&self, expected: &Value) -> bool {
            matches_payload(6, &self.value, expected)
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.DoubleTag");
        golden.check_method("toString()Ljava/lang/String;", |this: &mut DoubleTag| {
            this.to_string()
        });
        for name in [
            "accept(Lnet/minecraft/nbt/StreamTagVisitor;)Lnet/minecraft/nbt/StreamTagVisitor$ValueResult;",
            "accept(Lnet/minecraft/nbt/TagVisitor;)V",
        ] {
            golden.pending(name, "comes with the visitors");
        }
        golden.field("TYPE", DoubleTag::TYPE);
        golden.field("ZERO", DoubleTag::ZERO);
        golden.check("<init>(D)V", DoubleTag::new);
        golden.check(
            "valueOf(D)Lnet/minecraft/nbt/DoubleTag;",
            DoubleTag::value_of,
        );
        golden.check_method("value()D", |this: &mut DoubleTag| this.value());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut DoubleTag, other: DoubleTag| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut DoubleTag| this.hash_code());
        check_class::<DoubleTag>(&golden, "copy()Lnet/minecraft/nbt/DoubleTag;");
        golden.check_method("byteValue()B", |this: &mut DoubleTag| this.byte_value());
        golden.check_method("shortValue()S", |this: &mut DoubleTag| this.short_value());
        golden.check_method("intValue()I", |this: &mut DoubleTag| this.int_value());
        golden.check_method("longValue()J", |this: &mut DoubleTag| this.long_value());
        golden.check_method("floatValue()F", |this: &mut DoubleTag| this.float_value());
        golden.check_method("doubleValue()D", |this: &mut DoubleTag| this.double_value());
        golden.check_method("box()Ljava/lang/Number;", |this: &mut DoubleTag| {
            this.r#box()
        });
        golden.finish();
    }
}
