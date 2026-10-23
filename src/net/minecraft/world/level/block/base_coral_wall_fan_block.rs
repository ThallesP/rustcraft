//! net/minecraft/world/level/block/BaseCoralWallFanBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shape (against the wall it faces from) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_coral_fan_block::{self, BaseCoralFanBlock};
use crate::net::minecraft::world::level::block::base_coral_plant_type_block::BaseCoralPlantTypeBlock;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct BaseCoralWallFanBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(BaseCoralWallFanBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BaseCoralWallFanBlock",
        &base_coral_fan_block::CLASS,
    )
};

impl BaseCoralWallFanBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;

    pub fn new(properties: Properties) -> BlockData {
        BaseCoralWallFanBlock::new_with(
            properties,
            &CLASS,
            BaseCoralWallFanBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseCoralFanBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(BaseCoralWallFanBlock::FACING, Direction::NORTH)
                .set_value(BaseCoralPlantTypeBlock::WATERLOGGED, true),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            BaseCoralWallFanBlock::FACING,
            BaseCoralPlantTypeBlock::WATERLOGGED,
        ]);
    }
}

static SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal(&Block::box_z_sized(16.0, 8.0, 5.0, 16.0)));

impl BaseCoralWallFanBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(&SHAPES, state.get_value(BaseCoralWallFanBlock::FACING)).clone()
    }
}
