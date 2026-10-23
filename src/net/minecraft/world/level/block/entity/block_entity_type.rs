//! net/minecraft/world/level/block/entity/BlockEntityType.java
//!
//! Java's types are objects the game compares by reference, numbered by
//! `BuiltInRegistries.BLOCK_ENTITY_TYPE`, so here a type is its id there, `Copy`, like a `Block`.
//! Its factory and valid blocks are `BlockEntityTypes`' tables, and `create` makes a block entity
//! of the type's class (`BlockEntityData`). Not ported yet: `onlyOpCanSetNbt`.

use std::fmt;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::item::dye_color::DyeColor;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::entity::banner_block_entity::BannerBlockEntity;
use crate::net::minecraft::world::level::block::entity::block_entity::{
    BlockEntity, BlockEntityData,
};
use crate::net::minecraft::world::level::block::entity::block_entity_types::{
    BlockEntityTypes, CLASS_NAMES, KEYS, VALID_BLOCKS,
};
use crate::net::minecraft::world::level::block::entity::brushable_block_entity::BrushableBlockEntity;
use crate::net::minecraft::world::level::block::entity::campfire_block_entity::CampfireBlockEntity;
use crate::net::minecraft::world::level::block::entity::shulker_box_block_entity::ShulkerBoxBlockEntity;
use crate::net::minecraft::world::level::block::entity::sign_block_entity::SignBlockEntity;
use crate::net::minecraft::world::level::block::entity::skull_block_entity::SkullBlockEntity;
use crate::net::minecraft::world::level::block::entity::spawner_block_entity::SpawnerBlockEntity;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

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

    /// The class `register` gave it a factory of, by its canonical name.
    pub fn class_name(&self) -> &'static str {
        CLASS_NAMES[self.0 as usize]
    }

    /// The blocks it's valid for, in `register`'s order.
    pub fn valid_blocks(&self) -> &'static [Block] {
        &VALID_BLOCKS[self.0 as usize]
    }

    /// `isValid(state)`: whether its valid blocks have the state's block.
    pub fn is_valid(&self, state: BlockState) -> bool {
        self.valid_blocks().contains(&state.get_block())
    }

    /// `create(worldPosition, blockState)`: a block entity of its class, as the class's
    /// constructor makes it. Panics, where Java throws, for a state it isn't valid for.
    pub fn create(&self, world_position: BlockPos, block_state: BlockState) -> BlockEntity {
        let mut entity = BlockEntity::new(*self, world_position, block_state);
        let block = block_state.get_block();
        entity.data = match *self {
            BlockEntityTypes::SIGN | BlockEntityTypes::HANGING_SIGN => {
                BlockEntityData::Sign(SignBlockEntity::new())
            }
            BlockEntityTypes::BANNER => {
                // AbstractBannerBlock.getColor: the colour whose banner, standing or on a wall, it is.
                let color = color_of(&Blocks::BANNER, block)
                    .or_else(|| color_of(&Blocks::WALL_BANNER, block))
                    .expect("a banner's block is one of the banners");
                BlockEntityData::Banner(BannerBlockEntity::new(color))
            }
            BlockEntityTypes::SKULL => BlockEntityData::Skull(SkullBlockEntity::new()),
            BlockEntityTypes::MOB_SPAWNER => BlockEntityData::Spawner(SpawnerBlockEntity::new()),
            BlockEntityTypes::CAMPFIRE => BlockEntityData::Campfire(CampfireBlockEntity::new()),
            BlockEntityTypes::BRUSHABLE_BLOCK => {
                BlockEntityData::Brushable(BrushableBlockEntity::new())
            }
            // ShulkerBoxBlock.getColorFromBlock: a dyed box's colour, none for the plain one.
            BlockEntityTypes::SHULKER_BOX => BlockEntityData::ShulkerBox(
                ShulkerBoxBlockEntity::new(color_of(&Blocks::DYED_SHULKER_BOX, block)),
            ),
            _ => BlockEntityData::None,
        };
        entity
    }
}

/// The colour a collection of coloured blocks holds the block under, if it does.
fn color_of(
    collection: &crate::net::minecraft::world::level::block::color_collection::ColorCollection<
        Block,
    >,
    block: Block,
) -> Option<DyeColor> {
    DyeColor::VALUES
        .into_iter()
        .find(|&color| *collection.pick(color) == block)
}

impl fmt::Debug for BlockEntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::BlockEntityType;
    use crate::golden::Golden;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::world::level::block::blocks;
    use crate::net::minecraft::world::level::block::entity::block_entity_types::BY_BLOCK;

    /// Each block's type (what `newBlockEntity` makes) and each type's valid blocks.
    #[test]
    fn types_match_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.block.entity.BlockEntityType");
        let registry = blocks::registry();
        let new_block_entity = golden.cases("newBlockEntity");
        assert_eq!(new_block_entity.len(), registry.size());
        for (args, expected) in new_block_entity {
            let name = Identifier::parse(args[0].as_str().expect("a block"));
            let block = registry.get_value(&name).expect("a block the registry has");
            let actual =
                BY_BLOCK[block.0 as usize].map_or(Value::Null, |r#type| json!(r#type.key()));
            assert_eq!(actual, expected, "{name}");
        }
        let valid_blocks = golden.cases("validBlocks");
        assert_eq!(
            valid_blocks.len(),
            BlockEntityType::registry_size() as usize
        );
        for (id, (args, expected)) in valid_blocks.into_iter().enumerate() {
            let r#type = BlockEntityType::by_id_or_throw(id as i32);
            assert_eq!(args[0], json!(r#type.key()));
            let mut blocks = r#type.valid_blocks().to_vec();
            blocks.sort_by_key(|block| block.0);
            let actual: Vec<String> = blocks
                .into_iter()
                .map(|block| registry.get_key(block).to_string())
                .collect();
            assert_eq!(json!(actual), expected, "{}", r#type.key());
        }
        golden.finish();
    }
}
