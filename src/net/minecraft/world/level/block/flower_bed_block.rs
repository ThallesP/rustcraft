//! net/minecraft/world/level/block/FlowerBedBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `shapeHeight`.
//!
//! Of its behaviour, its shape (a quarter of a block for each flower, `SegmentableBlock`'s) is
//! ported, with its constructor's `shapeHeight`.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::segmentable_block::SegmentableBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct FlowerBedBlock {
    shape_height: i32,
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(FlowerBedBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.FlowerBedBlock",
        &vegetation_block::CLASS,
    )
};

impl FlowerBedBlock {
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::HORIZONTAL_FACING;
    pub const AMOUNT: &'static IntegerProperty = BlockStateProperties::FLOWER_AMOUNT;

    pub fn new(properties: Properties, shape_height: i32) -> BlockData {
        FlowerBedBlock::new_with(
            properties,
            shape_height,
            &CLASS,
            FlowerBedBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        shape_height: i32,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = VegetationBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(FlowerBedBlock::FACING, Direction::NORTH)
                .set_value(FlowerBedBlock::AMOUNT, 1),
        );
        let shapes = Block::get_shape_for_each_state(
            &block,
            SegmentableBlock::get_shape_calculator(
                f64::from(shape_height),
                FlowerBedBlock::FACING,
                FlowerBedBlock::AMOUNT,
            ),
        );
        block.set_fields(FlowerBedBlock {
            shape_height,
            shapes,
        });
        block
    }

    /// `getShapeHeight()`, `SegmentableBlock`'s, which it overrides with its constructor's.
    pub fn get_shape_height(block: &BlockData) -> f64 {
        f64::from(block.fields::<FlowerBedBlock>().shape_height)
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[FlowerBedBlock::FACING, FlowerBedBlock::AMOUNT]);
    }
}

impl FlowerBedBlock {
    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<FlowerBedBlock>().shapes[state.index() as usize].clone()
    }
}
