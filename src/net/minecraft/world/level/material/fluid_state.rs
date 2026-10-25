//! net/minecraft/world/level/material/FluidState.java
//!
//! A fluid state is a `StateHolder` of its fluid in Java. A fluid has at most two properties
//! (`FlowingFluid.FALLING`, and `LEVEL_FLOWING` for a flowing one), so here a state is its fluid
//! and their values, `Copy` and compared by value, which is comparing Java's references since a
//! definition holds one state for each combination. Its heights and flow (`getHeight`,
//! `getHeightForCamera`, `getOwnHeight`, `getFlow`, `getAABB`, `entityInside`) and its animations
//! (`animateTick`, `getDripParticle`) ask its fluid (`Fluid`). Not ported yet:
//! `Fluid.FLUID_STATE_REGISTRY`'s ids and the other behaviour methods (`getShape`, ticking), which
//! come with what calls them.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::particle_options::ParticleOptions;
use crate::net::minecraft::tags::fluid_tags::FluidTagKey;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::entity::inside_block_effect_applier::InsideBlockEffectApplier;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::liquid_block::LiquidBlock;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::level::Level;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::level::material::{lava_fluid, water_fluid};
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// One of a fluid's states.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FluidState {
    fluid: Fluids,
    falling: bool,
    // `LEVEL_FLOWING`'s value for a flowing fluid, 0 for the others.
    level: i32,
}

impl FluidState {
    /// `Fluids.EMPTY.defaultFluidState()`.
    pub const EMPTY: FluidState = FluidState {
        fluid: Fluids::EMPTY,
        falling: false,
        level: 0,
    };

    pub(crate) const fn new(fluid: Fluids, falling: bool, level: i32) -> FluidState {
        FluidState {
            fluid,
            falling,
            level,
        }
    }

    pub fn get_type(&self) -> Fluids {
        self.fluid
    }

    /// `FlowingFluid.FALLING`'s value, which the empty fluid's state doesn't have (`None`).
    pub fn falling(&self) -> Option<bool> {
        (!self.fluid.is_empty()).then_some(self.falling)
    }

    /// `FlowingFluid.LEVEL`'s value, which only a flowing fluid's states have.
    pub fn level(&self) -> Option<i32> {
        self.fluid.is_flowing().then_some(self.level)
    }

    /// `getType().isSource(this)`: a source fluid's states are.
    pub fn is_source(&self) -> bool {
        matches!(self.fluid, Fluids::WATER | Fluids::LAVA)
    }

    pub fn is_empty(&self) -> bool {
        self.fluid.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.get_amount() == 8
    }

    /// 8 for a source, the level for a flowing fluid, 0 for the empty one.
    pub fn get_amount(&self) -> i32 {
        match self.fluid {
            Fluids::EMPTY => 0,
            Fluids::WATER | Fluids::LAVA => 8,
            Fluids::FLOWING_WATER | Fluids::FLOWING_LAVA => self.level,
        }
    }

    /// `createLegacyBlock()`: the block a fluid alone is, water or lava at its legacy level
    /// (`FlowingFluid.getLegacyLevel`), air for the empty fluid.
    pub fn create_legacy_block(&self) -> BlockState {
        let block = match self.fluid {
            Fluids::EMPTY => return Blocks::AIR.default_block_state(),
            Fluids::WATER | Fluids::FLOWING_WATER => Blocks::WATER,
            Fluids::LAVA | Fluids::FLOWING_LAVA => Blocks::LAVA,
        };
        let legacy_level = if self.is_source() {
            0
        } else {
            8 - self.get_amount().min(8) + if self.falling { 8 } else { 0 }
        };
        block
            .default_block_state()
            .set_value(LiquidBlock::LEVEL, legacy_level)
    }

    pub fn is_randomly_ticking(&self) -> bool {
        self.fluid.is_randomly_ticking()
    }

    /// `animateTick(Level, BlockPos, RandomSource)`: its fluid's sounds and particles; the empty
    /// fluid has none.
    pub fn animate_tick(
        &self,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        match self.fluid {
            Fluids::EMPTY => {}
            Fluids::WATER | Fluids::FLOWING_WATER => {
                water_fluid::animate_tick(level, pos, *self, random)
            }
            Fluids::LAVA | Fluids::FLOWING_LAVA => lava_fluid::animate_tick(level, pos, random),
        }
    }

