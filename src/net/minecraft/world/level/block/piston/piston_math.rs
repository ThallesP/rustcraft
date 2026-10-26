//! net/minecraft/world/level/block/piston/PistonMath.java

use crate::java::lang::math::Math;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::phys::aabb::AABB;

pub struct PistonMath;

impl PistonMath {
    /// `getMovementArea`: the slab a box's face towards the direction sweeps moving `amount`.
    pub fn get_movement_area(aabb: &AABB, direction: Direction, amount: f64) -> AABB {
        let delta = amount * f64::from(direction.get_axis_direction().get_step());
        let min = Math::min_f64(delta, 0.0);
        let max = Math::max_f64(delta, 0.0);
        match direction {
            Direction::WEST => AABB::new(
                aabb.min_x + min,
                aabb.min_y,
                aabb.min_z,
                aabb.min_x + max,
                aabb.max_y,
                aabb.max_z,
            ),
            Direction::EAST => AABB::new(
                aabb.max_x + min,
                aabb.min_y,
                aabb.min_z,
                aabb.max_x + max,
                aabb.max_y,
                aabb.max_z,
            ),
            Direction::DOWN => AABB::new(
                aabb.min_x,
                aabb.min_y + min,
                aabb.min_z,
                aabb.max_x,
                aabb.min_y + max,
                aabb.max_z,
            ),
            Direction::UP => AABB::new(
                aabb.min_x,
                aabb.max_y + min,
                aabb.min_z,
                aabb.max_x,
                aabb.max_y + max,
                aabb.max_z,
            ),
            Direction::NORTH => AABB::new(
                aabb.min_x,
                aabb.min_y,
                aabb.min_z + min,
                aabb.max_x,
                aabb.max_y,
                aabb.min_z + max,
            ),
            Direction::SOUTH => AABB::new(
                aabb.min_x,
                aabb.min_y,
                aabb.max_z + min,
                aabb.max_x,
                aabb.max_y,
                aabb.max_z + max,
            ),
        }
    }
}
