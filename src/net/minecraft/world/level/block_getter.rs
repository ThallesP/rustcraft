//! net/minecraft/world/level/BlockGetter.java
//!
//! What collisions, the renderer and physics read blocks through: a level, one of its chunks, or
//! a copy of its sections, the ray cast through its blocks (`clip`, with `traverseBlocks` and
//! `clipWithInteractionOverride`), and the blocks a box passes through on its way somewhere
//! (`forEachBlockIntersectedBetween`, whose visitor, Java's `BlockStepVisitor`, is an `FnMut`). Its
//! `getBlockEntity` is the block entity a level or chunk holds there, borrowed, without making one
//! (Java's level makes one, `IMMEDIATE`, which only a block whose chunk came without its block
//! entity lacks). Not ported yet: `getBlockStates(AABB)`, `isBlockInLine` and `clipShape`. Java's
//! extends `LevelHeightAccessor`, which is a trait of its own here that the levels and chunks
//! implement, so that what only reads blocks (the shapes' `EmptyBlockGetter`, the collisions'
//! tests) needs no height.

use std::cell::Ref;
use std::collections::HashSet;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::vec3i::Vec3i;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntity;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::clip_context::ClipContext;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::block_hit_result::BlockHitResult;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// Something blocks can be read from: a level, or one of its chunks.
pub trait BlockGetter {
    /// `getBlockEntity(pos)`: none where it keeps no block entities.
    fn get_block_entity(&self, _pos: &BlockPos) -> Option<Ref<'_, BlockEntity>> {
        None
    }

    fn get_block_state(&self, pos: &BlockPos) -> BlockState;

    fn get_fluid_state(&self, pos: &BlockPos) -> FluidState;

    fn get_light_emission(&self, pos: &BlockPos) -> i32 {
        self.get_block_state(pos).get_light_emission()
    }
}

/// `clip(ClipContext)`: the first block or fluid shape the ray hits, the nearer of a block's two
/// where both are hit, or a miss at its end.
pub fn clip(level: &dyn BlockGetter, c: &ClipContext) -> BlockHitResult {
    traverse_blocks(
        &c.get_from(),
        &c.get_to(),
        c,
        |context, pos| {
            let block_state = level.get_block_state(pos);
            let fluid_state = level.get_fluid_state(pos);
            let from = context.get_from();
            let to = context.get_to();
            let block_shape = context.get_block_shape(block_state, level, pos);
            let block_result =
                clip_with_interaction_override(level, &from, &to, pos, &block_shape, block_state);
            let fluid_shape = context.get_fluid_shape(fluid_state, level, pos);
            let liquid_result = fluid_shape.clip(&from, &to, pos);
            let block_distance_squared = block_result.as_ref().map_or(f64::MAX, |result| {
                context.get_from().distance_to_sqr(&result.get_location())
            });
            let liquid_distance_squared = liquid_result.as_ref().map_or(f64::MAX, |result| {
                context.get_from().distance_to_sqr(&result.get_location())
            });
            if block_distance_squared <= liquid_distance_squared {
                block_result
            } else {
                liquid_result
            }
        },
        |context| {
            let delta = context.get_from().subtract(&context.get_to());
            BlockHitResult::miss(
                &context.get_to(),
                Direction::get_approximate_nearest_f64(delta.x, delta.y, delta.z),
                &BlockPos::containing(&context.get_to()),
            )
        },
    )
}

/// `clipWithInteractionOverride`: where the ray hits the shape, facing where it hits the block's
/// interaction shape if it hits that first.
pub fn clip_with_interaction_override(
    level: &dyn BlockGetter,
    from: &Vec3,
    to: &Vec3,
    pos: &BlockPos,
    block_shape: &VoxelShape,
    block_state: BlockState,
) -> Option<BlockHitResult> {
    let result = block_shape.clip(from, to, pos)?;
    if let Some(hit_override) = block_state
        .get_interaction_shape(level, pos)
        .clip(from, to, pos)
        && hit_override.get_location().subtract(from).length_sqr()
            < result.get_location().subtract(from).length_sqr()
    {
        return Some(result.with_direction(hit_override.get_direction()));
    }

    Some(result)
}