    /// `getDripParticle()`: what drips from under it, none for the empty fluid.
    pub fn get_drip_particle(&self) -> Option<ParticleOptions> {
        match self.fluid {
            Fluids::EMPTY => None,
            Fluids::WATER | Fluids::FLOWING_WATER => Some(water_fluid::get_drip_particle()),
            Fluids::LAVA | Fluids::FLOWING_LAVA => Some(lava_fluid::get_drip_particle()),
        }
    }

    /// `isSourceOfType(Fluid)`: a source of that fluid.
    pub fn is_source_of_type(&self, fluid: Fluids) -> bool {
        self.fluid == fluid && self.is_source()
    }

    /// `is(TagKey<Fluid>)`: whether its fluid is in the tag.
    pub fn is_tag(&self, tag: FluidTagKey) -> bool {
        self.fluid.is_tag(tag)
    }

    /// `getHeight(BlockGetter, BlockPos)`: how high it reaches in its block.
    pub fn get_height(&self, level: &dyn BlockGetter, pos: &BlockPos) -> f32 {
        self.fluid.get_height(*self, level, pos)
    }

    /// `getHeightForCamera`: a whole block for a source under a block whose bottom is sturdy, else
    /// its height.
    pub fn get_height_for_camera(&self, level: &dyn BlockGetter, pos: &BlockPos) -> f32 {
        if self.is_source() {
            let pos_above = pos.above();
            if level
                .get_block_state(&pos_above)
                .is_face_sturdy(level, &pos_above, Direction::DOWN)
            {
                return 1.0;
            }
        }

        self.get_height(level, pos)
    }

    /// `getOwnHeight()`: its height without what's above it.
    pub fn get_own_height(&self) -> f32 {
        self.fluid.get_own_height(*self)
    }

    /// `getShape(BlockGetter, BlockPos)`.
    pub fn get_shape(&self, level: &dyn BlockGetter, pos: &BlockPos) -> VoxelShape {
        self.fluid.get_shape(*self, level, pos)
    }

    /// `shouldRenderBackwardUpFace`: whether a block around the one above isn't this fluid and
    /// lets light through, so the top can be seen from below.
    pub fn should_render_backward_up_face(
        &self,
        level: &dyn BlockGetter,
        above: &BlockPos,
    ) -> bool {
        for ox in -1..=1 {
            for oz in -1..=1 {
                let offset = above.offset_xyz(ox, 0, oz);
                let fluid_state = level.get_fluid_state(&offset);
                if !fluid_state.get_type().is_same(self.get_type())
                    && !level.get_block_state(&offset).is_solid_render()
                {
                    return true;
                }
            }
        }

        false
    }

    /// `getFlow(BlockGetter, BlockPos)`: the direction it flows in, a unit vector or zero.
    pub fn get_flow(&self, level: &dyn BlockGetter, pos: &BlockPos) -> Vec3 {
        self.fluid.get_flow(level, pos, *self)
    }

    /// `getAABB(BlockGetter, BlockPos)`: its box in the block, none when it's empty.
    pub fn get_aabb(&self, level: &dyn BlockGetter, pos: &BlockPos) -> Option<AABB> {
        self.fluid.get_aabb(*self, level, pos)
    }

    /// `entityInside(Level, BlockPos, Entity, InsideBlockEffectApplier)`: its fluid's, which only
    /// asks for effects, whatever the level, the position or the entity.
    pub fn entity_inside(
        &self,
        _level: &dyn Level,
        _pos: &BlockPos,
        _entity: &mut dyn EntityClass,
        effect_applier: &mut dyn InsideBlockEffectApplier,
    ) {
        self.fluid.entity_inside(effect_applier);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::{Value, json};

    use super::FluidState;
    use crate::golden::Encode;

    /// `[fluid, values, isEmpty, isRandomlyTicking, isSource, getAmount]`, its values by their
    /// fields in `BlockStateProperties`, like a block state's.
    impl Encode for FluidState {
        fn encode(&self) -> Value {
            let mut values = Vec::new();
            if let Some(falling) = self.falling() {
                values.push(json!(["FALLING", falling]));
            }
            if let Some(level) = self.level() {
                values.push(json!(["LEVEL_FLOWING", level]));
            }
            json!([
                self.get_type().key(),
                values,
                self.is_empty(),
                self.is_randomly_ticking(),
                self.is_source(),
                self.get_amount()
            ])
        }
    }
}
