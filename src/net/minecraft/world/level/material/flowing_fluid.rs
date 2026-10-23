//! net/minecraft/world/level/material/FlowingFluid.java
//!
//! What water and lava answer about their heights and flow: `getHeight` (a block's worth when the
//! same fluid is above, else its own), `getOwnHeight` (its amount in ninths) and `getFlow` (toward
//! lower neighbours, and down a falling fluid's solid sides) and `getShape`. Not ported yet:
//! ticking and spreading (`tick`, `spread`, `getNewLiquid`, the slope search) and `canHoldFluid`.
//!
//! `getShape` keeps the first shape it makes for each state (`shapes`, an `IdentityHashMap` of the
//! fluid's), though the height it's made from depends on what's above the block: a state's shape
//! is its height where it was first asked for. Java's map lives as long as the game's fluids, a
//! JVM's; here it's the thread's, and a new game (`Minecraft::new`) starts it empty.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Direction, Plane};
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::world::level::block::ice_block;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::net::minecraft::world::phys::vec3::Vec3;

thread_local! {
    /// `shapes`, water's and lava's (each `FlowingFluid` has its own; states are told apart by
    /// their fluid anyway).
    static SHAPES: RefCell<HashMap<FluidState, VoxelShape>> = RefCell::new(HashMap::new());
}

/// Empties the shapes kept so far, as a new game's fluids start.
pub fn clear_shapes() {
    SHAPES.with(|shapes| shapes.borrow_mut().clear());
}

/// `getShape`: the whole block for a full fluid under the same fluid, else a box as high as the
/// fluid where the state was first asked for.
pub fn get_shape(state: FluidState, level: &dyn BlockGetter, pos: &BlockPos) -> VoxelShape {
    if state.get_amount() == 9 && has_same_above(state, level, pos) {
        return Shapes::block();
    }

    SHAPES.with(|shapes| {
        shapes
            .borrow_mut()
            .entry(state)
            .or_insert_with(|| {
                Shapes::r#box(
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                    f64::from(state.get_height(level, pos)),
                    1.0,
                )
            })
            .clone()
    })
}

/// `getFlow`: each horizontal neighbour the fluid can flow into pulls toward it by how much lower
/// it is (or, where it's empty over the same fluid, by that fluid's height below), normalized;
/// a falling fluid next to a solid face also flows down.
pub fn get_flow(
    this: Fluids,
    level: &dyn BlockGetter,
    pos: &BlockPos,
    fluid_state: FluidState,
) -> Vec3 {
    let mut flow_x = 0.0;
    let mut flow_z = 0.0;

    for direction in Plane::HORIZONTAL.iterator() {
        let block_pos = pos.relative(direction);
        let neighbour_fluid = level.get_fluid_state(&block_pos);
        if affects_flow(this, neighbour_fluid) {
            let mut neighbor_height = neighbour_fluid.get_own_height();
            let mut distance = 0.0_f32;
            if neighbor_height == 0.0 {
                if !level
                    .get_block_state(&block_pos)
                    .is_tag(BlockTags::BLOCKS_FLUID_FLOW)
                {
                    let neighbor_pos = block_pos.below();
                    let below_neighbor_state = level.get_fluid_state(&neighbor_pos);
                    if affects_flow(this, below_neighbor_state) {
                        neighbor_height = below_neighbor_state.get_own_height();
                        if neighbor_height > 0.0 {
                            distance = fluid_state.get_own_height() - (neighbor_height - 0.8888889);
                        }
                    }
                }
            } else if neighbor_height > 0.0 {
                distance = fluid_state.get_own_height() - neighbor_height;
            }

            if distance != 0.0 {
                flow_x += f64::from(direction.get_step_x() as f32 * distance);
                flow_z += f64::from(direction.get_step_z() as f32 * distance);
            }
        }
    }

    let mut flow = Vec3::new(flow_x, 0.0, flow_z);
    if fluid_state.falling() == Some(true) {
        for direction in Plane::HORIZONTAL.iterator() {
            let block_pos = pos.relative(direction);
            if is_solid_face(this, level, &block_pos, direction)
                || is_solid_face(this, level, &block_pos.above(), direction)
            {
                flow = flow.normalize().add_xyz(0.0, -6.0, 0.0);
                break;
            }
        }
    }

    flow.normalize()
}

/// `affectsFlow`: an empty neighbour or one of the same fluid.
fn affects_flow(this: Fluids, neighbour_fluid: FluidState) -> bool {
    neighbour_fluid.is_empty() || neighbour_fluid.get_type().is_same(this)
}

/// `isSolidFace`: a face of another fluid's block that's sturdy, ice's never.
fn is_solid_face(
    this: Fluids,
    level: &dyn BlockGetter,
    pos: &BlockPos,
    direction: Direction,
) -> bool {
    let state = level.get_block_state(pos);
    let fluid_state = level.get_fluid_state(pos);
    if fluid_state.get_type().is_same(this) {
        false
    } else if direction == Direction::UP {
        true
    } else if state.get_block().is_instance_of(&ice_block::CLASS) {
        false
    } else {
        state.is_face_sturdy(level, pos, direction)
    }
}

/// `hasSameAbove`.
fn has_same_above(fluid_state: FluidState, level: &dyn BlockGetter, pos: &BlockPos) -> bool {
    fluid_state
        .get_type()
        .is_same(level.get_fluid_state(&pos.above()).get_type())
}

/// `getHeight`: a whole block under the same fluid, else its own height.
pub fn get_height(fluid_state: FluidState, level: &dyn BlockGetter, pos: &BlockPos) -> f32 {
    if has_same_above(fluid_state, level, pos) {
        1.0
    } else {
        fluid_state.get_own_height()
    }
}

/// `getOwnHeight`: its amount in ninths of a block.
pub fn get_own_height(fluid_state: FluidState) -> f32 {
    fluid_state.get_amount() as f32 / 9.0
}
