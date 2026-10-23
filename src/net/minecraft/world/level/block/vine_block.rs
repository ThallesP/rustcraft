//! net/minecraft/world/level/block/VineBlock.java
//!
//! What decides its blocks' states (its properties and its default state) and its shape: a
//! sixteenth-thick sheet on each side it grows on, or a whole block where it grows on none, worked
//! out for each state when it's made (`makeShapes`). Vines have no collision (`noCollision` in
//! `Blocks`), so the shape is for outlines only. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, `propagatesSkylightDown` is ported, for the light engine.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::pipe_block::PipeBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct VineBlock {
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    propagates_skylight_down: Some(VineBlock::propagates_skylight_down),
    get_shape: Some(VineBlock::get_shape),
    ..BlockClass::new("net.minecraft.world.level.block.VineBlock", &block::CLASS)
};

impl VineBlock {
    pub const UP: &'static BooleanProperty = PipeBlock::UP;
    pub const NORTH: &'static BooleanProperty = PipeBlock::NORTH;
    pub const EAST: &'static BooleanProperty = PipeBlock::EAST;
    pub const SOUTH: &'static BooleanProperty = PipeBlock::SOUTH;
    pub const WEST: &'static BooleanProperty = PipeBlock::WEST;

    /// `PROPERTY_BY_DIRECTION`: `PipeBlock`'s but down's. Java collects them into a `HashMap`
    /// (`Util.toMap`), whose order the union `makeShapes` makes of them doesn't depend on.
    pub const PROPERTY_BY_DIRECTION: [(Direction, &'static BooleanProperty); 5] = [
        (Direction::UP, VineBlock::UP),
        (Direction::NORTH, VineBlock::NORTH),
        (Direction::SOUTH, VineBlock::SOUTH),
        (Direction::WEST, VineBlock::WEST),
        (Direction::EAST, VineBlock::EAST),
    ];

    pub fn new(properties: Properties) -> BlockData {
        VineBlock::new_with(properties, &CLASS, VineBlock::create_block_state_definition)
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
                .set_value(VineBlock::UP, false)
                .set_value(VineBlock::NORTH, false)
                .set_value(VineBlock::EAST, false)
                .set_value(VineBlock::SOUTH, false)
                .set_value(VineBlock::WEST, false),
        );
        let shapes = VineBlock::make_shapes(&block);
        block.set_fields(VineBlock { shapes });
        block
    }

    fn make_shapes(block: &BlockData) -> Vec<VoxelShape> {
        let shapes = Shapes::rotate_all(&Block::box_z_square(16.0, 0.0, 1.0));
        Block::get_shape_for_each_state(block, |state| {
            let mut shape = Shapes::empty();
            for (direction, property) in VineBlock::PROPERTY_BY_DIRECTION {
                if state.get_value(property) {
                    shape = Shapes::or(&shape, Shapes::get(&shapes, direction));
                }
            }

            if shape.is_empty() {
                Shapes::block()
            } else {
                shape
            }
        })
    }

    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<VineBlock>().shapes[state.index() as usize].clone()
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            VineBlock::UP,
            VineBlock::NORTH,
            VineBlock::EAST,
            VineBlock::SOUTH,
            VineBlock::WEST,
        ]);
    }
}

impl VineBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
