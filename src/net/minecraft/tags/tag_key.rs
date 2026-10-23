//! net/minecraft/tags/TagKey.java
//!
//! A tag's name in a registry. Java interns its keys like `ResourceKey`'s, so here a key is a
//! value that compares its names. `codec`, `streamCodec`, `isFor` and `cast` come with their
//! callers. Block tags are still `block_tags::TagKey`, the names `BlockState::is_tag` answers from
//! the vanilla data pack until `update_tags` is ported.

use std::fmt;

use crate::com::mojang::serialization::codec::{BoxedCodec, CodecExt, STRING};
use crate::com::mojang::serialization::data_result::DataResult;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TagKey {
    registry: ResourceKey,
    location: Identifier,
}

impl TagKey {
    pub fn create(registry: &ResourceKey, location: Identifier) -> TagKey {
        TagKey {
            registry: registry.clone(),
            location,
        }
    }

    /// `hashedCodec`: `#` and the tag's identifier.
    pub fn hashed_codec(registry_name: &ResourceKey) -> BoxedCodec<TagKey> {
        let registry_name = registry_name.clone();
        STRING
            .comap_flat_map(
                move |name: String| match name.strip_prefix('#') {
                    Some(id) => Identifier::read(id).map(|id| TagKey::create(&registry_name, id)),
                    None => DataResult::error(|| "Not a tag id".to_owned()),
                },
                |e: &TagKey| format!("#{}", e.location),
            )
            .boxed()
    }

    pub fn registry(&self) -> &ResourceKey {
        &self.registry
    }

    pub fn location(&self) -> &Identifier {
        &self.location
    }
}

impl fmt::Display for TagKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TagKey[{} / {}]",
            self.registry.identifier(),
            self.location
        )
    }
}
