//! net/minecraft/tags/EntityTypeTags.java
//!
//! Only the tags the blocks an entity is inside ask about. Like `BlockTags`, an entity type's tags
//! come from the server in Java (`ClientboundUpdateTagsPacket`), which isn't ported:
//! `EntityType::is_tag` answers from vanilla's data pack (`data/minecraft/tags/entity_type/`),
//! which is what a vanilla server sends.

use crate::net::minecraft::tags::block_tags;
use crate::net::minecraft::world::entity::entity_type::EntityType;

/// `TagKey<EntityType<?>>`: an entity type tag's name, under `minecraft:`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntityTypeTagKey(pub &'static str);

pub struct EntityTypeTags;

impl EntityTypeTags {
    pub const POWDER_SNOW_WALKABLE_MOBS: EntityTypeTagKey =
        EntityTypeTagKey("powder_snow_walkable_mobs");
    pub const FREEZE_IMMUNE_ENTITY_TYPES: EntityTypeTagKey =
        EntityTypeTagKey("freeze_immune_entity_types");
}

impl EntityType {
    /// `is(TagKey<EntityType<?>>)`, from vanilla's data pack until the server's tags are ported.
    pub fn is_tag(&self, tag: EntityTypeTagKey) -> bool {
        block_tags::tag_contains(
            "entity_type",
            tag.0,
            &EntityType::get_key(*self).to_string(),
        )
    }
}
