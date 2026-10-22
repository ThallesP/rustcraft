//! net/minecraft/world/phys/AABB.java
//!
//! Not ported yet: `of(BoundingBox)`, which comes with `BoundingBox`, and `toString`, which
//! needs Java's `Double.toString` (only logs and error messages print it).

use crate::java::lang::double::Double;
use crate::java::lang::math::Math;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::phys::block_hit_result::BlockHitResult;
use crate::net::minecraft::world::phys::vec3::Vec3;
use crate::org::joml::vector3f::Vector3f;

/// How far `clip` reaches past a box's edges, and how little a ray has to move along an axis to
/// count as moving along it.
const EPSILON: f64 = 1.0E-7;

/// An axis-aligned box in blocks: entities' bounding boxes and the parts of blocks' shapes.
#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

/// Java's `equals` compares the sides with `Double.compare`, so -0.0 differs from 0.0 and NaN
/// equals NaN.
impl PartialEq for AABB {
    fn eq(&self, other: &AABB) -> bool {
        Double::double_to_long_bits(other.min_x) == Double::double_to_long_bits(self.min_x)
            && Double::double_to_long_bits(other.min_y) == Double::double_to_long_bits(self.min_y)
            && Double::double_to_long_bits(other.min_z) == Double::double_to_long_bits(self.min_z)
            && Double::double_to_long_bits(other.max_x) == Double::double_to_long_bits(self.max_x)
            && Double::double_to_long_bits(other.max_y) == Double::double_to_long_bits(self.max_y)
            && Double::double_to_long_bits(other.max_z) == Double::double_to_long_bits(self.max_z)
    }
}

impl Eq for AABB {}

impl AABB {
    /// Takes the corners either way round: each min is `Math.min` of the pair (a NaN wins, and
    /// -0.0 is below 0.0) and each max is `Math.max`.
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> AABB {
        AABB {
            min_x: Math::min_f64(min_x, max_x),
            min_y: Math::min_f64(min_y, max_y),
            min_z: Math::min_f64(min_z, max_z),
            max_x: Math::max_f64(min_x, max_x),
            max_y: Math::max_f64(min_y, max_y),
            max_z: Math::max_f64(min_z, max_z),
        }
    }

    /// The block's cube. Its far sides are int sums, which wrap at `Integer.MAX_VALUE`.
    pub fn from_block_pos(pos: &BlockPos) -> AABB {
        AABB::new(
            f64::from(pos.get_x()),
            f64::from(pos.get_y()),
            f64::from(pos.get_z()),
            f64::from(pos.get_x().wrapping_add(1)),
            f64::from(pos.get_y().wrapping_add(1)),
            f64::from(pos.get_z().wrapping_add(1)),
        )
    }

    pub fn from_vec3(begin: &Vec3, end: &Vec3) -> AABB {
        AABB::new(begin.x, begin.y, begin.z, end.x, end.y, end.z)
    }

    pub fn unit_cube_from_lower_corner(pos: &Vec3) -> AABB {
        AABB::new(pos.x, pos.y, pos.z, pos.x + 1.0, pos.y + 1.0, pos.z + 1.0)
    }

    /// Both blocks and every block between them. The far sides are int sums, which wrap.
    pub fn encapsulating_full_blocks(pos0: &BlockPos, pos1: &BlockPos) -> AABB {
        AABB::new(
            f64::from(pos0.get_x().min(pos1.get_x())),
            f64::from(pos0.get_y().min(pos1.get_y())),
            f64::from(pos0.get_z().min(pos1.get_z())),
            f64::from(pos0.get_x().max(pos1.get_x()).wrapping_add(1)),
            f64::from(pos0.get_y().max(pos1.get_y()).wrapping_add(1)),
            f64::from(pos0.get_z().max(pos1.get_z()).wrapping_add(1)),
        )
    }

    pub fn set_min_x(&self, min_x: f64) -> AABB {
        AABB::new(
            min_x, self.min_y, self.min_z, self.max_x, self.max_y, self.max_z,
        )
    }

