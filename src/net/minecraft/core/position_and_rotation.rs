//! net/minecraft/core/PositionAndRotation.java
//!
//! Java's interface has two kinds, `Immutable` and `Mutable`, which hold the same fields; here
//! they're one struct, copied where Java calls `immutable()`. Not ported yet: its stream codec,
//! which nothing the client reads uses.

use crate::net::minecraft::world::phys::vec3::Vec3;

/// A position and a rotation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositionAndRotation {
    position: Vec3,
    y_rot: f32,
    x_rot: f32,
}

/// `Mutable`'s start: the origin, no rotation.
impl Default for PositionAndRotation {
    fn default() -> PositionAndRotation {
        PositionAndRotation::of(Vec3::ZERO, 0.0, 0.0)
    }
}

impl PositionAndRotation {
    pub fn of(position: Vec3, y_rot: f32, x_rot: f32) -> PositionAndRotation {
        PositionAndRotation {
            position,
            y_rot,
            x_rot,
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn y_rot(&self) -> f32 {
        self.y_rot
    }

    pub fn x_rot(&self) -> f32 {
        self.x_rot
    }

    /// `is`: the same rotation (by `==`, so -0.0 is 0.0 and NaN nothing) and position (by
    /// `Vec3.equals`).
    pub fn is(&self, position: &Vec3, y_rot: f32, x_rot: f32) -> bool {
        self.y_rot == y_rot && self.x_rot == x_rot && self.position == *position
    }

    /// `Mutable.set`.
    pub fn set(&mut self, position: Vec3, y_rot: f32, x_rot: f32) {
        self.position = position;
        self.y_rot = y_rot;
        self.x_rot = x_rot;
    }

    /// `Mutable.addDelta`.
    pub fn add_delta(&mut self, delta: &Vec3) {
        self.position = self.position.add(delta);
    }

    /// `Mutable.addRotation`.
    pub fn add_rotation(&mut self, y_rot: f32, x_rot: f32) {
        self.y_rot += y_rot;
        self.x_rot += x_rot;
    }
}
