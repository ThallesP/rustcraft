//! net/minecraft/world/level/block/MossyCarpetBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `propagatesSkylightDown` and its shapes (a carpet, with low or tall sheets up
//! the sides it covers, or the whole block for none; only the carpet to collide with) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::wall_side::WallSide;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct MossyCarpetBlock {
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(MossyCarpetBlock::get_shape),
    get_collision_shape: Some(MossyCarpetBlock::get_collision_shape),
    propagates_skylight_down: Some(MossyCarpetBlock::propagates_skylight_down),
    ..BlockClass::new(
        "net.minecraft.world.level.block.MossyCarpetBlock",
        &block::CLASS,
    )
};

impl MossyCarpetBlock {
    pub const BASE: &'static BooleanProperty = BlockStateProperties::BOTTOM;
    pub const NORTH: &'static EnumProperty<WallSide> = BlockStateProperties::NORTH_WALL;
    pub const EAST: &'static EnumProperty<WallSide> = BlockStateProperties::EAST_WALL;
    pub const SOUTH: &'static EnumProperty<WallSide> = BlockStateProperties::SOUTH_WALL;
    pub const WEST: &'static EnumProperty<WallSide> = BlockStateProperties::WEST_WALL;

    pub fn new(properties: Properties) -> BlockData {
        MossyCarpetBlock::new_with(
            properties,
            &CLASS,
            MossyCarpetBlock::create_block_state_definition,
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
                .set_value(MossyCarpetBlock::BASE, true)
                .set_value(MossyCarpetBlock::NORTH, WallSide::NONE)
                .set_value(MossyCarpetBlock::EAST, WallSide::NONE)
                .set_value(MossyCarpetBlock::SOUTH, WallSide::NONE)
                .set_value(MossyCarpetBlock::WEST, WallSide::NONE),
        );
        let shapes = MossyCarpetBlock::make_shapes(&block);
        block.set_fields(MossyCarpetBlock { shapes });
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            MossyCarpetBlock::BASE,
            MossyCarpetBlock::NORTH,
            MossyCarpetBlock::EAST,
            MossyCarpetBlock::SOUTH,
            MossyCarpetBlock::WEST,
        ]);
    }
}

impl MossyCarpetBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}

impl MossyCarpetBlock {
    /// `PROPERTY_BY_DIRECTION`, an `ImmutableMap` in this order.
    pub const PROPERTY_BY_DIRECTION: [(Direction, &'static EnumProperty<WallSide>); 4] = [
        (Direction::NORTH, MossyCarpetBlock::NORTH),
        (Direction::EAST, MossyCarpetBlock::EAST),
        (Direction::SOUTH, MossyCarpetBlock::SOUTH),
        (Direction::WEST, MossyCarpetBlock::WEST),
    ];

    /// `makeShapes`: the carpet when it has a base, and a low or a tall sheet up each side.
    fn make_shapes(block: &BlockData) -> Vec<VoxelShape> {
        let low = Shapes::rotate_horizontal(&Block::box_z(16.0, 0.0, 10.0, 0.0, 1.0));
        let tall = Shapes::rotate_all(&Block::box_z_square(16.0, 0.0, 1.0));
        Block::get_shape_for_each_state(block, |state| {
            let mut shape = if state.get_value(MossyCarpetBlock::BASE) {
                Shapes::get(&tall, Direction::DOWN).clone()
            } else {
                Shapes::empty()
            };
            for (direction, property) in MossyCarpetBlock::PROPERTY_BY_DIRECTION {
                match state.get_value(property) {
                    WallSide::NONE => {}
                    WallSide::LOW => shape = Shapes::or(&shape, Shapes::get(&low, direction)),
                    WallSide::TALL => shape = Shapes::or(&shape, Shapes::get(&tall, direction)),
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
        block.fields::<MossyCarpetBlock>().shapes[state.index() as usize].clone()
    }

    /// `getCollisionShape`: the default state's shape while it has a base.
    fn get_collision_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        if state.get_value(MossyCarpetBlock::BASE) {
            block.fields::<MossyCarpetBlock>().shapes[block.default_block_state().index() as usize]
                .clone()
        } else {
            Shapes::empty()
        }
    }
}
