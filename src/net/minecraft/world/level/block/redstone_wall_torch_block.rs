//! net/minecraft/world/level/block/RedstoneWallTorchBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shape (a wall torch's) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::redstone_torch_block::{self, RedstoneTorchBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::wall_torch_block::WallTorchBlock;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct RedstoneWallTorchBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(RedstoneWallTorchBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.RedstoneWallTorchBlock",
        &redstone_torch_block::CLASS,
    )
};

impl RedstoneWallTorchBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;
    pub const LIT: &'static BooleanProperty = RedstoneTorchBlock::LIT;

    pub fn new(properties: Properties) -> BlockData {
        RedstoneWallTorchBlock::new_with(
            properties,
            &CLASS,
            RedstoneWallTorchBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = RedstoneTorchBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(RedstoneWallTorchBlock::FACING, Direction::NORTH)
                .set_value(RedstoneWallTorchBlock::LIT, true),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[RedstoneWallTorchBlock::FACING, RedstoneWallTorchBlock::LIT]);
    }
}

impl RedstoneWallTorchBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        WallTorchBlock::get_shape_of(state)
    }
}
