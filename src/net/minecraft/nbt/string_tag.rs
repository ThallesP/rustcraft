//! net/minecraft/nbt/StringTag.java
//!
//! Not ported yet: `toString` and `accept`, like every tag's (see `Tag`), and `quoteAndEscape`
//! and `escapeWithoutQuotes`, which write SNBT and come with it.
//!
//! A record, written in modified UTF-8 (`writeUTF`), whose length Java measures in UTF-16 units.

use crate::java::io::data_input::DataInput;
use crate::java::io::data_output::DataOutput;
use crate::java::lang::string::JavaString;
use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
use crate::net::minecraft::nbt::tag::class;
use crate::net::minecraft::nbt::tag_type::TagType;

/// A string in NBT.
#[derive(Clone, Debug, PartialEq)]
pub struct StringTag {
    value: String,
}

impl StringTag {
    pub const TYPE: TagType = TagType::String;
    const SELF_SIZE_IN_BYTES: i32 = 36;

    /// The record's constructor, which Java deprecates for `valueOf`.
    pub fn new(value: String) -> StringTag {
        StringTag { value }
    }

    /// Skips a string: its length in bytes, then that many bytes, or as many as are left.
    pub fn skip_string(input: &mut dyn DataInput) {
        let length = input.read_unsigned_short();
        input.skip_bytes(length);
    }

    /// Java hands out one tag for the empty string, which is the same value.
    pub fn value_of(data: String) -> StringTag {
        StringTag::new(data)
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    /// The string in modified UTF-8, which panics past 65535 bytes.
    pub fn write(&self, output: &mut dyn DataOutput) {
        output.write_utf(&self.value);
    }

    /// 36, and 2 per UTF-16 unit.
    pub fn size_in_bytes(&self) -> i32 {
        StringTag::SELF_SIZE_IN_BYTES.wrapping_add(2i32.wrapping_mul(utf16_length(&self.value)))
    }

    pub fn get_id(&self) -> i8 {
        8
    }

    pub fn get_type(&self) -> TagType {
        StringTag::TYPE
    }

    pub fn copy(&self) -> StringTag {
        self.clone()
    }

    pub fn as_string(&self) -> Option<&str> {
        Some(&self.value)
    }

    /// `String.hashCode`, a record's of one component.
    pub fn hash_code(&self) -> i32 {
        JavaString::hash_code(self.value.as_str())
    }

    /// `TYPE.load`: the string, accounted as the tag's size and 2 per UTF-16 unit.
    pub(super) fn load(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> StringTag {
        StringTag::value_of(read_accounted(input, accounter))
    }
}

fn read_accounted(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> String {
    accounter.account_bytes(i64::from(StringTag::SELF_SIZE_IN_BYTES));
    let data = input.read_utf();
    accounter.account_bytes_count(2, i64::from(utf16_length(&data)));
    data
}

/// `String.length()`, in UTF-16 units.
pub(super) fn utf16_length(s: &str) -> i32 {
    s.encode_utf16().count() as i32
}

class!(StringTag, String);

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::StringTag;
    use crate::golden::{Decode, Encode, Golden};
    use crate::io::netty::buffer::byte_buf::ByteBuf;
    use crate::io::netty::buffer::byte_buf_input_stream::ByteBufInputStream;
    use crate::net::minecraft::nbt::tag::tests::{
        check_class, encode_payload, matches_payload, payload,
    };

    // [8, value].
    impl Decode for StringTag {
        fn decode(value: &Value) -> Self {
            StringTag::new(String::decode(payload(value, 8)))
        }
    }

    impl Encode for StringTag {
        fn encode(&self) -> Value {
            encode_payload(8, &self.value)
        }

        fn matches(&self, expected: &Value) -> bool {
            matches_payload(8, &self.value, expected)
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.StringTag");
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
        for name in [
            "quoteAndEscape(Ljava/lang/String;)Ljava/lang/String;",
            "quoteAndEscape(Ljava/lang/String;Ljava/lang/StringBuilder;)V",
            "escapeWithoutQuotes(Ljava/lang/String;)Ljava/lang/String;",
            "escapeWithoutQuotes(Ljava/lang/String;Ljava/lang/StringBuilder;)V",
        ] {
            golden.pending(name, "comes with SNBT, which writes strings with them");
        }
        golden.field("TYPE", StringTag::TYPE);
        golden.check("<init>(Ljava/lang/String;)V", StringTag::new);
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/nbt/StringTag;",
            StringTag::value_of,
        );
        golden.check_mut("skipString(Ljava/io/DataInput;)V", |input: &mut ByteBuf| {
            StringTag::skip_string(&mut ByteBufInputStream::new(input))
        });
        golden.check_method("value()Ljava/lang/String;", |this: &mut StringTag| {
            this.value().to_owned()
        });
        golden.check_method("asString()Ljava/util/Optional;", |this: &mut StringTag| {
            this.as_string().map(str::to_owned)
        });
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut StringTag, other: StringTag| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut StringTag| this.hash_code());
        check_class::<StringTag>(&golden, "copy()Lnet/minecraft/nbt/StringTag;");
        golden.finish();
    }
}
