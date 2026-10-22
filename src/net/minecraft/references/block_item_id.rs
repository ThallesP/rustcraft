//! net/minecraft/references/BlockItemId.java

use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;

/// The ids of a block and of its item, `minecraft:wheat` and `minecraft:wheat_seeds`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockItemId {
    block: ResourceKey,
    item: ResourceKey,
}

impl BlockItemId {
    pub fn new(block: ResourceKey, item: ResourceKey) -> BlockItemId {
        BlockItemId { block, item }
    }

    pub fn block(&self) -> &ResourceKey {
        &self.block
    }

    pub fn item(&self) -> &ResourceKey {
        &self.item
    }

    /// `create(Identifier, Identifier)`.
    pub fn create_from_identifiers(block_id: Identifier, item_id: Identifier) -> BlockItemId {
        BlockItemId::new(
            ResourceKey::create(Registries::BLOCK, block_id),
            ResourceKey::create(Registries::ITEM, item_id),
        )
    }

    /// `create(String, String)`: a block and an item named differently, in the default
    /// namespace.
    pub fn create_named(block_name: &str, item_name: &str) -> BlockItemId {
        BlockItemId::create_from_identifiers(
            Identifier::with_default_namespace(block_name),
            Identifier::with_default_namespace(item_name),
        )
    }

    /// `create(String)`: a block and its item of the same name, in the default namespace.
    pub fn create(name: &str) -> BlockItemId {
        let id = Identifier::with_default_namespace(name);
        BlockItemId::create_from_identifiers(id.clone(), id)
    }
}
