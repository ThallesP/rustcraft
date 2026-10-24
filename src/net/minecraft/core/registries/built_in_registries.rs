//! net/minecraft/core/registries/BuiltInRegistries.java
//!
//! Not ported yet: every registry but the attributes', the blocks', the entity types', the
//! items' and the data component types' (their names: `Items`, `DataComponents`), which come with
//! their classes (blocks' is `blocks::registry()`, with `BLOCK_STATE_REGISTRY`). A registry here is
//! its elements' keys in id order, the order Java registers them in (`Registry`). The entity
//! types' is also the registry `EntityTypes` fills, which hands out the types themselves, falling
//! back to the pig like Java's `DefaultedRegistry`.

use std::sync::LazyLock;

use crate::net::minecraft::core::component::data_components;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::core::registry::Registry;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::world::entity::ai::attributes::attributes::{self, Attributes};
use crate::net::minecraft::world::entity::entity_types::{self, EntityTypeRegistry};
use crate::net::minecraft::world::item::items;
use crate::net::minecraft::world::level::block::blocks;

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

static ITEM: LazyLock<Registry> = LazyLock::new(|| {
    Registry::new(
        (**Registries::ITEM).clone(),
        items::NAMES
            .iter()
            .map(|name| {
                ResourceKey::create(Registries::ITEM, Identifier::with_default_namespace(name))
            })
            .collect(),
    )
});

static DATA_COMPONENT_TYPE: LazyLock<Registry> = LazyLock::new(|| {
    Registry::new(
        (**Registries::DATA_COMPONENT_TYPE).clone(),
        data_components::TYPES
            .iter()
            .map(|(name, _)| {
                ResourceKey::create(
                    Registries::DATA_COMPONENT_TYPE,
                    Identifier::with_default_namespace(name),
                )
            })
            .collect(),
    )
});

/// The registries the client has built in, which the server doesn't send.
pub struct BuiltInRegistries;

impl BuiltInRegistries {
    pub const ATTRIBUTE: &'static LazyLock<Registry> = &ATTRIBUTE;
    pub const ENTITY_TYPE: &'static LazyLock<EntityTypeRegistry> = &entity_types::REGISTRY;
    pub const ITEM: &'static LazyLock<Registry> = &ITEM;
    pub const DATA_COMPONENT_TYPE: &'static LazyLock<Registry> = &DATA_COMPONENT_TYPE;

    /// The ones ported, which the client's registries start from (`ClientRegistryLayer.STATIC`).
    pub fn registries() -> Vec<Registry> {
        vec![
            Registry::clone(BuiltInRegistries::ATTRIBUTE),
            Registry::new(
                (**Registries::BLOCK).clone(),
                blocks::registry().keys().to_vec(),
            ),
            Registry::new(
                (**Registries::ENTITY_TYPE).clone(),
                BuiltInRegistries::ENTITY_TYPE.keys().to_vec(),
            ),
        ]
    }
}