    pub fn set_min_y(&self, min_y: f64) -> AABB {
        AABB::new(
            self.min_x, min_y, self.min_z, self.max_x, self.max_y, self.max_z,
        )
    }

    pub fn set_min_z(&self, min_z: f64) -> AABB {
        AABB::new(
            self.min_x, self.min_y, min_z, self.max_x, self.max_y, self.max_z,
        )
    }

    pub fn set_max_x(&self, max_x: f64) -> AABB {
        AABB::new(
            self.min_x, self.min_y, self.min_z, max_x, self.max_y, self.max_z,
        )
    }

    pub fn set_max_y(&self, max_y: f64) -> AABB {
        AABB::new(
            self.min_x, self.min_y, self.min_z, self.max_x, max_y, self.max_z,
        )
    }

    pub fn set_max_z(&self, max_z: f64) -> AABB {
        AABB::new(
            self.min_x, self.min_y, self.min_z, self.max_x, self.max_y, max_z,
        )
    }

    pub fn min(&self, axis: Axis) -> f64 {
        axis.choose_f64(self.min_x, self.min_y, self.min_z)
    }

    pub fn max(&self, axis: Axis) -> f64 {
        axis.choose_f64(self.max_x, self.max_y, self.max_z)
    }

    /// Each side's `Double.hashCode`, folded in by 31s.
    pub fn hash_code(&self) -> i32 {
        let mut result = Double::hash_code(self.min_x);
        for side in [self.min_y, self.min_z, self.max_x, self.max_y, self.max_z] {
            result = result
                .wrapping_mul(31)
                .wrapping_add(Double::hash_code(side));
        }
        result
    }

