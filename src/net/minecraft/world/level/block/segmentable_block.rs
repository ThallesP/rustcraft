//! net/minecraft/world/level/block/SegmentableBlock.java
//!
//! Only `AMOUNT` and `getSegmentAmountProperty`'s default, which decide leaf litter's states. Not
//! ported yet: the rest, the segments' shapes and placement.

use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;

/// A block of up to four segments on the ground, like leaf litter.
pub struct SegmentableBlock;

impl SegmentableBlock {
    pub const AMOUNT: &'static IntegerProperty = BlockStateProperties::SEGMENT_AMOUNT;

    /// `getSegmentAmountProperty()`'s default.
    pub fn get_segment_amount_property() -> &'static IntegerProperty {
        SegmentableBlock::AMOUNT
    }
}
