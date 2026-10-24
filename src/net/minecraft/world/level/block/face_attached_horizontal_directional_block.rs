//! net/minecraft/world/level/block/FaceAttachedHorizontalDirectionalBlock.java
//!
//! Its blocks' states are its parent's, and `getConnectedDirection` is ported. Not ported yet: the
//! rest of its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::{
    self, HorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::attach_face::AttachFace;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct FaceAttachedHorizontalDirectionalBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.FaceAttachedHorizontalDirectionalBlock",
    &horizontal_directional_block::CLASS,
);

impl FaceAttachedHorizontalDirectionalBlock {
    pub const FACE: &'static EnumProperty<AttachFace> = BlockStateProperties::ATTACH_FACE;

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        HorizontalDirectionalBlock::new_with(properties, class, definition)
    }

    /// `getConnectedDirection`: the way from it to what it's attached to's opposite: up on a
    /// floor, down on a ceiling, its facing on a wall.
    pub fn get_connected_direction(state: BlockState) -> Direction {
        match state.get_value(FaceAttachedHorizontalDirectionalBlock::FACE) {
            AttachFace::CEILING => Direction::DOWN,
            AttachFace::FLOOR => Direction::UP,
            AttachFace::WALL => state.get_value(HorizontalDirectionalBlock::FACING),
        }
    }
}