/// `traverseBlocks`: the blocks a segment passes through, in order (an amanatides-woo walk from
/// just before `from` to just short of `to`), until the consumer answers; the miss otherwise.
pub fn traverse_blocks<C, T>(
    from: &Vec3,
    to: &Vec3,
    context: &C,
    mut consumer: impl FnMut(&C, &BlockPos) -> Option<T>,
    miss_factory: impl Fn(&C) -> T,
) -> T {
    if from == to {
        return miss_factory(context);
    }

    let to_x = Mth::lerp_f64(-1.0E-7, to.x, from.x);
    let to_y = Mth::lerp_f64(-1.0E-7, to.y, from.y);
    let to_z = Mth::lerp_f64(-1.0E-7, to.z, from.z);
    let from_x = Mth::lerp_f64(-1.0E-7, from.x, to.x);
    let from_y = Mth::lerp_f64(-1.0E-7, from.y, to.y);
    let from_z = Mth::lerp_f64(-1.0E-7, from.z, to.z);
    let mut current_block_x = Mth::floor_f64(from_x);
    let mut current_block_y = Mth::floor_f64(from_y);
    let mut current_block_z = Mth::floor_f64(from_z);
    let pos = BlockPos::new(current_block_x, current_block_y, current_block_z);
    if let Some(first) = consumer(context, &pos) {
        return first;
    }

    let dx = to_x - from_x;
    let dy = to_y - from_y;
    let dz = to_z - from_z;
    let sign_x = Mth::sign(dx);
    let sign_y = Mth::sign(dy);
    let sign_z = Mth::sign(dz);
    let t_delta_x = if sign_x == 0 {
        f64::MAX
    } else {
        f64::from(sign_x) / dx
    };
    let t_delta_y = if sign_y == 0 {
        f64::MAX
    } else {
        f64::from(sign_y) / dy
    };
    let t_delta_z = if sign_z == 0 {
        f64::MAX
    } else {
        f64::from(sign_z) / dz
    };
    let mut t_x = t_delta_x
        * if sign_x > 0 {
            1.0 - Mth::frac_f64(from_x)
        } else {
            Mth::frac_f64(from_x)
        };
    let mut t_y = t_delta_y
        * if sign_y > 0 {
            1.0 - Mth::frac_f64(from_y)
        } else {
            Mth::frac_f64(from_y)
        };
    let mut t_z = t_delta_z
        * if sign_z > 0 {
            1.0 - Mth::frac_f64(from_z)
        } else {
            Mth::frac_f64(from_z)
        };

    while t_x <= 1.0 || t_y <= 1.0 || t_z <= 1.0 {
        if t_x < t_y {
            if t_x < t_z {
                current_block_x += sign_x;
                t_x += t_delta_x;
            } else {
                current_block_z += sign_z;
                t_z += t_delta_z;
            }
        } else if t_y < t_z {
            current_block_y += sign_y;
            t_y += t_delta_y;
        } else {
            current_block_z += sign_z;
            t_z += t_delta_z;
        }

        let pos = BlockPos::new(current_block_x, current_block_y, current_block_z);
        if let Some(result) = consumer(context, &pos) {
            return result;
        }
    }

    miss_factory(context)
}

/// `forEachBlockIntersectedBetween(Vec3, Vec3, AABB, BlockStepVisitor)`: every block the box
/// passes through on its way from `from` to `to` (where it's `aabb_at_target`), each once, with
/// the step it's met at: the blocks of the box where it starts (step 0), those its leading corner
/// passes through and the box with it, then those of the box where it ends; only those of the
/// box at the end for a movement too small to walk. False when the visitor stopped the walk.
pub fn for_each_block_intersected_between(
    from: &Vec3,
    to: &Vec3,
    aabb_at_target: &AABB,
    visitor: &mut dyn FnMut(&BlockPos, i32) -> bool,
) -> bool {
    let travel = to.subtract(from);
    if travel.length_sqr() < f64::from(Mth::square_f32(1.0E-5)) {
        for block_pos in BlockPos::between_closed_aabb(aabb_at_target) {
            if !visitor(&block_pos, 0) {
                return false;
            }
        }

        true
    } else {
        // A `LongOpenHashSet`, which is only asked what it holds.
        let mut visited_blocks = HashSet::new();

        for block_pos in BlockPos::between_corners_in_direction_aabb(
            &aabb_at_target.r#move(&travel.scale(-1.0)),
            &travel,
        ) {
            if !visitor(&block_pos, 0) {
                return false;
            }

            visited_blocks.insert(block_pos.as_long());
        }

        let iterations =
            add_collisions_along_travel(&mut visited_blocks, &travel, aabb_at_target, visitor);
        if iterations < 0 {
            return false;
        }

        for block_pos in BlockPos::between_corners_in_direction_aabb(aabb_at_target, &travel) {
            if visited_blocks.insert(block_pos.as_long()) && !visitor(&block_pos, iterations + 1) {
                return false;
            }
        }

        true
    }
}

