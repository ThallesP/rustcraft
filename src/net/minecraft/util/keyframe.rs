//! net/minecraft/util/Keyframe.java
//!
//! `codec` is `KeyframeTrack::from_json`'s, which reads `ticks` and `value`.

/// A value at a tick of a track.
#[derive(Clone, Copy, Debug)]
pub struct Keyframe<T> {
    pub ticks: i32,
    pub value: T,
}

impl<T> Keyframe<T> {
    pub fn new(ticks: i32, value: T) -> Keyframe<T> {
        Keyframe { ticks, value }
    }
}
