//! net/minecraft/world/level/block/state/properties/RotationSegment.java
//!
//! Only `getMaxSegmentIndex`, the most a standing sign's or banner's `rotation` goes to, and
//! `convertToSegment(Direction)`, `SEGMENTED_ANGLE16.fromDirection` written out. Not ported yet:
//! `convertToSegment(float)` and `convertToDegrees`, which wait for `SegmentedAnglePrecision`.

use crate::net::minecraft::core::direction::Direction;

pub struct RotationSegment;

impl RotationSegment {
    // `SegmentedAnglePrecision(4)`'s mask: sixteen segments, from 0.
    const MAX_SEGMENT_INDEX: i32 = (1 << 4) - 1;

    pub const fn get_max_segment_index() -> i32 {
        RotationSegment::MAX_SEGMENT_INDEX
    }

    /// `convertToSegment(Direction)`: 0 for up or down, else the direction's quarter turn from
    /// south in sixteenths of a turn.
    pub fn convert_to_segment(direction: Direction) -> i32 {
        if direction.get_axis().is_vertical() {
            return 0;
        }
        direction.get_2d_data_value() << (4 - 2)
    }
}