    /// Pulls in the side facing the movement on each axis: the max side for a positive one, the
    /// min side for a negative one.
    pub fn contract(&self, xa: f64, ya: f64, za: f64) -> AABB {
        let mut min_x = self.min_x;
        let mut min_y = self.min_y;
        let mut min_z = self.min_z;
        let mut max_x = self.max_x;
        let mut max_y = self.max_y;
        let mut max_z = self.max_z;
        if xa < 0.0 {
            min_x -= xa;
        } else if xa > 0.0 {
            max_x -= xa;
        }

        if ya < 0.0 {
            min_y -= ya;
        } else if ya > 0.0 {
            max_y -= ya;
        }

        if za < 0.0 {
            min_z -= za;
        } else if za > 0.0 {
            max_z -= za;
        }

        AABB::new(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    pub fn expand_towards(&self, delta: &Vec3) -> AABB {
        self.expand_towards_xyz(delta.x, delta.y, delta.z)
    }

    /// Pushes out the side facing the movement on each axis. Zero, -0.0 and NaN move nothing.
    pub fn expand_towards_xyz(&self, xa: f64, ya: f64, za: f64) -> AABB {
        let mut min_x = self.min_x;
        let mut min_y = self.min_y;
        let mut min_z = self.min_z;
        let mut max_x = self.max_x;
        let mut max_y = self.max_y;
        let mut max_z = self.max_z;
        if xa < 0.0 {
            min_x += xa;
        } else if xa > 0.0 {
            max_x += xa;
        }

        if ya < 0.0 {
            min_y += ya;
        } else if ya > 0.0 {
            max_y += ya;
        }

        if za < 0.0 {
            min_z += za;
        } else if za > 0.0 {
            max_z += za;
        }

        AABB::new(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    pub fn inflate_xyz(&self, x_add: f64, y_add: f64, z_add: f64) -> AABB {
        let min_x = self.min_x - x_add;
        let min_y = self.min_y - y_add;
        let min_z = self.min_z - z_add;
        let max_x = self.max_x + x_add;
        let max_y = self.max_y + y_add;
        let max_z = self.max_z + z_add;
        AABB::new(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    pub fn inflate(&self, amount_to_add_in_all_directions: f64) -> AABB {
        self.inflate_xyz(
            amount_to_add_in_all_directions,
            amount_to_add_in_all_directions,
            amount_to_add_in_all_directions,
        )
    }

    /// The overlap. Boxes that don't overlap give a box with its sides swapped back into order.
    pub fn intersect(&self, other: &AABB) -> AABB {
        let min_x = Math::max_f64(self.min_x, other.min_x);
        let min_y = Math::max_f64(self.min_y, other.min_y);
        let min_z = Math::max_f64(self.min_z, other.min_z);
        let max_x = Math::min_f64(self.max_x, other.max_x);
        let max_y = Math::min_f64(self.max_y, other.max_y);
        let max_z = Math::min_f64(self.max_z, other.max_z);
        AABB::new(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    /// The smallest box around both.
    pub fn minmax(&self, other: &AABB) -> AABB {
        let min_x = Math::min_f64(self.min_x, other.min_x);
        let min_y = Math::min_f64(self.min_y, other.min_y);
        let min_z = Math::min_f64(self.min_z, other.min_z);
        let max_x = Math::max_f64(self.max_x, other.max_x);
        let max_y = Math::max_f64(self.max_y, other.max_y);
        let max_z = Math::max_f64(self.max_z, other.max_z);
        AABB::new(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    pub fn move_xyz(&self, xa: f64, ya: f64, za: f64) -> AABB {
        AABB::new(
            self.min_x + xa,
            self.min_y + ya,
            self.min_z + za,
            self.max_x + xa,
            self.max_y + ya,
            self.max_z + za,
        )
    }

    pub fn move_block_pos(&self, pos: &BlockPos) -> AABB {
        AABB::new(
            self.min_x + f64::from(pos.get_x()),
            self.min_y + f64::from(pos.get_y()),
            self.min_z + f64::from(pos.get_z()),
            self.max_x + f64::from(pos.get_x()),
            self.max_y + f64::from(pos.get_y()),
            self.max_z + f64::from(pos.get_z()),
        )
    }

    pub fn r#move(&self, pos: &Vec3) -> AABB {
        self.move_xyz(pos.x, pos.y, pos.z)
    }

    pub fn move_vector3f(&self, pos: &Vector3f) -> AABB {
        self.move_xyz(f64::from(pos.x), f64::from(pos.y), f64::from(pos.z))
    }

    pub fn intersects(&self, aabb: &AABB) -> bool {
        self.intersects_xyz(
            aabb.min_x, aabb.min_y, aabb.min_z, aabb.max_x, aabb.max_y, aabb.max_z,
        )
    }

    /// Strictly: boxes that only touch don't intersect.
    pub fn intersects_xyz(
        &self,
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> bool {
        self.min_x < max_x
            && self.max_x > min_x
            && self.min_y < max_y
            && self.max_y > min_y
            && self.min_z < max_z
            && self.max_z > min_z
    }

    pub fn intersects_vec3(&self, min: &Vec3, max: &Vec3) -> bool {
        self.intersects_xyz(
            Math::min_f64(min.x, max.x),
            Math::min_f64(min.y, max.y),
            Math::min_f64(min.z, max.z),
            Math::max_f64(min.x, max.x),
            Math::max_f64(min.y, max.y),
            Math::max_f64(min.z, max.z),
        )
    }

    /// The block's cube, whose far sides are int sums, which wrap.
    pub fn intersects_block_pos(&self, pos: &BlockPos) -> bool {
        self.intersects_xyz(
            f64::from(pos.get_x()),
            f64::from(pos.get_y()),
            f64::from(pos.get_z()),
            f64::from(pos.get_x().wrapping_add(1)),
            f64::from(pos.get_y().wrapping_add(1)),
            f64::from(pos.get_z().wrapping_add(1)),
        )
    }

    pub fn contains(&self, vec: &Vec3) -> bool {
        self.contains_xyz(vec.x, vec.y, vec.z)
    }

    /// The min sides are inside and the max sides outside.
    pub fn contains_xyz(&self, x: f64, y: f64, z: f64) -> bool {
        x >= self.min_x
            && x < self.max_x
            && y >= self.min_y
            && y < self.max_y
            && z >= self.min_z
            && z < self.max_z
    }

    /// The average of the three sizes.
    pub fn get_size(&self) -> f64 {
        let xs = self.get_xsize();
        let ys = self.get_ysize();
        let zs = self.get_zsize();
        (xs + ys + zs) / 3.0
    }

    pub fn get_xsize(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn get_ysize(&self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn get_zsize(&self) -> f64 {
        self.max_z - self.min_z
    }

    pub fn deflate_xyz(&self, x_substract: f64, y_subtract: f64, z_subtract: f64) -> AABB {
        self.inflate_xyz(-x_substract, -y_subtract, -z_subtract)
    }

    pub fn deflate(&self, amount: f64) -> AABB {
        self.inflate(-amount)
    }

    pub fn clip(&self, from: &Vec3, to: &Vec3) -> Option<Vec3> {
        AABB::clip_xyz(
            self.min_x, self.min_y, self.min_z, self.max_x, self.max_y, self.max_z, from, to,
        )
    }

    /// Where the ray from `from` to `to` enters the box, if it does before `to`.
    #[allow(clippy::too_many_arguments)]
    pub fn clip_xyz(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
        from: &Vec3,
        to: &Vec3,
    ) -> Option<Vec3> {
        let mut scale_reference = 1.0;
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let dz = to.z - from.z;
        AABB::get_direction_xyz(
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
            from,
            &mut scale_reference,
            None,
            dx,
            dy,
            dz,
        )?;

        let scale = scale_reference;
        Some(from.add_xyz(scale * dx, scale * dy, scale * dz))
    }

    /// Java's `Iterable<AABB>` overload: where the ray first enters one of a block's shapes,
    /// given relative to the block. On a tie, the first box wins.
    pub fn clip_aabbs(
        aab_bs: &[AABB],
        from: &Vec3,
        to: &Vec3,
        pos: &BlockPos,
    ) -> Option<BlockHitResult> {
        let mut scale_reference = 1.0;
        let mut direction = None;
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let dz = to.z - from.z;

        for aabb in aab_bs {
            direction = AABB::get_direction(
                &aabb.move_block_pos(pos),
                from,
                &mut scale_reference,
                direction,
                dx,
                dy,
                dz,
            );
        }

        let direction = direction?;
        let scale = scale_reference;
        Some(BlockHitResult::new(
            &from.add_xyz(scale * dx, scale * dy, scale * dz),
            direction,
            pos,
            false,
        ))
    }

    fn get_direction(
        aabb: &AABB,
        from: &Vec3,
        scale_reference: &mut f64,
        direction: Option<Direction>,
        dx: f64,
        dy: f64,
        dz: f64,
    ) -> Option<Direction> {
        AABB::get_direction_xyz(
            aabb.min_x,
            aabb.min_y,
            aabb.min_z,
            aabb.max_x,
            aabb.max_y,
            aabb.max_z,
            from,
            scale_reference,
            direction,
            dx,
            dy,
            dz,
        )
    }

    /// Tries the side the ray enters through on each axis it moves along, x first. A side
    /// replaces `direction` when the ray reaches it before any side so far.
    #[allow(clippy::too_many_arguments)]
    fn get_direction_xyz(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
        from: &Vec3,
        scale_reference: &mut f64,
        mut direction: Option<Direction>,
        dx: f64,
        dy: f64,
        dz: f64,
    ) -> Option<Direction> {
        if dx > EPSILON {
            direction = AABB::clip_point(
                scale_reference,
                direction,
                dx,
                dy,
                dz,
                min_x,
                min_y,
                max_y,
                min_z,
                max_z,
                Direction::WEST,
                from.x,
                from.y,
                from.z,
            );
        } else if dx < -EPSILON {
            direction = AABB::clip_point(
                scale_reference,
                direction,
                dx,
                dy,
                dz,
                max_x,
                min_y,
                max_y,
                min_z,
                max_z,
                Direction::EAST,
                from.x,
                from.y,
                from.z,
            );
        }

        if dy > EPSILON {
            direction = AABB::clip_point(
                scale_reference,
                direction,
                dy,
                dz,
                dx,
                min_y,
                min_z,
                max_z,
                min_x,
                max_x,
                Direction::DOWN,
                from.y,
                from.z,
                from.x,
            );
        } else if dy < -EPSILON {
            direction = AABB::clip_point(
                scale_reference,
                direction,
                dy,
                dz,
                dx,
                max_y,
                min_z,
                max_z,
                min_x,
                max_x,
                Direction::UP,
                from.y,
                from.z,
                from.x,
            );
        }

        if dz > EPSILON {
            direction = AABB::clip_point(
                scale_reference,
                direction,
                dz,
                dx,
                dy,
                min_z,
                min_x,
                max_x,
                min_y,
                max_y,
                Direction::NORTH,
                from.z,
                from.x,
                from.y,
            );
        } else if dz < -EPSILON {
            direction = AABB::clip_point(
                scale_reference,
                direction,
                dz,
                dx,
                dy,
                max_z,
                min_x,
                max_x,
                min_y,
                max_y,
                Direction::SOUTH,
                from.z,
                from.x,
                from.y,
            );
        }

        direction
    }

    /// Where the ray reaches the plane `point` on axis a, as a fraction of the ray. It hits when
    /// that's strictly between 0 and the nearest hit so far, and within the side's bounds on
    /// the other two axes, give or take `EPSILON`.
    #[allow(clippy::too_many_arguments)]
    fn clip_point(
        scale_reference: &mut f64,
        direction: Option<Direction>,
        da: f64,
        db: f64,
        dc: f64,
        point: f64,
        min_b: f64,
        max_b: f64,
        min_c: f64,
        max_c: f64,
        new_direction: Direction,
        from_a: f64,
        from_b: f64,
        from_c: f64,
    ) -> Option<Direction> {
        let s = (point - from_a) / da;
        let pb = from_b + s * db;
        let pc = from_c + s * dc;
        if 0.0 < s
            && s < *scale_reference
            && min_b - EPSILON < pb
            && pb < max_b + EPSILON
            && min_c - EPSILON < pc
            && pc < max_c + EPSILON
        {
            *scale_reference = s;
            Some(new_direction)
        } else {
            direction
        }
    }

    /// Whether this box, moved along `vector`, runs into any of the boxes: they grow by half
    /// this box (less `EPSILON`), and the path is a ray from this box's center.
    pub fn collided_along_vector(&self, vector: &Vec3, aabbs: &[AABB]) -> bool {
        let from = self.get_center();
        let to = from.add(vector);

        for shape_part in aabbs {
            let inflated = shape_part.inflate_xyz(
                self.get_xsize() * 0.5 - EPSILON,
                self.get_ysize() * 0.5 - EPSILON,
                self.get_zsize() * 0.5 - EPSILON,
            );
            if inflated.contains(&to) || inflated.contains(&from) {
                return true;
            }

            if inflated.clip(&from, &to).is_some() {
                return true;
            }
        }

        false
    }

    pub fn distance_to_sqr(&self, point: &Vec3) -> f64 {
        let dx = Math::max_f64(
            Math::max_f64(self.min_x - point.x, point.x - self.max_x),
            0.0,
        );
        let dy = Math::max_f64(
            Math::max_f64(self.min_y - point.y, point.y - self.max_y),
            0.0,
        );
        let dz = Math::max_f64(
            Math::max_f64(self.min_z - point.z, point.z - self.max_z),
            0.0,
        );
        Mth::length_squared3_f64(dx, dy, dz)
    }

    pub fn distance_to_sqr_aabb(&self, bounding_box: &AABB) -> f64 {
        let dx = Math::max_f64(
            Math::max_f64(
                self.min_x - bounding_box.max_x,
                bounding_box.min_x - self.max_x,
            ),
            0.0,
        );
        let dy = Math::max_f64(
            Math::max_f64(
                self.min_y - bounding_box.max_y,
                bounding_box.min_y - self.max_y,
            ),
            0.0,
        );
        let dz = Math::max_f64(
            Math::max_f64(
                self.min_z - bounding_box.max_z,
                bounding_box.min_z - self.max_z,
            ),
            0.0,
        );
        Mth::length_squared3_f64(dx, dy, dz)
    }

    pub fn has_nan(&self) -> bool {
        self.min_x.is_nan()
            || self.min_y.is_nan()
            || self.min_z.is_nan()
            || self.max_x.is_nan()
            || self.max_y.is_nan()
            || self.max_z.is_nan()
    }

    pub fn get_center(&self) -> Vec3 {
        Vec3::new(
            Mth::lerp_f64(0.5, self.min_x, self.max_x),
            Mth::lerp_f64(0.5, self.min_y, self.max_y),
            Mth::lerp_f64(0.5, self.min_z, self.max_z),
        )
    }

    pub fn get_bottom_center(&self) -> Vec3 {
        Vec3::new(
            Mth::lerp_f64(0.5, self.min_x, self.max_x),
            self.min_y,
            Mth::lerp_f64(0.5, self.min_z, self.max_z),
        )
    }

    pub fn get_min_position(&self) -> Vec3 {
        Vec3::new(self.min_x, self.min_y, self.min_z)
    }

    pub fn get_max_position(&self) -> Vec3 {
        Vec3::new(self.max_x, self.max_y, self.max_z)
    }

    pub fn of_size(center: &Vec3, size_x: f64, size_y: f64, size_z: f64) -> AABB {
        AABB::new(
            center.x - size_x / 2.0,
            center.y - size_y / 2.0,
            center.z - size_z / 2.0,
            center.x + size_x / 2.0,
            center.y + size_y / 2.0,
            center.z + size_z / 2.0,
        )
    }

    /// Every side moved in by an ulp.
    pub fn next_deflated(&self) -> AABB {
        AABB::new(
            Math::next_up_f64(self.min_x),
            Math::next_up_f64(self.min_y),
            Math::next_up_f64(self.min_z),
            Math::next_down_f64(self.max_x),
            Math::next_down_f64(self.max_y),
            Math::next_down_f64(self.max_z),
        )
    }
}

/// `AABB.Builder`: the smallest box around the points it's given, in floats.
#[derive(Clone, Copy, Debug)]
pub struct Builder {
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,
    defined: bool,
}

impl Default for Builder {
    fn default() -> Builder {
        Builder::new()
    }
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            min_x: f32::INFINITY,
            min_y: f32::INFINITY,
            min_z: f32::INFINITY,
            max_x: f32::NEG_INFINITY,
            max_y: f32::NEG_INFINITY,
            max_z: f32::NEG_INFINITY,
            defined: false,
        }
    }

    /// A NaN coordinate sticks: `Math.min` and `Math.max` keep it from then on.
    pub fn include(&mut self, v: &Vector3f) {
        self.min_x = Math::min_f32(self.min_x, v.x);
        self.min_y = Math::min_f32(self.min_y, v.y);
        self.min_z = Math::min_f32(self.min_z, v.z);
        self.max_x = Math::max_f32(self.max_x, v.x);
        self.max_y = Math::max_f32(self.max_y, v.y);
        self.max_z = Math::max_f32(self.max_z, v.z);
        self.defined = true;
    }

    pub fn is_defined(&self) -> bool {
        self.defined
    }

    /// Panics before a point is included, where Java throws `IllegalStateException`.
    pub fn build(&self) -> AABB {
        if !self.defined {
            panic!(
                "IllegalStateException: Cannot build an undefined AABB. Include at least one point."
            );
        }

        AABB::new(
            f64::from(self.min_x),
            f64::from(self.min_y),
            f64::from(self.min_z),
            f64::from(self.max_x),
            f64::from(self.max_y),
            f64::from(self.max_z),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{AABB, Builder};
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::world::phys::vec3::Vec3;
    use crate::org::joml::vector3f::Vector3f;

    encode_fields!(AABB {
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64
    });

    encode_fields!(Builder {
        min_x: f32,
        min_y: f32,
        min_z: f32,
        max_x: f32,
        max_y: f32,
        max_z: f32,
        defined: bool
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.phys.AABB");
        golden.pending(
            "of(Lnet/minecraft/world/level/levelgen/structure/BoundingBox;)Lnet/minecraft/world/phys/AABB;",
            "waits for BoundingBox",
        );
        golden.pending(
            "toString()Ljava/lang/String;",
            "needs Java's Double.toString, and only logs and error messages print it",
        );
        golden.check("<init>(DDDDDD)V", AABB::new);
        golden.check("<init>(Lnet/minecraft/core/BlockPos;)V", |pos: BlockPos| {
            AABB::from_block_pos(&pos)
        });
        golden.check(
            "<init>(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;)V",
            |begin: Vec3, end: Vec3| AABB::from_vec3(&begin, &end),
        );
        golden.check(
            "unitCubeFromLowerCorner(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/AABB;",
            |pos: Vec3| AABB::unit_cube_from_lower_corner(&pos),
        );
        golden.check(
            "encapsulatingFullBlocks(Lnet/minecraft/core/BlockPos;Lnet/minecraft/core/BlockPos;)Lnet/minecraft/world/phys/AABB;",
            |pos0: BlockPos, pos1: BlockPos| AABB::encapsulating_full_blocks(&pos0, &pos1),
        );
        golden.check_method(
            "setMinX(D)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, min_x| this.set_min_x(min_x),
        );
        golden.check_method(
            "setMinY(D)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, min_y| this.set_min_y(min_y),
        );
        golden.check_method(
            "setMinZ(D)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, min_z| this.set_min_z(min_z),
        );
        golden.check_method(
            "setMaxX(D)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, max_x| this.set_max_x(max_x),
        );
        golden.check_method(
            "setMaxY(D)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, max_y| this.set_max_y(max_y),
        );
        golden.check_method(
            "setMaxZ(D)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, max_z| this.set_max_z(max_z),
        );
        golden.check_method(
            "min(Lnet/minecraft/core/Direction$Axis;)D",
            |this: &mut AABB, axis| this.min(axis),
        );
        golden.check_method(
            "max(Lnet/minecraft/core/Direction$Axis;)D",
            |this: &mut AABB, axis| this.max(axis),
        );
        golden.check_method("equals(Ljava/lang/Object;)Z", |this: &mut AABB, o: AABB| {
            *this == o
        });
        golden.check_method("hashCode()I", |this: &mut AABB| this.hash_code());
        golden.check_method(
            "contract(DDD)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, xa, ya, za| this.contract(xa, ya, za),
        );
        golden.check_method(
            "expandTowards(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, delta: Vec3| this.expand_towards(&delta),
        );
        golden.check_method(
            "expandTowards(DDD)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, xa, ya, za| this.expand_towards_xyz(xa, ya, za),
        );
        golden.check_method(
            "inflate(DDD)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, x_add, y_add, z_add| this.inflate_xyz(x_add, y_add, z_add),
        );
        golden.check_method(
            "inflate(D)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, amount| this.inflate(amount),
        );
        golden.check_method(
            "intersect(Lnet/minecraft/world/phys/AABB;)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, other: AABB| this.intersect(&other),
        );
        golden.check_method(
            "minmax(Lnet/minecraft/world/phys/AABB;)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, other: AABB| this.minmax(&other),
        );
        golden.check_method(
            "move(DDD)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, xa, ya, za| this.move_xyz(xa, ya, za),
        );
        golden.check_method(
            "move(Lnet/minecraft/core/BlockPos;)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, pos: BlockPos| this.move_block_pos(&pos),
        );
        golden.check_method(
            "move(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, pos: Vec3| this.r#move(&pos),
        );
        golden.check_method(
            "move(Lorg/joml/Vector3f;)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, pos: Vector3f| this.move_vector3f(&pos),
        );
        golden.check_method(
            "intersects(Lnet/minecraft/world/phys/AABB;)Z",
            |this: &mut AABB, aabb: AABB| this.intersects(&aabb),
        );
        golden.check_method(
            "intersects(DDDDDD)Z",
            |this: &mut AABB, min_x, min_y, min_z, max_x, max_y, max_z| {
                this.intersects_xyz(min_x, min_y, min_z, max_x, max_y, max_z)
            },
        );
        golden.check_method(
            "intersects(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;)Z",
            |this: &mut AABB, min: Vec3, max: Vec3| this.intersects_vec3(&min, &max),
        );
        golden.check_method(
            "intersects(Lnet/minecraft/core/BlockPos;)Z",
            |this: &mut AABB, pos: BlockPos| this.intersects_block_pos(&pos),
        );
        golden.check_method(
            "contains(Lnet/minecraft/world/phys/Vec3;)Z",
            |this: &mut AABB, vec: Vec3| this.contains(&vec),
        );
        golden.check_method("contains(DDD)Z", |this: &mut AABB, x, y, z| {
            this.contains_xyz(x, y, z)
        });
        golden.check_method("getSize()D", |this: &mut AABB| this.get_size());
        golden.check_method("getXsize()D", |this: &mut AABB| this.get_xsize());
        golden.check_method("getYsize()D", |this: &mut AABB| this.get_ysize());
        golden.check_method("getZsize()D", |this: &mut AABB| this.get_zsize());
        golden.check_method(
            "deflate(DDD)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, x_substract, y_subtract, z_subtract| {
                this.deflate_xyz(x_substract, y_subtract, z_subtract)
            },
        );
        golden.check_method(
            "deflate(D)Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB, amount| this.deflate(amount),
        );
        golden.check_method(
            "clip(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;)Ljava/util/Optional;",
            |this: &mut AABB, from: Vec3, to: Vec3| this.clip(&from, &to),
        );
        golden.check(
            "clip(DDDDDDLnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;)Ljava/util/Optional;",
            |min_x, min_y, min_z, max_x, max_y, max_z, from: Vec3, to: Vec3| {
                AABB::clip_xyz(min_x, min_y, min_z, max_x, max_y, max_z, &from, &to)
            },
        );
        golden.check(
            "clip(Ljava/lang/Iterable;Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/core/BlockPos;)Lnet/minecraft/world/phys/BlockHitResult;",
            |aab_bs: Vec<AABB>, from: Vec3, to: Vec3, pos: BlockPos| {
                AABB::clip_aabbs(&aab_bs, &from, &to, &pos)
            },
        );
        golden.check_method(
            "collidedAlongVector(Lnet/minecraft/world/phys/Vec3;Ljava/util/List;)Z",
            |this: &mut AABB, vector: Vec3, aabbs: Vec<AABB>| {
                this.collided_along_vector(&vector, &aabbs)
            },
        );
        golden.check_method(
            "distanceToSqr(Lnet/minecraft/world/phys/Vec3;)D",
            |this: &mut AABB, point: Vec3| this.distance_to_sqr(&point),
        );
        golden.check_method(
            "distanceToSqr(Lnet/minecraft/world/phys/AABB;)D",
            |this: &mut AABB, bounding_box: AABB| this.distance_to_sqr_aabb(&bounding_box),
        );
        golden.check_method("hasNaN()Z", |this: &mut AABB| this.has_nan());
        golden.check_method(
            "getCenter()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut AABB| this.get_center(),
        );
        golden.check_method(
            "getBottomCenter()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut AABB| this.get_bottom_center(),
        );
        golden.check_method(
            "getMinPosition()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut AABB| this.get_min_position(),
        );
        golden.check_method(
            "getMaxPosition()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut AABB| this.get_max_position(),
        );
        golden.check(
            "ofSize(Lnet/minecraft/world/phys/Vec3;DDD)Lnet/minecraft/world/phys/AABB;",
            |center: Vec3, size_x, size_y, size_z| AABB::of_size(&center, size_x, size_y, size_z),
        );
        golden.check_method(
            "nextDeflated()Lnet/minecraft/world/phys/AABB;",
            |this: &mut AABB| this.next_deflated(),
        );
        golden.finish();
    }

    #[test]
    fn builder_matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.phys.AABB$Builder");
        golden.check("<init>()V", Builder::new);
        golden.check_method(
            "include(Lorg/joml/Vector3fc;)V",
            |this: &mut Builder, v: Vector3f| this.include(&v),
        );
        golden.check_method("isDefined()Z", |this: &mut Builder| this.is_defined());
        golden.check_method(
            "build()Lnet/minecraft/world/phys/AABB;",
            |this: &mut Builder| this.build(),
        );
        golden.finish();
    }
}
