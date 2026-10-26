//! net/minecraft/resources/ResourceKey.java
//!
//! Not ported yet: `codec`, which waits for DataFixerUpper's `Codec`, and
//! `dependent(ResourceKey, UnaryOperator)`, which the golden harness can't generate an argument
//! for (`dependent` is the `String` overload).
//!
//! Java interns its keys: `create` hands out the one key there is for a registry's name and an
//! identifier, so the game compares keys by reference (`ResourceKey` doesn't override `equals`)
//! and they hash their identity. With one key per pair, comparing references is comparing the
//! pairs, so here a key is a value that compares its names. Its `hashCode`, an identity hash that
//! changes from run to run, isn't ported. `ResourceKey<T>`'s type argument only tells the compiler
//! what it's a key of, so it's a plain `ResourceKey`, like `PacketType`.
//!
//! The names are shared, like Java's one object: a key copied is the same names (`Arc`), which a
//! comparison checks first, and a key hashes a digest of its names worked out when it's made, so a
//! map of keys (a registry's ids, a biome's holder) hashes eight bytes rather than four strings.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, LazyLock};

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};
use crate::net::minecraft::resources::identifier::Identifier;

static REGISTRY_STREAM_CODEC: StaticStreamCodec<ByteBuf, ResourceKey> = LazyLock::new(|| {
    Box::new(
        Identifier::STREAM_CODEC.map(ResourceKey::create_registry_key, |key: &ResourceKey| {
            key.identifier().clone()
        }),
    )
});

/// The name of an element of a registry, `minecraft:plains` in `minecraft:worldgen/biome`. A
/// registry's own key is in the root registry, `minecraft:root`.
#[derive(Clone, Debug)]
pub struct ResourceKey(Arc<Names>);

/// A key's names, and the digest of them it hashes.
#[derive(Debug)]
struct Names {
    registry_name: Identifier,
    identifier: Identifier,
    digest: u64,
}

impl PartialEq for ResourceKey {
    fn eq(&self, other: &ResourceKey) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
            || self.0.digest == other.0.digest
                && self.0.registry_name == other.0.registry_name
                && self.0.identifier == other.0.identifier
    }
}

impl Eq for ResourceKey {}

/// The names' digest: keys with the same names hash the same.
impl Hash for ResourceKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0.digest);
    }
}

impl ResourceKey {
    /// A registry's key, by its identifier, as the server names a registry it sends.
    pub const REGISTRY_STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ResourceKey> =
        &REGISTRY_STREAM_CODEC;

    /// A key in the registry, by its identifier.
    pub fn stream_codec(
        registry_name: &ResourceKey,
    ) -> impl StreamCodec<Buffer = ByteBuf, Value = ResourceKey> + use<> {
        let registry_name = registry_name.clone();
        Identifier::STREAM_CODEC.map(
            move |name| ResourceKey::create(&registry_name, name),
            |key: &ResourceKey| key.identifier().clone(),
        )
    }

    pub fn create(registry_name: &ResourceKey, location: Identifier) -> ResourceKey {
        ResourceKey::create_named(registry_name.identifier().clone(), location)
    }

    pub fn create_registry_key(identifier: Identifier) -> ResourceKey {
        ResourceKey::create_named((**Registries::ROOT_REGISTRY_NAME).clone(), identifier)
    }

