//! net/minecraft/core/RegistrySynchronization.java
//!
//! Not ported yet: everything but `PackedRegistryEntry`. `packRegistries` and the networkable
//! registries are the server's, which packs its registries with DataFixerUpper's codecs, and the
//! client's side of them comes with the registries.

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::object::HashCode;
use crate::net::minecraft::nbt::tag::Tag;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::resources::identifier::Identifier;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, PackedRegistryEntry> = LazyLock::new(|| {
    Box::new(stream_codec::composite2(
        Identifier::STREAM_CODEC,
        |entry: &PackedRegistryEntry| entry.id().clone(),
        ByteBufCodecs::TAG.apply(ByteBufCodecs::optional),
        |entry: &PackedRegistryEntry| entry.data().cloned(),
        PackedRegistryEntry::new,
    ))
});

/// An element of a registry the server sends: its identifier, and its data as NBT, or none when
/// the client has the element in a pack both know.
#[derive(Clone, Debug, PartialEq)]
pub struct PackedRegistryEntry {
    id: Identifier,
    data: Option<Tag>,
}

impl PackedRegistryEntry {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, PackedRegistryEntry> =
        &STREAM_CODEC;

    pub fn new(id: Identifier, data: Option<Tag>) -> PackedRegistryEntry {
        PackedRegistryEntry { id, data }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn data(&self) -> Option<&Tag> {
        self.data.as_ref()
    }

    /// A record's: each component's hash, added to 31 times the ones before. An empty
    /// `Optional` hashes to 0.
    pub fn hash_code(&self) -> i32 {
        self.id
            .hash_code()
            .wrapping_mul(31)
            .wrapping_add(self.data.as_ref().map_or(0, Tag::hash_code))
    }
}

/// A record's: `PackedRegistryEntry[id=minecraft:plains, data=Optional[{…}]]`, an `Optional`
/// printing `Optional.empty` for none.
impl fmt::Display for PackedRegistryEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PackedRegistryEntry[id={}, data=", self.id)?;
        match &self.data {
            Some(data) => write!(f, "Optional[{data}]]"),
            None => f.write_str("Optional.empty]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PackedRegistryEntry;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::nbt::tag::Tag;
    use crate::net::minecraft::resources::identifier::Identifier;

    // A record: its components.
    encode_fields!(PackedRegistryEntry {
        id: Identifier,
        data: Option<Tag>,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.RegistrySynchronization$PackedRegistryEntry");
        golden.check_method(
            "toString()Ljava/lang/String;",
            |this: &mut PackedRegistryEntry| this.to_string(),
        );
        golden.check_codec("STREAM_CODEC", || PackedRegistryEntry::STREAM_CODEC);
        golden.check(
            "<init>(Lnet/minecraft/resources/Identifier;Ljava/util/Optional;)V",
            PackedRegistryEntry::new,
        );
        golden.check_method(
            "id()Lnet/minecraft/resources/Identifier;",
            |this: &mut PackedRegistryEntry| this.id().clone(),
        );
        golden.check_method(
            "data()Ljava/util/Optional;",
            |this: &mut PackedRegistryEntry| this.data().cloned(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut PackedRegistryEntry, other: PackedRegistryEntry| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut PackedRegistryEntry| {
            this.hash_code()
        });
        golden.finish();
    }
}
