//! net/minecraft/nbt/ByteArrayTag.java
//!
//! Not ported yet: `toString` and `accept`, like every tag's (see `Tag`).
//!
//! Java's `ArrayUtils.add` and `remove` (Apache Commons) make a new array a byte longer or
//! shorter, which is `Vec`'s `insert` and `remove`, panicking for an index out of range where Java
//! throws `IndexOutOfBoundsException`.

use crate::java::io::data_input::DataInput;
use crate::java::io::data_output::DataOutput;
use crate::net::minecraft::nbt::byte_tag::ByteTag;
use crate::net::minecraft::nbt::collection_tag::CollectionTag;
use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
use crate::net::minecraft::nbt::tag::{Tag, class};
use crate::net::minecraft::nbt::tag_type::TagType;

/// An array of bytes in NBT.
#[derive(Clone, Debug, PartialEq)]
pub struct ByteArrayTag {
    data: Vec<i8>,
}

impl ByteArrayTag {
    pub const TYPE: TagType = TagType::ByteArray;
    const SELF_SIZE_IN_BYTES: i32 = 24;

    pub fn new(data: Vec<i8>) -> ByteArrayTag {
        ByteArrayTag { data }
    }

    /// Its length as an int, then each byte.
    pub fn write(&self, output: &mut dyn DataOutput) {
        output.write_int(self.data.len() as i32);
        output.write(&self.data);
    }

    /// 24, and 1 per byte.
    pub fn size_in_bytes(&self) -> i32 {
        ByteArrayTag::SELF_SIZE_IN_BYTES.wrapping_add(1i32.wrapping_mul(self.data.len() as i32))
    }

    pub fn get_id(&self) -> i8 {
        7
    }

    pub fn get_type(&self) -> TagType {
        ByteArrayTag::TYPE
    }

    pub fn copy(&self) -> Tag {
        Tag::ByteArray(ByteArrayTag::new(self.data.clone()))
    }

    /// `Arrays.hashCode`: each byte's hash added to 31 times the ones before, from 1.
    pub fn hash_code(&self) -> i32 {
        self.data.iter().fold(1, |hash: i32, element| {
            hash.wrapping_mul(31).wrapping_add(i32::from(*element))
        })
    }

    pub fn get_as_byte_array(&self) -> &[i8] {
        &self.data
    }

    /// The byte at the index, as a tag.
    pub fn get(&self, index: i32) -> ByteTag {
        ByteTag::value_of(self.data[index_of(index)])
    }

    /// Takes the byte at the index out, and returns it as a tag.
    pub fn remove(&mut self, index: i32) -> ByteTag {
        let prev = self.data[index_of(index)];
        self.data.remove(index_of(index));
        ByteTag::value_of(prev)
    }

    pub fn as_byte_array(&self) -> Option<&[i8]> {
        Some(&self.data)
    }

    /// `TYPE.load`: the length, accounted with the tag's size and 1 per byte before the
    /// array is made, then the bytes.
    pub(super) fn load(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> ByteArrayTag {
        ByteArrayTag::new(read_accounted(input, accounter))
    }

    /// `TYPE.skip`: the length, then that many bytes' bytes, or as many as are left. The
    /// product wraps.
    pub(super) fn skip(input: &mut dyn DataInput) {
        let length = input.read_int();
        input.skip_bytes(length.wrapping_mul(1));
    }
}

fn read_accounted(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> Vec<i8> {
    accounter.account_bytes(i64::from(ByteArrayTag::SELF_SIZE_IN_BYTES));
    let length = input.read_int();
    accounter.account_bytes_count(1, i64::from(length));
    let mut data = vec![0i8; length as usize];
    input.read_fully(&mut data);
    data
}

// An array index, which panics when it's negative, like Java's.
fn index_of(index: i32) -> usize {
    usize::try_from(index).unwrap_or_else(|_| panic!("Index {index} out of bounds"))
}

impl CollectionTag for ByteArrayTag {
    fn clear(&mut self) {
        self.data = Vec::new();
    }

    /// Takes a number, as a byte.
    fn set_tag(&mut self, index: i32, tag: Tag) -> bool {
        match tag.as_numeric_tag() {
            Some(numeric) => {
                self.data[index_of(index)] = numeric.byte_value();
                true
            }
            None => false,
        }
    }

    /// Takes a number, as a byte, panicking for an index past the end.
    fn add_tag(&mut self, index: i32, tag: Tag) -> bool {
        match tag.as_numeric_tag() {
            Some(numeric) => {
                let value = numeric.byte_value();
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
        Tag::from(ByteArrayTag::remove(self, index))
    }

    fn get(&self, index: i32) -> Tag {
        Tag::from(ByteArrayTag::get(self, index))
    }

    fn size(&self) -> i32 {
        self.data.len() as i32
    }
}

class!(ByteArrayTag, ByteArray);

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::ByteArrayTag;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::nbt::collection_tag::CollectionTag;
    use crate::net::minecraft::nbt::tag::Tag;
    use crate::net::minecraft::nbt::tag::tests::{
        check_class, encode_payload, matches_payload, payload,
    };

    // [7, [byte, …]].
    impl Decode for ByteArrayTag {
        fn decode(value: &Value) -> Self {
            ByteArrayTag::new(Vec::decode(payload(value, 7)))
        }
    }

    impl Encode for ByteArrayTag {
        fn encode(&self) -> Value {
            encode_payload(7, &self.data)
        }

        fn matches(&self, expected: &Value) -> bool {
            matches_payload(7, &self.data, expected)
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.ByteArrayTag");
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
        golden.field("TYPE", ByteArrayTag::TYPE);
        golden.check("<init>([B)V", ByteArrayTag::new);
        check_class::<ByteArrayTag>(&golden, "copy()Lnet/minecraft/nbt/Tag;");
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut ByteArrayTag, other: ByteArrayTag| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut ByteArrayTag| this.hash_code());
        golden.check_method("getAsByteArray()[B", |this: &mut ByteArrayTag| {
            this.get_as_byte_array().to_vec()
        });
        golden.check_method("size()I", |this: &mut ByteArrayTag| this.size());
        golden.check_method(
            "get(I)Lnet/minecraft/nbt/ByteTag;",
            |this: &mut ByteArrayTag, index: i32| this.get(index),
        );
        golden.check_method(
            "setTag(ILnet/minecraft/nbt/Tag;)Z",
            |this: &mut ByteArrayTag, index: i32, tag: Tag| this.set_tag(index, tag),
        );
        golden.check_method(
            "addTag(ILnet/minecraft/nbt/Tag;)Z",
            |this: &mut ByteArrayTag, index: i32, tag: Tag| this.add_tag(index, tag),
        );
        golden.check_method(
            "remove(I)Lnet/minecraft/nbt/ByteTag;",
            |this: &mut ByteArrayTag, index: i32| this.remove(index),
        );
        golden.check_method("clear()V", |this: &mut ByteArrayTag| this.clear());
        golden.check_method(
            "asByteArray()Ljava/util/Optional;",
            |this: &mut ByteArrayTag| this.as_byte_array().map(<[i8]>::to_vec),
        );
        golden.finish();
    }
}
