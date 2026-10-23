//! net/minecraft/nbt/LongArrayTag.java
//!
//! Not ported yet: `toString` and `accept`, like every tag's (see `Tag`).
//!
//! Java's `ArrayUtils.add` and `remove` (Apache Commons) make a new array a long longer or
//! shorter, which is `Vec`'s `insert` and `remove`, panicking for an index out of range where Java
//! throws `IndexOutOfBoundsException`.

use std::fmt;

use crate::java::io::data_input::DataInput;
use crate::java::io::data_output::DataOutput;
use crate::net::minecraft::nbt::collection_tag::CollectionTag;
use crate::net::minecraft::nbt::long_tag::LongTag;
use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
use crate::net::minecraft::nbt::string_tag_visitor::StringTagVisitor;
use crate::net::minecraft::nbt::tag::{Tag, class};
use crate::net::minecraft::nbt::tag_type::TagType;

/// An array of longs in NBT.
#[derive(Clone, Debug, PartialEq)]
pub struct LongArrayTag {
    data: Vec<i64>,
}

impl LongArrayTag {
    pub const TYPE: TagType = TagType::LongArray;
    const SELF_SIZE_IN_BYTES: i32 = 24;

    pub fn new(data: Vec<i64>) -> LongArrayTag {
        LongArrayTag { data }
    }

    /// Its length as an int, then each long.
    pub fn write(&self, output: &mut dyn DataOutput) {
        output.write_int(self.data.len() as i32);
        for &i in &self.data {
            output.write_long(i);
        }
    }

    /// 24, and 8 per long.
    pub fn size_in_bytes(&self) -> i32 {
        LongArrayTag::SELF_SIZE_IN_BYTES.wrapping_add(8i32.wrapping_mul(self.data.len() as i32))
    }

    pub fn get_id(&self) -> i8 {
        12
    }

    pub fn get_type(&self) -> TagType {
        LongArrayTag::TYPE
    }

    pub fn copy(&self) -> LongArrayTag {
        LongArrayTag::new(self.data.clone())
    }

    /// `Arrays.hashCode`: each long's `Long.hashCode` hash added to 31 times the ones before, from 1.
    pub fn hash_code(&self) -> i32 {
        self.data.iter().fold(1, |hash: i32, element| {
            hash.wrapping_mul(31)
                .wrapping_add((*element ^ ((*element as u64) >> 32) as i64) as i32)
        })
    }

    pub fn get_as_long_array(&self) -> &[i64] {
        &self.data
    }

    /// The long at the index, as a tag.
    pub fn get(&self, index: i32) -> LongTag {
        LongTag::value_of(self.data[index_of(index)])
    }

    /// Takes the long at the index out, and returns it as a tag.
    pub fn remove(&mut self, index: i32) -> LongTag {
        let prev = self.data[index_of(index)];
        self.data.remove(index_of(index));
        LongTag::value_of(prev)
    }

    pub fn as_long_array(&self) -> Option<&[i64]> {
        Some(&self.data)
    }

