//! net/minecraft/world/level/block/SlimeBlock.java
//!
//! Its blocks' states are its parent's. Of its behaviour, `stepOn`: walking on it, an entity that
//! isn't stepping carefully is slowed down the less it moves up or down. Its bounce is the block's
//! restitution (`Entity.restituteMovementAfterCollisions`), and its `fallOn` hurts on the server
//! only (no damage unless the entity suppresses the bounce).

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::half_transparent_block::{
    self, HalfTransparentBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::level::Level;

pub struct SlimeBlock;

pub static CLASS: BlockClass = BlockClass {
    step_on: Some(SlimeBlock::step_on),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SlimeBlock",
        &half_transparent_block::CLASS,
    )
};

impl SlimeBlock {
    pub fn new(properties: Properties) -> BlockData {
        SlimeBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        HalfTransparentBlock::new_with(properties, class, definition)
    }

    /// `stepOn`, then `Block`'s, which does nothing.
    fn step_on(
        _block: &BlockData,
        _level: &dyn Level,
        _pos: &BlockPos,
        _on_state: BlockState,
        entity: &mut dyn EntityClass,
    ) {
        let abs_delta_y = entity.entity().get_delta_movement().y.abs();
        if abs_delta_y < 0.1 && !entity.is_stepping_carefully() {
            let scale = 0.4 + abs_delta_y * 0.2;
            let movement = entity
                .entity()
                .get_delta_movement()
                .multiply_xyz(scale, 1.0, scale);
            entity.entity_mut().set_delta_movement(movement);
        }
    }
}
