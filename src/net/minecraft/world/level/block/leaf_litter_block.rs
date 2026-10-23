//! net/minecraft/world/level/block/LeafLitterBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shape (a quarter of a block for each segment, `SegmentableBlock`'s) is
//! ported.

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

pub struct LeafLitterBlock {
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(LeafLitterBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.LeafLitterBlock",
        &vegetation_block::CLASS,
    )
};

impl LeafLitterBlock {
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::HORIZONTAL_FACING;

    pub fn new(properties: Properties) -> BlockData {
        LeafLitterBlock::new_with(
            properties,
            &CLASS,
            LeafLitterBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = VegetationBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(LeafLitterBlock::FACING, Direction::NORTH)
                .set_value(LeafLitterBlock::get_segment_amount_property(), 1),
        );
        let shapes = LeafLitterBlock::make_shapes(&block);
        block.set_fields(LeafLitterBlock { shapes });
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            LeafLitterBlock::FACING,
            LeafLitterBlock::get_segment_amount_property(),
        ]);
    }

    /// `SegmentableBlock`'s, which it doesn't override.
    pub fn get_segment_amount_property() -> &'static IntegerProperty {
        SegmentableBlock::get_segment_amount_property()
    }
}

impl LeafLitterBlock {
    /// `makeShapes`: `SegmentableBlock`'s, a sixteenth tall.
    fn make_shapes(block: &BlockData) -> Vec<VoxelShape> {
        Block::get_shape_for_each_state(
            block,
            SegmentableBlock::get_shape_calculator(
                SegmentableBlock::DEFAULT_SHAPE_HEIGHT,
                LeafLitterBlock::FACING,
                LeafLitterBlock::get_segment_amount_property(),
            ),
        )
    }

    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<LeafLitterBlock>().shapes[state.index() as usize].clone()
    }
}
