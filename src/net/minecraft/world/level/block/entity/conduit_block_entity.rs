//! net/minecraft/world/level/block/entity/ConduitBlockEntity.java
//!
//! A conduit as the client ticks it (`clientTick`): every 40 game ticks it looks again for the
//! water around it and the frame of prismarine (`updateShape`), which makes it active (16 frame
//! blocks) and hunting (42); its nautilus particles drift from the frame to its eye, one in fifty
//! of the frame's blocks a tick, from the level's random (`animationTick`); and it turns while
//! active. Not ported yet: the target it attacks (`Target`, an `EntityReference` to a living
//! entity, whose particles go to it): the level's other entities aren't told apart as living
//! entities yet, so a conduit finds no target; and the server's ticker, its effects, attacks and
//! sounds.

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::tags::fluid_tags::FluidTags;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntityData;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::vec3::Vec3;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ConduitBlockEntity {
    pub tick_count: i32,
    active_rotation: f32,
    is_active: bool,
    is_hunting: bool,
    effect_blocks: Vec<BlockPos>,
}

impl ConduitBlockEntity {
    const MIN_ACTIVE_SIZE: usize = 16;
    const MIN_KILL_SIZE: usize = 42;

    pub fn new() -> ConduitBlockEntity {
        ConduitBlockEntity::default()
    }

    /// `clientTick`.
    pub fn client_tick(level: &mut LevelTicking<'_>, pos: &BlockPos, _state: BlockState) {
        let effects = level.effects_level();
        let mut block_entity = effects
            .get_block_entity_mut(pos)
            .expect("a ticker's block entity is the level's");
        let BlockEntityData::Conduit(entity) = &mut block_entity.data else {
            return;
        };
        entity.tick_count += 1;
        let game_time = effects.get_game_time();
        if game_time % 40 == 0 {
            entity.is_active =
                ConduitBlockEntity::update_shape(&*effects, pos, &mut entity.effect_blocks);
            entity.is_hunting = entity.effect_blocks.len() >= ConduitBlockEntity::MIN_KILL_SIZE;
        }

        // `EntityReference.getLivingEntity(destroyTarget, level)`: none (above). The level's random
        // is borrowed a draw at a time: adding a particle may draw from it too.
        let hh = f64::from(
            Mth::sin(f64::from(entity.tick_count.wrapping_add(35) as f32 * 0.1)) / 2.0 + 0.5,
        );
        let hh = (hh * hh + hh) * f64::from(0.3_f32);
        let particle_end = Vec3::new(
            f64::from(pos.get_x()) + 0.5,
            f64::from(pos.get_y()) + 1.5 + hh,
            f64::from(pos.get_z()) + 0.5,
        );
        for effect_pos in &entity.effect_blocks {
            if effects.get_random().next_int_bounded(50) == 0 {
                let delta = effect_pos.subtract(pos);
                let dx = -0.5 + effects.get_random().next_float() + delta.get_x() as f32;
                let dy = -2.0 + effects.get_random().next_float() + delta.get_y() as f32;
                let dz = -0.5 + effects.get_random().next_float() + delta.get_z() as f32;
                effects.add_particle(
                    ParticleTypes::NAUTILUS,
                    particle_end.x,
                    particle_end.y,
                    particle_end.z,
                    f64::from(dx),
                    f64::from(dy),
                    f64::from(dz),
                );
            }
        }

        if entity.is_active {
            entity.active_rotation += 1.0;
        }
    }

    /// `updateShape`: whether the 3 by 3 by 3 around it is all water and at least 16 of the frame
    /// around that are conduit frame blocks, which it keeps.
    fn update_shape(
        level: &dyn BlockGetter,
        world_position: &BlockPos,
        effect_blocks: &mut Vec<BlockPos>,
    ) -> bool {
        effect_blocks.clear();

        for ox in -1..=1 {
            for oy in -1..=1 {
                for oz in -1..=1 {
                    let test_pos = world_position.offset_xyz(ox, oy, oz);
                    // `isWaterAt`.
                    if !level.get_fluid_state(&test_pos).is_tag(FluidTags::WATER) {
                        return false;
                    }
                }
            }
        }

        for ox in -2_i32..=2 {
            for oy in -2_i32..=2 {
                for oz in -2_i32..=2 {
                    let ax = ox.abs();
                    let ay = oy.abs();
                    let az = oz.abs();
                    if (ax > 1 || ay > 1 || az > 1)
                        && (ox == 0 && (ay == 2 || az == 2)
                            || oy == 0 && (ax == 2 || az == 2)
                            || oz == 0 && (ax == 2 || ay == 2))
                    {
                        let test_pos = world_position.offset_xyz(ox, oy, oz);
                        let test_block = level.get_block_state(&test_pos);
                        if test_block.is_tag(BlockTags::CONDUIT_EFFECT_BLOCK) {
                            effect_blocks.push(test_pos);
                        }
                    }
                }
            }
        }

        effect_blocks.len() >= ConduitBlockEntity::MIN_ACTIVE_SIZE
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn is_hunting(&self) -> bool {
        self.is_hunting
    }

    pub fn get_active_rotation(&self, a: f32) -> f32 {
        (self.active_rotation + a) * -0.0375
    }

    pub fn active_rotation(&self) -> f32 {
        self.active_rotation
    }

    pub fn effect_blocks(&self) -> &[BlockPos] {
        &self.effect_blocks
    }
}
