//! net/minecraft/world/level/block/WallSkullBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `type`.
//!
//! Of its behaviour, its shape (against the wall it faces from) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::abstract_skull_block::{self, AbstractSkullBlock};
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::skull_block::Types;
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

pub struct WallSkullBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(WallSkullBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WallSkullBlock",
        &abstract_skull_block::CLASS,
    )
};

impl WallSkullBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;

    pub fn new(skull_type: Types, properties: Properties) -> BlockData {
        WallSkullBlock::new_with(
            skull_type,
            properties,
            &CLASS,
            WallSkullBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        skull_type: Types,
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = AbstractSkullBlock::new_with(skull_type, properties, class, definition);
        block.register_default_state(
            block
                .default_block_state()
                .set_value(WallSkullBlock::FACING, Direction::NORTH),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        AbstractSkullBlock::create_block_state_definition(builder);
        builder.add(&[WallSkullBlock::FACING]);
    }
}

static SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal(&Block::box_z_square(8.0, 8.0, 16.0)));

impl WallSkullBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(&SHAPES, state.get_value(WallSkullBlock::FACING)).clone()
    }
}
