//! net/minecraft/world/level/block/PipeBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `size`.
//!
//! Of its behaviour, `propagatesSkylightDown` and its shape (a core as wide as its constructor's
//! `size`, with an arm toward each side it connects to) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct PipeBlock {
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(PipeBlock::get_shape),
    propagates_skylight_down: Some(PipeBlock::propagates_skylight_down),
    ..BlockClass::new("net.minecraft.world.level.block.PipeBlock", &block::CLASS)
};

impl PipeBlock {
    /// `PROPERTY_BY_DIRECTION`, an `EnumMap`: each direction's property, by the direction's
    /// ordinal.
    pub const PROPERTY_BY_DIRECTION: [&'static BooleanProperty; 6] = [
        PipeBlock::DOWN,
        PipeBlock::UP,
        PipeBlock::NORTH,
        PipeBlock::SOUTH,
        PipeBlock::WEST,
        PipeBlock::EAST,
    ];

    pub const NORTH: &'static BooleanProperty = BlockStateProperties::NORTH;
    pub const EAST: &'static BooleanProperty = BlockStateProperties::EAST;
    pub const SOUTH: &'static BooleanProperty = BlockStateProperties::SOUTH;
    pub const WEST: &'static BooleanProperty = BlockStateProperties::WEST;
    pub const UP: &'static BooleanProperty = BlockStateProperties::UP;
    pub const DOWN: &'static BooleanProperty = BlockStateProperties::DOWN;

    pub(crate) fn new_with(
        size: f32,
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        let shapes = PipeBlock::make_shapes(&block, size);
        block.set_fields(PipeBlock { shapes });
        block
    }
}

impl PipeBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        false
    }
}

impl PipeBlock {
    /// `makeShapes`: the core, with an arm toward each side it connects to, joined in
    /// `PROPERTY_BY_DIRECTION`'s order.
    fn make_shapes(block: &BlockData, size: f32) -> Vec<VoxelShape> {
        let core = Block::cube(f64::from(size));
        let shapes = Shapes::rotate_all(&Block::box_z_square(f64::from(size), 0.0, 8.0));
        Block::get_shape_for_each_state(block, |state| {
            let mut shape = core.clone();
            for (direction, property) in Direction::VALUES
                .into_iter()
                .zip(PipeBlock::PROPERTY_BY_DIRECTION)
            {
                if state.get_value(property) {
                    shape = Shapes::or(Shapes::get(&shapes, direction), &shape);
                }
            }
            shape
        })
    }

    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<PipeBlock>().shapes[state.index() as usize].clone()
    }
}
