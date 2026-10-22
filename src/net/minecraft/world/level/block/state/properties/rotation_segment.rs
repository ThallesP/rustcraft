//! net/minecraft/world/level/block/state/properties/RotationSegment.java
//!
//! Only `getMaxSegmentIndex`, the most a standing sign's or banner's `rotation` goes to. Not ported
//! yet: `convertToSegment` and `convertToDegrees`, which wait for `SegmentedAnglePrecision`.

pub struct RotationSegment;

impl RotationSegment {
    // `SegmentedAnglePrecision(4)`'s mask: sixteen segments, from 0.
    const MAX_SEGMENT_INDEX: i32 = (1 << 4) - 1;

    pub const fn get_max_segment_index() -> i32 {
        RotationSegment::MAX_SEGMENT_INDEX
    }
}
