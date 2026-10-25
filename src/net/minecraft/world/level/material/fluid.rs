//! net/minecraft/world/level/material/Fluid.java
//!
//! What a fluid's states answer about their height and flow, which an entity in a fluid reads:
//! Java's abstract methods, dispatched here on the fluid to its class's override (`EmptyFluid`, or
//! `FlowingFluid` for water and lava), `isSame` and `entityInside`, which `WaterFluid` and
//! `LavaFluid` override, and `getAABB`. Not ported yet: ticking, spreading, sounds and particles.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::entity::inside_block_effect_applier::InsideBlockEffectApplier;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::level::material::{
    empty_fluid, flowing_fluid, lava_fluid, water_fluid,
};
use crate::net::minecraft::world::phys::aabb::AABB;
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

    /// `entityInside(Level, BlockPos, Entity, InsideBlockEffectApplier)`: `Fluid`'s does nothing.
    pub fn entity_inside(&self, effect_applier: &mut dyn InsideBlockEffectApplier) {
        match self {
            Fluids::WATER | Fluids::FLOWING_WATER => water_fluid::entity_inside(effect_applier),
            Fluids::LAVA | Fluids::FLOWING_LAVA => lava_fluid::entity_inside(effect_applier),
            Fluids::EMPTY => {}
        }
    }

    /// `getAABB(FluidState, BlockGetter, BlockPos)`: the block up to the fluid's height, none for
    /// no fluid.
    pub fn get_aabb(
        &self,
        state: FluidState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> Option<AABB> {
        if self.is_empty() {
            return None;
        }

        let height = state.get_height(level, pos);
        // `pos.getY() + height` adds as floats.
        Some(AABB::new(
            f64::from(pos.get_x()),
            f64::from(pos.get_y()),
            f64::from(pos.get_z()),
            f64::from(pos.get_x()) + 1.0,
            f64::from(pos.get_y() as f32 + height),
            f64::from(pos.get_z()) + 1.0,
        ))
    }
}
