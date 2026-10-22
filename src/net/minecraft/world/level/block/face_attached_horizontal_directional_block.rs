//! net/minecraft/world/level/block/FaceAttachedHorizontalDirectionalBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::{
    self, HorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::attach_face::AttachFace;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;

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
}
