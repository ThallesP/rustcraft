//! net/minecraft/world/level/border/WorldBorder.java
//!
//! Only a still border (`StaticBorderExtent`) and what collisions ask it: its bounds, the distance
//! to it, whether a box is close to it, and its collision shape. Not ported yet: a moving border
//! (`MovingBorderExtent`, `lerpSizeBetween`), its listeners, damage and warnings, the saved data
//! and the packets that set it, which come when the client handles `initialize_border`.

use crate::java::lang::math::Math;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

/// The square the world ends at.
#[derive(Clone, Debug)]
pub struct WorldBorder {
    center_x: f64,
    center_z: f64,
    absolute_max_size: i32,
    extent: StaticBorderExtent,
}

/// A border that doesn't move, its bounds worked out from the center and size.
#[derive(Clone, Debug)]
struct StaticBorderExtent {
    size: f64,
    min_x: f64,
    min_z: f64,
    max_x: f64,
    max_z: f64,
    shape: VoxelShape,
}

impl StaticBorderExtent {
    fn new(border: &WorldBorder, size: f64) -> StaticBorderExtent {
        let max_size = f64::from(border.absolute_max_size);
        let min_x = Mth::clamp_f64(border.center_x - size / 2.0, -max_size, max_size);
        let min_z = Mth::clamp_f64(border.center_z - size / 2.0, -max_size, max_size);
        let max_x = Mth::clamp_f64(border.center_x + size / 2.0, -max_size, max_size);
        let max_z = Mth::clamp_f64(border.center_z + size / 2.0, -max_size, max_size);
        let shape = Shapes::join(
            &Shapes::infinity(),
            &Shapes::r#box(
                min_x.floor(),
                f64::NEG_INFINITY,
                min_z.floor(),
                max_x.ceil(),
                f64::INFINITY,
                max_z.ceil(),
            ),
            BooleanOp::ONLY_FIRST,
        );
        StaticBorderExtent {
            size,
            min_x,
            min_z,
            max_x,
            max_z,
            shape,
        }
    }
}

impl Default for WorldBorder {
    fn default() -> WorldBorder {
        WorldBorder::new()
    }
}

impl WorldBorder {
    /// `new WorldBorder()`, with `Settings.DEFAULT`: centered on 0, 0, 5.999997E7F across.
    pub fn new() -> WorldBorder {
        let mut border = WorldBorder {
            center_x: 0.0,
            center_z: 0.0,
            absolute_max_size: 29999984,
            extent: StaticBorderExtent {
                size: 0.0,
                min_x: 0.0,
                min_z: 0.0,
                max_x: 0.0,
                max_z: 0.0,
                shape: Shapes::empty(),
            },
        };
        border.extent = StaticBorderExtent::new(&border, f64::from(5.999997E7_f32));
        border
    }

    pub fn get_min_x(&self) -> f64 {
        self.extent.min_x
    }

    pub fn get_min_z(&self) -> f64 {
        self.extent.min_z
    }

    pub fn get_max_x(&self) -> f64 {
        self.extent.max_x
    }

    pub fn get_max_z(&self) -> f64 {
        self.extent.max_z
    }

    pub fn get_size(&self) -> f64 {
        self.extent.size
    }

    /// `isWithinBounds(double, double, double)`.
    pub fn is_within_bounds_margin(&self, x: f64, z: f64, margin: f64) -> bool {
        x >= self.get_min_x() - margin
            && x < self.get_max_x() + margin
            && z >= self.get_min_z() - margin
            && z < self.get_max_z() + margin
    }

    pub fn get_collision_shape(&self) -> VoxelShape {
        self.extent.shape.clone()
    }

    /// `getDistanceToBorder(double, double)`.
    pub fn get_distance_to_border(&self, x: f64, z: f64) -> f64 {
        let from_north = z - self.get_min_z();
        let from_south = self.get_max_z() - z;
        let from_west = x - self.get_min_x();
        let from_east = self.get_max_x() - x;
        let min = Math::min_f64(from_west, from_east);
        let min = Math::min_f64(min, from_north);
        Math::min_f64(min, from_south)
    }

    /// Whether an entity at `x`, `z` with this box is inside the border and within twice the
    /// box's size of it.
    pub fn is_inside_close_to_border(&self, x: f64, z: f64, bounding_box: &AABB) -> bool {
        let bb_max = Math::max_f64(
            Mth::abs_max_f64(bounding_box.get_xsize(), bounding_box.get_zsize()),
            1.0,
        );
        self.get_distance_to_border(x, z) < bb_max * 2.0
            && self.is_within_bounds_margin(x, z, bb_max)
    }
}
