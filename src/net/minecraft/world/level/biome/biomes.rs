//! net/minecraft/world/level/biome/Biomes.java
//!
//! Only `PLAINS`, the biome a client level answers where it has no chunk; the other keys come with
//! what names them.

use std::sync::LazyLock;

use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;

static PLAINS: LazyLock<ResourceKey> = LazyLock::new(|| {
    ResourceKey::create(
        Registries::BIOME,
        Identifier::with_default_namespace("plains"),
    )
});

/// The vanilla biomes' keys.
pub struct Biomes;

impl Biomes {
    pub const PLAINS: &'static LazyLock<ResourceKey> = &PLAINS;
}
