//! net/minecraft/world/level/block/FireBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shape (a sixteenth on each face it burns on, or on the ground) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_fire_block::{self, BaseFireBlock};
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::pipe_block::PipeBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct FireBlock {
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(FireBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.FireBlock",
        &base_fire_block::CLASS,
    )
};

impl FireBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_15;
    pub const NORTH: &'static BooleanProperty = PipeBlock::NORTH;
    pub const EAST: &'static BooleanProperty = PipeBlock::EAST;
    pub const SOUTH: &'static BooleanProperty = PipeBlock::SOUTH;
    pub const WEST: &'static BooleanProperty = PipeBlock::WEST;
    pub const UP: &'static BooleanProperty = PipeBlock::UP;

    pub fn new(properties: Properties) -> BlockData {
        FireBlock::new_with(properties, &CLASS, FireBlock::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseFireBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(FireBlock::AGE, 0)
                .set_value(FireBlock::NORTH, false)
                .set_value(FireBlock::EAST, false)
                .set_value(FireBlock::SOUTH, false)
                .set_value(FireBlock::WEST, false)
                .set_value(FireBlock::UP, false),
        );
        let shapes = FireBlock::make_shapes(&block);
        block.set_fields(FireBlock { shapes });
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            FireBlock::AGE,
            FireBlock::NORTH,
            FireBlock::EAST,
            FireBlock::SOUTH,
            FireBlock::WEST,
            FireBlock::UP,
        ]);
    }
}

impl FireBlock {
    /// `PROPERTY_BY_DIRECTION`: `PipeBlock`'s, but down.
    pub const PROPERTY_BY_DIRECTION: [(Direction, &'static BooleanProperty); 5] = [
        (Direction::UP, FireBlock::UP),
        (Direction::NORTH, FireBlock::NORTH),
        (Direction::SOUTH, FireBlock::SOUTH),
        (Direction::WEST, FireBlock::WEST),
        (Direction::EAST, FireBlock::EAST),
    ];

    /// `makeShapes`: a sixteenth on each face it burns on, or `BaseFireBlock`'s on the ground,
    /// for each state but for its age. Java's map of properties is a `HashMap` of directions,
    /// whose order changes from run to run and not the union.
    fn make_shapes(block: &BlockData) -> Vec<VoxelShape> {
        let shapes = Shapes::rotate_all(&Block::box_z_square(16.0, 0.0, 1.0));
        Block::get_shape_for_each_state_ignoring(
            block,
            |state| {
                let mut shape = Shapes::empty();
                for (direction, property) in FireBlock::PROPERTY_BY_DIRECTION {
                    if state.get_value(property) {
                        shape = Shapes::or(&shape, Shapes::get(&shapes, direction));
                    }
                }
                if shape.is_empty() {
                    base_fire_block::SHAPE.clone()
                } else {
                    shape
                }
            },
            &[FireBlock::AGE],
        )
    }

    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<FireBlock>().shapes[state.index() as usize].clone()
    }
}
