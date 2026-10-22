//! net/minecraft/nbt/NumericTag.java
//!
//! The numbers' interface, a trait that each of the six number tags implements. Its overrides of
//! `Tag`'s `asNumber`, `asByte` and the rest are `Tag`'s own here, which check for a number with
//! `as_numeric_tag`.

use crate::java::lang::number::Number;

/// A number tag's value, converted to each of Java's number types the way its class does: a
/// float or double is floored before it's cut to an int, a short or a byte.
pub trait NumericTag {
    fn byte_value(&self) -> i8;

    fn short_value(&self) -> i16;

    fn int_value(&self) -> i32;

    fn long_value(&self) -> i64;

    fn float_value(&self) -> f32;

    fn double_value(&self) -> f64;

    /// The value boxed as its own kind.
    fn r#box(&self) -> Number;
}

#[cfg(test)]
mod tests {
    use crate::golden::Golden;
    use crate::net::minecraft::nbt::numeric_tag::NumericTag;
    use crate::net::minecraft::nbt::tag::Tag;

    fn numeric(tag: &Tag) -> &dyn NumericTag {
        tag.as_numeric_tag()
            .expect("a NumericTag's receiver is a number")
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.NumericTag");
        golden.check_method("byteValue()B", |this: &mut Tag| numeric(this).byte_value());
        golden.check_method("shortValue()S", |this: &mut Tag| {
            numeric(this).short_value()
        });
        golden.check_method("intValue()I", |this: &mut Tag| numeric(this).int_value());
        golden.check_method("longValue()J", |this: &mut Tag| numeric(this).long_value());
        golden.check_method("floatValue()F", |this: &mut Tag| {
            numeric(this).float_value()
        });
        golden.check_method("doubleValue()D", |this: &mut Tag| {
            numeric(this).double_value()
        });
        golden.check_method("box()Ljava/lang/Number;", |this: &mut Tag| {
            numeric(this).r#box()
        });
        golden.check_method("asNumber()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_number()
        });
        golden.check_method("asByte()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_byte()
        });
        golden.check_method("asShort()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_short()
        });
        golden.check_method("asInt()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_int()
        });
        golden.check_method("asLong()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_long()
        });
        golden.check_method("asFloat()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_float()
        });
        golden.check_method("asDouble()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_double()
        });
        golden.check_method("asBoolean()Ljava/util/Optional;", |this: &mut Tag| {
            this.as_boolean()
        });
        golden.finish();
    }
}
