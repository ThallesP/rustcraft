//! net/minecraft/server/packs/repository/KnownPack.java
//!
//! `vanilla(String)` names the game's version, `SharedConstants.getCurrentVersion().id()`, which
//! Java reads from the jar's `version.json` (`SharedConstants`).

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::object::HashCode;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::shared_constants::SharedConstants;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, KnownPack> = LazyLock::new(|| {
    Box::new(stream_codec::composite3(
        ByteBufCodecs::STRING_UTF8,
        |pack: &KnownPack| pack.namespace().to_owned(),
        ByteBufCodecs::STRING_UTF8,
        |pack: &KnownPack| pack.id().to_owned(),
        ByteBufCodecs::STRING_UTF8,
        |pack: &KnownPack| pack.version().to_owned(),
        KnownPack::new,
    ))
});

/// A data pack both sides may have, by its namespace, id and version, so the server can leave
/// out of the configuration what the client already has (the vanilla `core` pack).
#[derive(Clone, Debug, PartialEq)]
pub struct KnownPack {
    namespace: String,
    id: String,
    version: String,
}

impl KnownPack {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, KnownPack> = &STREAM_CODEC;
    pub const VANILLA_NAMESPACE: &'static str = "minecraft";

    pub fn new(namespace: String, id: String, version: String) -> KnownPack {
        KnownPack {
            namespace,
            id,
            version,
        }
    }

    /// One of the game's own packs, of the game's version.
    pub fn vanilla(id: &str) -> KnownPack {
        KnownPack::new(
            KnownPack::VANILLA_NAMESPACE.to_string(),
            id.to_string(),
            SharedConstants::get_current_version_id().to_string(),
        )
    }

    pub fn is_vanilla(&self) -> bool {
        self.namespace == KnownPack::VANILLA_NAMESPACE
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    /// A record's: each component's hash, added to 31 times the ones before.
    pub fn hash_code(&self) -> i32 {
        self.namespace
            .hash_code()
            .wrapping_mul(31)
            .wrapping_add(self.id.hash_code())
            .wrapping_mul(31)
            .wrapping_add(self.version.hash_code())
    }
}

/// `minecraft:core:26.3`.
impl fmt::Display for KnownPack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.namespace, self.id, self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::KnownPack;
    use crate::golden::{Golden, encode_fields};

    // A record: its components.
    encode_fields!(KnownPack {
        namespace: String,
        id: String,
        version: String,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.server.packs.repository.KnownPack");
        golden.check(
            "vanilla(Ljava/lang/String;)Lnet/minecraft/server/packs/repository/KnownPack;",
            |id: String| KnownPack::vanilla(&id),
        );
        golden.check_codec("STREAM_CODEC", || KnownPack::STREAM_CODEC);
        golden.field("VANILLA_NAMESPACE", KnownPack::VANILLA_NAMESPACE.to_owned());
        golden.check(
            "<init>(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
            KnownPack::new,
        );
        golden.check_method("isVanilla()Z", |this: &mut KnownPack| this.is_vanilla());
        golden.check_method("namespace()Ljava/lang/String;", |this: &mut KnownPack| {
            this.namespace().to_owned()
        });
        golden.check_method("id()Ljava/lang/String;", |this: &mut KnownPack| {
            this.id().to_owned()
        });
        golden.check_method("version()Ljava/lang/String;", |this: &mut KnownPack| {
            this.version().to_owned()
        });
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut KnownPack, other: KnownPack| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut KnownPack| this.hash_code());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut KnownPack| {
            this.to_string()
        });
        golden.finish();
    }
}
