//! net/minecraft/world/level/block/TripWireHookBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shape (against the wall it faces from) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct TripWireHookBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(TripWireHookBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.TripWireHookBlock",
        &block::CLASS,
    )
};

impl TripWireHookBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;
    pub const ATTACHED: &'static BooleanProperty = BlockStateProperties::ATTACHED;

    pub fn new(properties: Properties) -> BlockData {
        TripWireHookBlock::new_with(
            properties,
            &CLASS,
            TripWireHookBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(TripWireHookBlock::FACING, Direction::NORTH)
                .set_value(TripWireHookBlock::POWERED, false)
                .set_value(TripWireHookBlock::ATTACHED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            TripWireHookBlock::FACING,
            TripWireHookBlock::POWERED,
            TripWireHookBlock::ATTACHED,
        ]);
    }
}

static SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal(&Block::box_z(6.0, 0.0, 10.0, 10.0, 16.0)));

impl TripWireHookBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(&SHAPES, state.get_value(TripWireHookBlock::FACING)).clone()
    }
}
