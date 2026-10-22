//! net/minecraft/nbt/EndTag.java
//!
//! Not ported yet: `toString` and `accept`, like every tag's (see `Tag`).
//!
//! The tag that ends a compound, and what a read of no tag gives. Java's is a singleton,
//! `INSTANCE`, which the game compares tags with by reference: every `EndTag` here is it.

use crate::java::io::data_output::DataOutput;
use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
use crate::net::minecraft::nbt::tag::class;
use crate::net::minecraft::nbt::tag_type::TagType;

/// No tag: nothing to write after its id.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EndTag;

impl EndTag {
    pub const TYPE: TagType = TagType::End;
    pub const INSTANCE: EndTag = EndTag;
    const SELF_SIZE_IN_BYTES: i32 = 8;

    pub fn write(&self, _output: &mut dyn DataOutput) {}

    pub fn size_in_bytes(&self) -> i32 {
        EndTag::SELF_SIZE_IN_BYTES
    }

    pub fn get_id(&self) -> i8 {
        0
    }

    pub fn get_type(&self) -> TagType {
        EndTag::TYPE
    }

    pub fn copy(&self) -> EndTag {
        *self
    }

    /// `TYPE.load`: reads nothing, but accounts for the tag.
    pub(super) fn load(accounter: &mut NbtAccounter) -> EndTag {
        accounter.account_bytes(i64::from(EndTag::SELF_SIZE_IN_BYTES));
        EndTag::INSTANCE
    }
}

class!(EndTag, End);

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::EndTag;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::nbt::tag::tests::{
        check_class, encode_payload, matches_payload, payload,
    };

    // [0, null].
    impl Decode for EndTag {
        fn decode(value: &Value) -> Self {
            assert!(
                payload(value, 0).is_null(),
                "an EndTag has no payload, got {value}"
            );
            EndTag
        }
    }

    impl Encode for EndTag {
        fn encode(&self) -> Value {
            encode_payload(0, &())
        }

        fn matches(&self, expected: &Value) -> bool {
            matches_payload(0, &(), expected)
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.EndTag");
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
        golden.field("TYPE", EndTag::TYPE);
        golden.field("INSTANCE", EndTag::INSTANCE);
        check_class::<EndTag>(&golden, "copy()Lnet/minecraft/nbt/EndTag;");
        golden.finish();
    }
}
