//! net/minecraft/world/entity/monster/Shulker.java
//!
//! Only the boxes a shulker's lid sweeps, which a shulker box's block entity takes too
//! (`getProgressAabb`, `getProgressDeltaAabb`). The shulker itself isn't ported yet.

use crate::java::lang::math::Math;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::vec3::Vec3;

pub struct Shulker;

impl Shulker {
    /// `getProgressAabb`: a box of the size with its bottom at the position, its lid open as far
    /// as `progress_to` towards the direction.
    pub fn get_progress_aabb(
        size: f32,
        direction: Direction,
        progress_to: f32,
        position: &Vec3,
    ) -> AABB {
        Shulker::get_progress_delta_aabb(size, direction, -1.0, progress_to, position)
    }

    /// `getProgressDeltaAabb`: what the lid sweeps from one progress to another.
    pub fn get_progress_delta_aabb(
        size: f32,
        direction: Direction,
        progress_from: f32,
        progress_to: f32,
        position: &Vec3,
    ) -> AABB {
        let half = f64::from(size) * 0.5;
        let bounds_at_bottom_center = AABB::new(
            f64::from(-size) * 0.5,
            0.0,
            f64::from(-size) * 0.5,
            half,
            f64::from(size),
            half,
        );
        let max_movement = f64::from(Math::max_f32(progress_from, progress_to));
        let min_movement = f64::from(Math::min_f32(progress_from, progress_to));
        let size = f64::from(size);
        let aabb = bounds_at_bottom_center
            .expand_towards_xyz(
                f64::from(direction.get_step_x()) * max_movement * size,
                f64::from(direction.get_step_y()) * max_movement * size,
                f64::from(direction.get_step_z()) * max_movement * size,
            )
            .contract(
                f64::from(-direction.get_step_x()) * (1.0 + min_movement) * size,
                f64::from(-direction.get_step_y()) * (1.0 + min_movement) * size,
                f64::from(-direction.get_step_z()) * (1.0 + min_movement) * size,
            );
        aabb.move_xyz(position.x, position.y, position.z)
    }
}
