//! net/minecraft/world/level/block/TripWireBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `hook`.
//!
//! Of its behaviour, its shape (lower once it's attached), which is also what an entity has to
//! touch to be in it, is ported. What it does to an entity inside (`entityInside`) is the
//! server's.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::pipe_block::PipeBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct TripWireBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(TripWireBlock::get_shape),
    get_entity_inside_collision_shape: Some(TripWireBlock::get_entity_inside_collision_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.TripWireBlock",
        &block::CLASS,
    )
};

impl TripWireBlock {
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;
    pub const ATTACHED: &'static BooleanProperty = BlockStateProperties::ATTACHED;
    pub const DISARMED: &'static BooleanProperty = BlockStateProperties::DISARMED;
    pub const NORTH: &'static BooleanProperty = PipeBlock::NORTH;
    pub const EAST: &'static BooleanProperty = PipeBlock::EAST;
    pub const SOUTH: &'static BooleanProperty = PipeBlock::SOUTH;
    pub const WEST: &'static BooleanProperty = PipeBlock::WEST;

    pub fn new(properties: Properties) -> BlockData {
        TripWireBlock::new_with(
            properties,
            &CLASS,
            TripWireBlock::create_block_state_definition,
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
                .set_value(TripWireBlock::POWERED, false)
                .set_value(TripWireBlock::ATTACHED, false)
                .set_value(TripWireBlock::DISARMED, false)
                .set_value(TripWireBlock::NORTH, false)
                .set_value(TripWireBlock::EAST, false)
                .set_value(TripWireBlock::SOUTH, false)
                .set_value(TripWireBlock::WEST, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            TripWireBlock::POWERED,
            TripWireBlock::ATTACHED,
            TripWireBlock::DISARMED,
            TripWireBlock::NORTH,
            TripWireBlock::EAST,
            TripWireBlock::WEST,
            TripWireBlock::SOUTH,
        ]);
    }
}

static SHAPE_ATTACHED: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 1.0, 2.5));
static SHAPE_NOT_ATTACHED: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 8.0));

impl TripWireBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        if state.get_value(TripWireBlock::ATTACHED) {
            SHAPE_ATTACHED.clone()
        } else {
            SHAPE_NOT_ATTACHED.clone()
        }
    }

    /// `getEntityInsideCollisionShape`: its shape.
    fn get_entity_inside_collision_shape(
        _block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        _entity: &dyn EntityClass,
    ) -> VoxelShape {
        state.get_shape(level, pos)
    }
}
