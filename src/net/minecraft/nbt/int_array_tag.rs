//! net/minecraft/nbt/IntArrayTag.java
//!
//! Not ported yet: `toString` and `accept`, like every tag's (see `Tag`).
//!
//! Java's `ArrayUtils.add` and `remove` (Apache Commons) make a new array a int longer or
//! shorter, which is `Vec`'s `insert` and `remove`, panicking for an index out of range where Java
//! throws `IndexOutOfBoundsException`.

use crate::java::io::data_input::DataInput;
use crate::java::io::data_output::DataOutput;
use crate::net::minecraft::nbt::collection_tag::CollectionTag;
use crate::net::minecraft::nbt::int_tag::IntTag;
use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
use crate::net::minecraft::nbt::tag::{Tag, class};
use crate::net::minecraft::nbt::tag_type::TagType;

/// An array of ints in NBT.
#[derive(Clone, Debug, PartialEq)]
pub struct IntArrayTag {
    data: Vec<i32>,
}

impl IntArrayTag {
    pub const TYPE: TagType = TagType::IntArray;
    const SELF_SIZE_IN_BYTES: i32 = 24;

    pub fn new(data: Vec<i32>) -> IntArrayTag {
        IntArrayTag { data }
    }

    /// Its length as an int, then each int.
    pub fn write(&self, output: &mut dyn DataOutput) {
        output.write_int(self.data.len() as i32);
        for &i in &self.data {
            output.write_int(i);
        }
    }

    /// 24, and 4 per int.
    pub fn size_in_bytes(&self) -> i32 {
        IntArrayTag::SELF_SIZE_IN_BYTES.wrapping_add(4i32.wrapping_mul(self.data.len() as i32))
    }

    pub fn get_id(&self) -> i8 {
        11
    }

    pub fn get_type(&self) -> TagType {
        IntArrayTag::TYPE
    }

    pub fn copy(&self) -> IntArrayTag {
        IntArrayTag::new(self.data.clone())
    }

    /// `Arrays.hashCode`: each int's hash added to 31 times the ones before, from 1.
    pub fn hash_code(&self) -> i32 {
        self.data.iter().fold(1, |hash: i32, element| {
            hash.wrapping_mul(31).wrapping_add(*element)
        })
    }

    pub fn get_as_int_array(&self) -> &[i32] {
        &self.data
    }

    /// The int at the index, as a tag.
    pub fn get(&self, index: i32) -> IntTag {
        IntTag::value_of(self.data[index_of(index)])
    }

    /// Takes the int at the index out, and returns it as a tag.
    pub fn remove(&mut self, index: i32) -> IntTag {
        let prev = self.data[index_of(index)];
        self.data.remove(index_of(index));
        IntTag::value_of(prev)
    }

    pub fn as_int_array(&self) -> Option<&[i32]> {
        Some(&self.data)
    }

    /// `TYPE.load`: the length, accounted with the tag's size and 4 per int before the
    /// array is made, then the ints.
    pub(super) fn load(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> IntArrayTag {
        IntArrayTag::new(read_accounted(input, accounter))
    }

    /// `TYPE.skip`: the length, then that many ints' bytes, or as many as are left. The
    /// product wraps.
    pub(super) fn skip(input: &mut dyn DataInput) {
        let length = input.read_int();
        input.skip_bytes(length.wrapping_mul(4));
    }
}

fn read_accounted(input: &mut dyn DataInput, accounter: &mut NbtAccounter) -> Vec<i32> {
    accounter.account_bytes(i64::from(IntArrayTag::SELF_SIZE_IN_BYTES));
    let length = input.read_int();
    accounter.account_bytes_count(4, i64::from(length));
    let mut data = vec![0i32; length as usize];
    for element in &mut data {
        *element = input.read_int();
    }
    data
}

// An array index, which panics when it's negative, like Java's.
fn index_of(index: i32) -> usize {
    usize::try_from(index).unwrap_or_else(|_| panic!("Index {index} out of bounds"))
}

impl CollectionTag for IntArrayTag {
    fn clear(&mut self) {
        self.data = Vec::new();
    }

    /// Takes a number, as a int.
    fn set_tag(&mut self, index: i32, tag: Tag) -> bool {
        match tag.as_numeric_tag() {
            Some(numeric) => {
                self.data[index_of(index)] = numeric.int_value();
                true
            }
            None => false,
        }
    }

    /// Takes a number, as a int, panicking for an index past the end.
    fn add_tag(&mut self, index: i32, tag: Tag) -> bool {
        match tag.as_numeric_tag() {
            Some(numeric) => {
                let value = numeric.int_value();
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
        Tag::from(IntArrayTag::remove(self, index))
    }

    fn get(&self, index: i32) -> Tag {
        Tag::from(IntArrayTag::get(self, index))
    }

    fn size(&self) -> i32 {
        self.data.len() as i32
    }
}

class!(IntArrayTag, IntArray);

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::IntArrayTag;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::nbt::collection_tag::CollectionTag;
    use crate::net::minecraft::nbt::tag::Tag;
    use crate::net::minecraft::nbt::tag::tests::{
        check_class, encode_payload, matches_payload, payload,
    };

    // [11, [int, …]].
    impl Decode for IntArrayTag {
        fn decode(value: &Value) -> Self {
            IntArrayTag::new(Vec::decode(payload(value, 11)))
        }
    }

    impl Encode for IntArrayTag {
        fn encode(&self) -> Value {
            encode_payload(11, &self.data)
        }

        fn matches(&self, expected: &Value) -> bool {
            matches_payload(11, &self.data, expected)
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.IntArrayTag");
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
        golden.field("TYPE", IntArrayTag::TYPE);
        golden.check("<init>([I)V", IntArrayTag::new);
        check_class::<IntArrayTag>(&golden, "copy()Lnet/minecraft/nbt/IntArrayTag;");
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut IntArrayTag, other: IntArrayTag| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut IntArrayTag| this.hash_code());
        golden.check_method("getAsIntArray()[I", |this: &mut IntArrayTag| {
            this.get_as_int_array().to_vec()
        });
        golden.check_method("size()I", |this: &mut IntArrayTag| this.size());
        golden.check_method(
            "get(I)Lnet/minecraft/nbt/IntTag;",
            |this: &mut IntArrayTag, index: i32| this.get(index),
        );
        golden.check_method(
            "setTag(ILnet/minecraft/nbt/Tag;)Z",
            |this: &mut IntArrayTag, index: i32, tag: Tag| this.set_tag(index, tag),
        );
        golden.check_method(
            "addTag(ILnet/minecraft/nbt/Tag;)Z",
            |this: &mut IntArrayTag, index: i32, tag: Tag| this.add_tag(index, tag),
        );
        golden.check_method(
            "remove(I)Lnet/minecraft/nbt/IntTag;",
            |this: &mut IntArrayTag, index: i32| this.remove(index),
        );
        golden.check_method("clear()V", |this: &mut IntArrayTag| this.clear());
        golden.check_method(
            "asIntArray()Ljava/util/Optional;",
            |this: &mut IntArrayTag| this.as_int_array().map(<[i32]>::to_vec),
        );
        golden.finish();
    }
}
