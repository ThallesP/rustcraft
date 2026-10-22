//! net/minecraft/nbt/CollectionTag.java
//!
//! The interface of lists and arrays, a trait that `ListTag`, `ByteArrayTag`, `IntArrayTag` and
//! `LongArrayTag` implement. Java's `Iterable<Tag>` is `iterator`, which hands out each element
//! as `get` does.

use crate::net::minecraft::nbt::tag::Tag;

/// A list's tags, or an array's numbers as tags.
pub trait CollectionTag {
    fn clear(&mut self);

    /// Puts the tag at the index, and answers whether it could: an array only takes a number,
    /// which it converts to its own kind.
    fn set_tag(&mut self, index: i32, tag: Tag) -> bool;

    /// Inserts the tag at the index, and answers whether it could, like `set_tag`.
    fn add_tag(&mut self, index: i32, tag: Tag) -> bool;

    fn remove(&mut self, index: i32) -> Tag;

    /// The element at the index: a list's own tag (a copy of it here, where Java shares it), or
    /// an array's number as a new tag.
    fn get(&self, index: i32) -> Tag;

    fn size(&self) -> i32;

    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// Each element in order, as `get` hands it out.
    fn iterator(&self) -> Box<dyn Iterator<Item = Tag> + '_> {
        Box::new((0..self.size()).map(|index| self.get(index)))
    }

    fn stream(&self) -> Box<dyn Iterator<Item = Tag> + '_> {
        self.iterator()
    }
}

#[cfg(test)]
mod tests {
    use crate::golden::Golden;
    use crate::net::minecraft::nbt::collection_tag::CollectionTag;
    use crate::net::minecraft::nbt::tag::Tag;

    fn collection(tag: &mut Tag) -> &mut dyn CollectionTag {
        tag.as_collection_tag_mut()
            .expect("a CollectionTag's receiver is a list or an array")
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.CollectionTag");
        golden.check_method("clear()V", |this: &mut Tag| collection(this).clear());
        golden.check_method(
            "setTag(ILnet/minecraft/nbt/Tag;)Z",
            |this: &mut Tag, index: i32, tag: Tag| collection(this).set_tag(index, tag),
        );
        golden.check_method(
            "addTag(ILnet/minecraft/nbt/Tag;)Z",
            |this: &mut Tag, index: i32, tag: Tag| collection(this).add_tag(index, tag),
        );
        golden.check_method(
            "remove(I)Lnet/minecraft/nbt/Tag;",
            |this: &mut Tag, index: i32| collection(this).remove(index),
        );
        golden.check_method(
            "get(I)Lnet/minecraft/nbt/Tag;",
            |this: &mut Tag, index: i32| collection(this).get(index),
        );
        golden.check_method("size()I", |this: &mut Tag| collection(this).size());
        golden.check_method("isEmpty()Z", |this: &mut Tag| collection(this).is_empty());
        golden.check_method("iterator()Ljava/util/Iterator;", |this: &mut Tag| {
            collection(this).iterator().collect::<Vec<_>>()
        });
        golden.check_method("stream()Ljava/util/stream/Stream;", |this: &mut Tag| {
            collection(this).stream().collect::<Vec<_>>()
        });
        golden.finish();
    }
}
