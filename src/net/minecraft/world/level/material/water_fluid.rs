//! net/minecraft/world/level/material/WaterFluid.java
//!
//! Only `isSame` so far: still and flowing water are the same fluid. Not ported yet: its ticking,
//! sounds and particles.

use crate::net::minecraft::world::level::material::fluids::Fluids;

pub fn is_same(other: Fluids) -> bool {
    other == Fluids::WATER || other == Fluids::FLOWING_WATER
}
