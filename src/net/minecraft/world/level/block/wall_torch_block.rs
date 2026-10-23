//! net/minecraft/world/level/block/WallTorchBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `flameParticle`.
//!
//! Of its behaviour, its shape (leaning from the wall it faces from) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
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
use crate::net::minecraft::world::level::block::torch_block::{self, TorchBlock};
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct WallTorchBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(WallTorchBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WallTorchBlock",
        &torch_block::CLASS,
    )
};

impl WallTorchBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;

    pub fn new(properties: Properties) -> BlockData {
        WallTorchBlock::new_with(
            properties,
            &CLASS,
            WallTorchBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = TorchBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(WallTorchBlock::FACING, Direction::NORTH),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[WallTorchBlock::FACING]);
    }
}

static SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal(&Block::box_z(5.0, 3.0, 13.0, 11.0, 16.0)));

impl WallTorchBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        WallTorchBlock::get_shape_of(state)
    }

    /// The static `getShape(BlockState)`, which redstone wall torches answer too.
    pub fn get_shape_of(state: BlockState) -> VoxelShape {
        Shapes::get(&SHAPES, state.get_value(WallTorchBlock::FACING)).clone()
    }
}
