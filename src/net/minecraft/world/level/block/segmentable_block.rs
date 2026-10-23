//! net/minecraft/world/level/block/SegmentableBlock.java
//!
//! `AMOUNT`, `getSegmentAmountProperty`'s default and `getShapeCalculator`, the segments' shapes,
//! which take the implementing block's `getShapeHeight()` (the interface's 1.0 unless it
//! overrides it). Not ported yet: placement.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

/// A block of up to four segments on the ground, like leaf litter.
pub struct SegmentableBlock;

impl SegmentableBlock {
    pub const AMOUNT: &'static IntegerProperty = BlockStateProperties::SEGMENT_AMOUNT;

    /// `getShapeHeight()`'s default.
    pub const DEFAULT_SHAPE_HEIGHT: f64 = 1.0;

    /// `getSegmentAmountProperty()`'s default.
    pub fn get_segment_amount_property() -> &'static IntegerProperty {
        SegmentableBlock::AMOUNT
    }

    /// `getShapeCalculator(EnumProperty, IntegerProperty)`: a quarter of the block for each
    /// segment, counter-clockwise from the one it faces, as one box around them all.
    pub fn get_shape_calculator(
        shape_height: f64,
        facing: &'static EnumProperty<Direction>,
        amount: &'static IntegerProperty,
    ) -> impl Fn(BlockState) -> VoxelShape {
        let shapes =
            Shapes::rotate_horizontal(&Block::r#box(0.0, 0.0, 0.0, 8.0, shape_height, 8.0));
        move |state| {
            let mut shape = Shapes::empty();
            let mut direction = state.get_value(facing);
            let count = state.get_value(amount);
            for _ in 0..count {
                shape = Shapes::or(&shape, Shapes::get(&shapes, direction));
                direction = direction.get_counter_clock_wise();
            }
            shape.single_encompassing()
        }
    }
}
