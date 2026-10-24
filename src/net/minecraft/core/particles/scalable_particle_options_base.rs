//! net/minecraft/core/particles/ScalableParticleOptionsBase.java
//!
//! An abstract class whose one field, the scale, its subclasses keep themselves; this is its
//! constructor's clamp. Not ported yet: its `SCALE` codec.

use crate::net::minecraft::util::mth::Mth;

pub struct ScalableParticleOptionsBase;

impl ScalableParticleOptionsBase {
    pub const MIN_SCALE: f32 = 0.01;
    pub const MAX_SCALE: f32 = 4.0;

    /// The scale the constructor keeps: clamped to `MIN_SCALE`..`MAX_SCALE`.
    pub fn scale(scale: f32) -> f32 {
        Mth::clamp_f32(
            scale,
            ScalableParticleOptionsBase::MIN_SCALE,
            ScalableParticleOptionsBase::MAX_SCALE,
        )
    }
}