/// `addCollisionsAlongTravel`: the blocks the box's leading corner passes through, walked like a
/// ray, and for each the blocks of the box with its corner there, not visited yet; how many steps
/// it took, or -1 when the visitor stopped the walk.
fn add_collisions_along_travel(
    visited_blocks: &mut HashSet<i64>,
    delta_move: &Vec3,
    aabb_at_target: &AABB,
    visitor: &mut dyn FnMut(&BlockPos, i32) -> bool,
) -> i32 {
    let box_size_x = aabb_at_target.get_xsize();
    let box_size_y = aabb_at_target.get_ysize();
    let box_size_z = aabb_at_target.get_zsize();
    let corner_dir = get_furthest_corner(delta_move);
    let to_center = aabb_at_target.get_center();
    let to_corner = Vec3::new(
        to_center.x + box_size_x * 0.5 * f64::from(corner_dir.get_x()),
        to_center.y + box_size_y * 0.5 * f64::from(corner_dir.get_y()),
        to_center.z + box_size_z * 0.5 * f64::from(corner_dir.get_z()),
    );
    let from_corner = to_corner.subtract(delta_move);
    let mut corner_visited_block_x = Mth::floor_f64(from_corner.x);
    let mut corner_visited_block_y = Mth::floor_f64(from_corner.y);
    let mut corner_visited_block_z = Mth::floor_f64(from_corner.z);
    let sign_x = Mth::sign(delta_move.x);
    let sign_y = Mth::sign(delta_move.y);
    let sign_z = Mth::sign(delta_move.z);
    let t_delta_x = if sign_x == 0 {
        f64::MAX
    } else {
        f64::from(sign_x) / delta_move.x
    };
    let t_delta_y = if sign_y == 0 {
        f64::MAX
    } else {
        f64::from(sign_y) / delta_move.y
    };
    let t_delta_z = if sign_z == 0 {
        f64::MAX
    } else {
        f64::from(sign_z) / delta_move.z
    };
    let mut t_x = t_delta_x
        * if sign_x > 0 {
            1.0 - Mth::frac_f64(from_corner.x)
        } else {
            Mth::frac_f64(from_corner.x)
        };
    let mut t_y = t_delta_y
        * if sign_y > 0 {
            1.0 - Mth::frac_f64(from_corner.y)
        } else {
            Mth::frac_f64(from_corner.y)
        };
    let mut t_z = t_delta_z
        * if sign_z > 0 {
            1.0 - Mth::frac_f64(from_corner.z)
        } else {
            Mth::frac_f64(from_corner.z)
        };
    let mut iterations = 0;

    while t_x <= 1.0 || t_y <= 1.0 || t_z <= 1.0 {
        if t_x < t_y {
            if t_x < t_z {
                corner_visited_block_x += sign_x;
                t_x += t_delta_x;
            } else {
                corner_visited_block_z += sign_z;
                t_z += t_delta_z;
            }
        } else if t_y < t_z {
            corner_visited_block_y += sign_y;
            t_y += t_delta_y;
        } else {
            corner_visited_block_z += sign_z;
            t_z += t_delta_z;
        }

        let hit_point = AABB::clip_xyz(
            f64::from(corner_visited_block_x),
            f64::from(corner_visited_block_y),
            f64::from(corner_visited_block_z),
            f64::from(corner_visited_block_x + 1),
            f64::from(corner_visited_block_y + 1),
            f64::from(corner_visited_block_z + 1),
            &from_corner,
            &to_corner,
        );
        if let Some(hit_point) = hit_point {
            iterations += 1;
            // `cornerVisitedBlockX + 1.0E-5F`, which the bytecode adds as doubles.
            let corner_hit_x = Mth::clamp_f64(
                hit_point.x,
                f64::from(corner_visited_block_x) + f64::from(1.0E-5_f32),
                f64::from(corner_visited_block_x) + 1.0 - f64::from(1.0E-5_f32),
            );
            let corner_hit_y = Mth::clamp_f64(
                hit_point.y,
                f64::from(corner_visited_block_y) + f64::from(1.0E-5_f32),
                f64::from(corner_visited_block_y) + 1.0 - f64::from(1.0E-5_f32),
            );
            let corner_hit_z = Mth::clamp_f64(
                hit_point.z,
                f64::from(corner_visited_block_z) + f64::from(1.0E-5_f32),
                f64::from(corner_visited_block_z) + 1.0 - f64::from(1.0E-5_f32),
            );
            let opposite_corner_x =
                Mth::floor_f64(corner_hit_x - box_size_x * f64::from(corner_dir.get_x()));
            let opposite_corner_y =
                Mth::floor_f64(corner_hit_y - box_size_y * f64::from(corner_dir.get_y()));
            let opposite_corner_z =
                Mth::floor_f64(corner_hit_z - box_size_z * f64::from(corner_dir.get_z()));
            let current_iteration = iterations;

            for pos in BlockPos::between_corners_in_direction_xyz(
                corner_visited_block_x,
                corner_visited_block_y,
                corner_visited_block_z,
                opposite_corner_x,
                opposite_corner_y,
                opposite_corner_z,
                delta_move,
            ) {
                if visited_blocks.insert(pos.as_long()) && !visitor(&pos, current_iteration) {
                    return -1;
                }
            }
        }
    }

    iterations
}