    /// `create(Identifier, Identifier)`, which interns the key in Java.
    fn create_named(registry_name: Identifier, identifier: Identifier) -> ResourceKey {
        // FNV-1a over the four names, each ended by a byte no name has.
        let mut digest: u64 = 0xcbf29ce484222325;
        for name in [
            registry_name.get_namespace(),
            registry_name.get_path(),
            identifier.get_namespace(),
            identifier.get_path(),
        ] {
            for byte in name.bytes().chain([0xff]) {
                digest = (digest ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
        }
        ResourceKey(Arc::new(Names {
            registry_name,
            identifier,
            digest,
        }))
    }

    pub fn is_for(&self, registry: &ResourceKey) -> bool {
        self.0.registry_name == *registry.identifier()
    }

    /// The key itself when it's in the registry. Java returns the same key, cast.
    pub fn cast(&self, registry: &ResourceKey) -> Option<ResourceKey> {
        if self.is_for(registry) {
            Some(self.clone())
        } else {
            None
        }
    }

    /// The key in the other registry whose identifier is this one's with the suffix. Panics
    /// where `Identifier.withSuffix` throws.
    pub fn dependent(&self, registry_key: &ResourceKey, suffix: &str) -> ResourceKey {
        ResourceKey::create(registry_key, self.0.identifier.with_suffix(suffix))
    }

    pub fn identifier(&self) -> &Identifier {
        &self.0.identifier
    }

    /// The registry's name: its key's identifier.
    pub fn registry(&self) -> &Identifier {
        &self.0.registry_name
    }

    pub fn registry_key(&self) -> ResourceKey {
        ResourceKey::create_registry_key(self.0.registry_name.clone())
    }
}

/// `ResourceKey[minecraft:root / minecraft:worldgen/biome]`.
impl fmt::Display for ResourceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ResourceKey[{} / {}]",
            self.0.registry_name, self.0.identifier
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ResourceKey;
    use crate::golden::{Decode, Encode, Golden, elements};
    use crate::net::minecraft::resources::identifier::Identifier;

    // [registry, identifier].
    impl Decode for ResourceKey {
        fn decode(value: &serde_json::Value) -> Self {
            let [registry_name, identifier] = elements(value);
            ResourceKey::create_named(
                Identifier::decode(registry_name),
                Identifier::decode(identifier),
            )
        }
    }

    impl Encode for ResourceKey {
        fn encode(&self) -> serde_json::Value {
            serde_json::Value::from(vec![self.registry().encode(), self.identifier().encode()])
        }

        fn matches(&self, expected: &serde_json::Value) -> bool {
            match expected.as_array().map(Vec::as_slice) {
                Some([registry_name, identifier]) => {
                    self.registry().matches(registry_name) && self.identifier().matches(identifier)
                }
                _ => false,
            }
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.resources.ResourceKey");
        golden.pending(
            "codec(Lnet/minecraft/resources/ResourceKey;)Lcom/mojang/serialization/Codec;",
            "waits for DataFixerUpper's codecs",
        );
        golden.pending(
            "dependent(Lnet/minecraft/resources/ResourceKey;Ljava/util/function/UnaryOperator;)Lnet/minecraft/resources/ResourceKey;",
            "the golden harness can't generate a UnaryOperator",
        );
        golden.check_codec("REGISTRY_STREAM_CODEC", || {
            ResourceKey::REGISTRY_STREAM_CODEC
        });
        golden.check_codec_from(
            "streamCodec(Lnet/minecraft/resources/ResourceKey;)Lnet/minecraft/network/codec/StreamCodec;",
            |registry_name: ResourceKey| ResourceKey::stream_codec(&registry_name),
        );
        golden.check(
            "create(Lnet/minecraft/resources/ResourceKey;Lnet/minecraft/resources/Identifier;)Lnet/minecraft/resources/ResourceKey;",
            |registry_name: ResourceKey, location| ResourceKey::create(&registry_name, location),
        );
        golden.check(
            "createRegistryKey(Lnet/minecraft/resources/Identifier;)Lnet/minecraft/resources/ResourceKey;",
            ResourceKey::create_registry_key,
        );
        golden.check_method(
            "isFor(Lnet/minecraft/resources/ResourceKey;)Z",
            |this: &mut ResourceKey, registry: ResourceKey| this.is_for(&registry),
        );
        golden.check_method(
            "cast(Lnet/minecraft/resources/ResourceKey;)Ljava/util/Optional;",
            |this: &mut ResourceKey, registry: ResourceKey| this.cast(&registry),
        );
        golden.check_method(
            "dependent(Lnet/minecraft/resources/ResourceKey;Ljava/lang/String;)Lnet/minecraft/resources/ResourceKey;",
            |this: &mut ResourceKey, registry_key: ResourceKey, suffix: String| {
                this.dependent(&registry_key, &suffix)
            },
        );
        golden.check_method(
            "identifier()Lnet/minecraft/resources/Identifier;",
            |this: &mut ResourceKey| this.identifier().clone(),
        );
        golden.check_method(
            "registry()Lnet/minecraft/resources/Identifier;",
            |this: &mut ResourceKey| this.registry().clone(),
        );
        golden.check_method(
            "registryKey()Lnet/minecraft/resources/ResourceKey;",
            |this: &mut ResourceKey| this.registry_key(),
        );
        golden.check_method("toString()Ljava/lang/String;", |this: &mut ResourceKey| {
            this.to_string()
        });
        golden.finish();
    }
}
