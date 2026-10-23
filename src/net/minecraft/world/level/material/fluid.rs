//! net/minecraft/world/level/material/Fluid.java
//!
//! What a fluid's states answer about their height and flow, which an entity in a fluid reads:
//! Java's abstract methods, dispatched here on the fluid to its class's override (`EmptyFluid`, or
//! `FlowingFluid` for water and lava), and `isSame`, which `WaterFluid` and `LavaFluid` override.
//! Not ported yet: ticking, spreading, shapes, sounds and particles.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::level::material::{
    empty_fluid, flowing_fluid, lava_fluid, water_fluid,
};
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::net::minecraft::world::phys::vec3::Vec3;

impl Fluids {
    /// `isSame(Fluid)`: the same fluid, or for water and lava, still or flowing.
    pub fn is_same(&self, other: Fluids) -> bool {
        match self {
            Fluids::WATER | Fluids::FLOWING_WATER => water_fluid::is_same(other),
            Fluids::LAVA | Fluids::FLOWING_LAVA => lava_fluid::is_same(other),
            Fluids::EMPTY => other == *self,
        }
    }

    /// `getFlow(BlockGetter, BlockPos, FluidState)`.
    pub fn get_flow(
        &self,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        fluid_state: FluidState,
    ) -> Vec3 {
        match self {
            Fluids::EMPTY => empty_fluid::get_flow(),
            _ => flowing_fluid::get_flow(*self, level, pos, fluid_state),
        }
    }

    /// `getHeight(FluidState, BlockGetter, BlockPos)`.
    pub fn get_height(
        &self,
        fluid_state: FluidState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> f32 {
        match self {
            Fluids::EMPTY => empty_fluid::get_height(),
            _ => flowing_fluid::get_height(fluid_state, level, pos),
        }
    }

    /// `getShape(FluidState, BlockGetter, BlockPos)`.
    pub fn get_shape(
        &self,
        fluid_state: FluidState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> VoxelShape {
        match self {
            Fluids::EMPTY => Shapes::empty(),
            _ => flowing_fluid::get_shape(fluid_state, level, pos),
        }
    }

    /// `getOwnHeight(FluidState)`.
    pub fn get_own_height(&self, fluid_state: FluidState) -> f32 {
        match self {
            Fluids::EMPTY => empty_fluid::get_own_height(),
            _ => flowing_fluid::get_own_height(fluid_state),
        }
    }
}