/// `getFurthestCorner`: the corner of a box that leads it along the direction, as the signs of
/// its offset from the center.
fn get_furthest_corner(direction: &Vec3) -> Vec3i {
    let x_dot = Vec3::X_AXIS.dot(direction).abs();
    let y_dot = Vec3::Y_AXIS.dot(direction).abs();
    let z_dot = Vec3::Z_AXIS.dot(direction).abs();
    let x_sign = if direction.x >= 0.0 { 1 } else { -1 };
    let y_sign = if direction.y >= 0.0 { 1 } else { -1 };
    let z_sign = if direction.z >= 0.0 { 1 } else { -1 };
    if x_dot <= y_dot && x_dot <= z_dot {
        Vec3i::new(-x_sign, -z_sign, y_sign)
    } else if y_dot <= z_dot {
        Vec3i::new(z_sign, -y_sign, -x_sign)
    } else {
        Vec3i::new(-y_sign, x_sign, -z_sign)
    }
}

#[cfg(test)]
mod tests {
    use super::for_each_block_intersected_between;
    use crate::golden::Golden;
    use crate::net::minecraft::world::phys::aabb::AABB;
    use crate::net::minecraft::world::phys::vec3::Vec3;

    /// The walk `Entity.checkInsideBlocks` takes (`ShapeSuites.BlockGetterSuite`): every block
    /// visited with its step, and whether the walk ended, the visitor stopping it after its nth
    /// visit (-1 for never).
    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.BlockGetter");
        golden.check(
            "forEachBlockIntersectedBetween(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/AABB;Lnet/minecraft/world/level/BlockGetter$BlockStepVisitor;)Z",
            |from: Vec3, to: Vec3, aabb_at_target: AABB, stop_after: i32| {
                let mut visits: Vec<[i32; 4]> = Vec::new();
                let ended = for_each_block_intersected_between(
                    &from,
                    &to,
                    &aabb_at_target,
                    &mut |pos, iteration| {
                        visits.push([pos.get_x(), pos.get_y(), pos.get_z(), iteration]);
                        stop_after < 0 || (visits.len() as i32) < stop_after
                    },
                );
                (visits, ended)
            },
        );
        golden.finish();
    }
}