    /// `TYPE.load`: the length, accounted with the tag's size and 8 per long before the
    /// array is made, then the longs.
    pub(super) fn load(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> LongArrayTag {
        LongArrayTag::new(read_accounted(input, accounter))
    }

    /// `TYPE.skip`: the length, then that many longs' bytes, or as many as are left. The
    /// product wraps.
    pub(super) fn skip(input: &mut dyn DataInput) {
        let length = input.read_int();
        input.skip_bytes(length.wrapping_mul(8));
    }
}

fn read_accounted(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> Vec<i64> {
    accounter.account_bytes(i64::from(LongArrayTag::SELF_SIZE_IN_BYTES));
    let length = input.read_int();
    accounter.account_bytes_count(8, i64::from(length));
    let mut data = vec![0i64; length as usize];
    for element in &mut data {
        *element = input.read_long();
    }
    data
}

// An array index, which panics when it's negative, like Java's.
fn index_of(index: i32) -> usize {
    usize::try_from(index).unwrap_or_else(|_| panic!("Index {index} out of bounds"))
}

impl CollectionTag for LongArrayTag {
    fn clear(&mut self) {
        self.data = Vec::new();
    }

    /// Takes a number, as a long.
    fn set_tag(&mut self, index: i32, tag: Tag) -> bool {
        match tag.as_numeric_tag() {
            Some(numeric) => {
                self.data[index_of(index)] = numeric.long_value();
                true
            }
            None => false,
        }
    }

    /// Takes a number, as a long, panicking for an index past the end.
    fn add_tag(&mut self, index: i32, tag: Tag) -> bool {
        match tag.as_numeric_tag() {
            Some(numeric) => {
                let value = numeric.long_value();
                assert!(
                    index >= 0 && index as usize <= self.data.len(),
                    "Index: {index}, Length: {}",
                    self.data.len()
                );
                self.data.insert(index as usize, value);
                true
            }
            None => false,
        }
    }

    fn remove(&mut self, index: i32) -> Tag {
        Tag::from(LongArrayTag::remove(self, index))
    }

    fn get(&self, index: i32) -> Tag {
        Tag::from(LongArrayTag::get(self, index))
    }

    fn size(&self) -> i32 {
        self.data.len() as i32
    }
}

class!(LongArrayTag, LongArray);

/// `toString()`: its SNBT (`StringTagVisitor`).
impl fmt::Display for LongArrayTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut visitor = StringTagVisitor::new();
        visitor.visit_long_array(self);
        f.write_str(&visitor.build())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::LongArrayTag;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::nbt::collection_tag::CollectionTag;
    use crate::net::minecraft::nbt::tag::Tag;
    use crate::net::minecraft::nbt::tag::tests::{
        check_class, encode_payload, matches_payload, payload,
    };

    // [12, [long, …]].
    impl Decode for LongArrayTag {
        fn decode(value: &Value) -> Self {
            LongArrayTag::new(Vec::decode(payload(value, 12)))
        }
    }

    impl Encode for LongArrayTag {
        fn encode(&self) -> Value {
            encode_payload(12, &self.data)
        }

        fn matches(&self, expected: &Value) -> bool {
            matches_payload(12, &self.data, expected)
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.LongArrayTag");
        golden.check_method("toString()Ljava/lang/String;", |this: &mut LongArrayTag| {
            this.to_string()
        });
        for name in [
            "accept(Lnet/minecraft/nbt/StreamTagVisitor;)Lnet/minecraft/nbt/StreamTagVisitor$ValueResult;",
            "accept(Lnet/minecraft/nbt/TagVisitor;)V",
        ] {
            golden.pending(name, "comes with the visitors");
        }
        golden.field("TYPE", LongArrayTag::TYPE);
        golden.check("<init>([J)V", LongArrayTag::new);
        check_class::<LongArrayTag>(&golden, "copy()Lnet/minecraft/nbt/LongArrayTag;");
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut LongArrayTag, other: LongArrayTag| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut LongArrayTag| this.hash_code());
        golden.check_method("getAsLongArray()[J", |this: &mut LongArrayTag| {
            this.get_as_long_array().to_vec()
        });
        golden.check_method("size()I", |this: &mut LongArrayTag| this.size());
        golden.check_method(
            "get(I)Lnet/minecraft/nbt/LongTag;",
            |this: &mut LongArrayTag, index: i32| this.get(index),
        );
        golden.check_method(
            "setTag(ILnet/minecraft/nbt/Tag;)Z",
            |this: &mut LongArrayTag, index: i32, tag: Tag| this.set_tag(index, tag),
        );
        golden.check_method(
            "addTag(ILnet/minecraft/nbt/Tag;)Z",
            |this: &mut LongArrayTag, index: i32, tag: Tag| this.add_tag(index, tag),
        );
        golden.check_method(
            "remove(I)Lnet/minecraft/nbt/LongTag;",
            |this: &mut LongArrayTag, index: i32| this.remove(index),
        );
        golden.check_method("clear()V", |this: &mut LongArrayTag| this.clear());
        golden.check_method(
            "asLongArray()Ljava/util/Optional;",
            |this: &mut LongArrayTag| this.as_long_array().map(<[i64]>::to_vec),
        );
        golden.finish();
    }
}
