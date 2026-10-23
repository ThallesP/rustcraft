//! net/minecraft/world/level/material/LavaFluid.java
//!
//! Only `isSame` so far: still and flowing lava are the same fluid. Not ported yet: its ticking,
//! fire, sounds and particles.

use crate::net::minecraft::world::level::material::fluids::Fluids;

pub fn is_same(other: Fluids) -> bool {
    other == Fluids::LAVA || other == Fluids::FLOWING_LAVA
}
