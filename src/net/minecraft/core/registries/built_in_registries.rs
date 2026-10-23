//! net/minecraft/core/registries/BuiltInRegistries.java
//!
//! Not ported yet: every registry but the attributes' and the entity types', which come with
//! their classes (blocks' is `Block`'s own numbering, `BLOCK_STATE_REGISTRY`). A registry here is
//! its elements' keys in id order, the order Java registers them in (`Registry`). The entity
//! types' is also the registry `EntityTypes` fills, which hands out the types themselves, falling
//! back to the pig like Java's `DefaultedRegistry`.

use std::sync::LazyLock;

use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::core::registry::Registry;
use crate::net::minecraft::world::entity::ai::attributes::attributes::{self, Attributes};
use crate::net::minecraft::world::entity::entity_types::{self, EntityTypeRegistry};

static ATTRIBUTE: LazyLock<Registry> = LazyLock::new(|| {
    Registry::new(
        (**Registries::ATTRIBUTE).clone(),
        attributes::REGISTERED
            .iter()
            .map(|(_, name)| {
                Attributes::holder(name)
                    .unwrap_key()
                    .expect("a reference has its key")
                    .clone()
            })
            .collect(),
    )
});

/// The registries the client has built in, which the server doesn't send.
pub struct BuiltInRegistries;

impl BuiltInRegistries {
    pub const ATTRIBUTE: &'static LazyLock<Registry> = &ATTRIBUTE;
    pub const ENTITY_TYPE: &'static LazyLock<EntityTypeRegistry> = &entity_types::REGISTRY;

    /// The ones ported, which the client's registries start from (`ClientRegistryLayer.STATIC`).
    pub fn registries() -> Vec<Registry> {
        vec![
            Registry::clone(BuiltInRegistries::ATTRIBUTE),
            Registry::new(
                (**Registries::ENTITY_TYPE).clone(),
                BuiltInRegistries::ENTITY_TYPE.keys().to_vec(),
            ),
        ]
    }
}
