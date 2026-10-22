//! net/minecraft/world/level/block/LeverBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::face_attached_horizontal_directional_block::{
    self, FaceAttachedHorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::attach_face::AttachFace;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct LeverBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.LeverBlock",
    &face_attached_horizontal_directional_block::CLASS,
);

impl LeverBlock {
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;

    pub fn new(properties: Properties) -> BlockData {
        LeverBlock::new_with(
            properties,
            &CLASS,
            LeverBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block =
            FaceAttachedHorizontalDirectionalBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(HorizontalDirectionalBlock::FACING, Direction::NORTH)
                .set_value(LeverBlock::POWERED, false)
                .set_value(
                    FaceAttachedHorizontalDirectionalBlock::FACE,
                    AttachFace::WALL,
                ),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            FaceAttachedHorizontalDirectionalBlock::FACE,
            HorizontalDirectionalBlock::FACING,
            LeverBlock::POWERED,
        ]);
    }
}
