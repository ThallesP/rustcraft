//! net/minecraft/world/level/material/EmptyFluid.java
//!
//! The empty fluid's answers: no height and no flow. `Fluids::EMPTY` dispatches here.

use crate::net::minecraft::world::phys::vec3::Vec3;

pub fn get_flow() -> Vec3 {
    Vec3::ZERO
}

pub fn get_height() -> f32 {
    0.0
}

pub fn get_own_height() -> f32 {
    0.0
}
