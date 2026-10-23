//! net/minecraft/world/level/block/entity/BlockEntityType.java
//!
//! Java's types are objects the game compares by reference, numbered by
//! `BuiltInRegistries.BLOCK_ENTITY_TYPE`, so here a type is its id there, `Copy`, like a `Block`.
//! Not ported yet: `create`, `isValid`, `getBlockEntity` and `onlyOpCanSetNbt`, which come with
//! block entities.

use std::fmt;

use crate::net::minecraft::world::level::block::entity::block_entity_types::KEYS;

/// A kind of block entity, by its id in `BuiltInRegistries.BLOCK_ENTITY_TYPE`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockEntityType(pub(crate) u16);

impl BlockEntityType {
    pub fn id(&self) -> i32 {
        self.0 as i32
    }

    /// Its name in the registry, `minecraft:chest`.
    pub fn key(&self) -> &'static str {
        KEYS[self.0 as usize]
    }

    /// `BuiltInRegistries.BLOCK_ENTITY_TYPE.byId(id)`.
    pub fn by_id(id: i32) -> Option<BlockEntityType> {
        usize::try_from(id)
            .ok()
            .filter(|&id| id < KEYS.len())
            .map(|id| BlockEntityType(id as u16))
    }

    /// `BuiltInRegistries.BLOCK_ENTITY_TYPE.byIdOrThrow(id)`: panics for an id it doesn't have,
    /// where Java throws `IllegalArgumentException`.
    pub fn by_id_or_throw(id: i32) -> BlockEntityType {
        BlockEntityType::by_id(id).unwrap_or_else(|| panic!("No value with id {id}"))
    }

    pub fn registry_size() -> i32 {
        KEYS.len() as i32
    }
}

impl fmt::Debug for BlockEntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}
