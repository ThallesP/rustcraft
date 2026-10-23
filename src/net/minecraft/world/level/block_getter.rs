//! net/minecraft/world/level/BlockGetter.java
//!
//! What collisions, the renderer and physics read blocks through: a level, one of its chunks, or
//! a copy of its sections, and the ray cast through its blocks (`clip`, with `traverseBlocks` and
//! `clipWithInteractionOverride`). Not ported yet: `getBlockEntity` (block entities),
//! `getBlockStates(AABB)`, `isBlockInLine` and `clipShape`. Java's extends
//! `LevelHeightAccessor`, which is a trait of its own here that the levels and chunks implement,
//! so that what only reads blocks (the shapes' `EmptyBlockGetter`, the collisions' tests) needs
//! no height.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::clip_context::ClipContext;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::phys::block_hit_result::BlockHitResult;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// Something blocks can be read from: a level, or one of its chunks.
pub trait BlockGetter {
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
