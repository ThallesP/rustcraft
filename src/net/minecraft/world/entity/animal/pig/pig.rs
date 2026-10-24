//! net/minecraft/world/entity/animal/pig/Pig.java
//!
//! A pig on the client: a living entity the server moves, through the stepped interpolation every
//! living entity has, which pushes the local player when they touch and is pushed by it
//! (`LivingEntity.pushEntities`, `isPushable`). `Mob`, `PathfinderMob`, `AgeableMob` and `Animal`,
//! the classes between `LivingEntity` and it, add nothing the client's tick moves: their AI and
//! goals run only where the mob is authoritative, the server.
//!
//! Not ported yet: its synced data (the saddle, its boost, its variant, its age), riding it,
//! breeding, and its sounds.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::entity::ai::attributes::attribute::Attribute;
use crate::net::minecraft::world::entity::ai::attributes::attribute_map::AttributeMap;
use crate::net::minecraft::world::entity::ai::attributes::attributes::Attributes;
use crate::net::minecraft::world::entity::entity::{Entity, EntityClass};
use crate::net::minecraft::world::entity::entity_dimensions::EntityDimensions;
use crate::net::minecraft::world::entity::entity_types::EntityTypes;
use crate::net::minecraft::world::entity::living_entity::{self, LivingEntity, LivingEntityClass};
use crate::net::minecraft::world::entity::pose::Pose;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::level::Level;

/// What a pig has besides a living entity's.
#[derive(Clone, Debug)]
pub struct Pig {
    living_entity: LivingEntity,
}

impl Pig {
    /// `Pig(EntityType, Level)`.
    pub fn new() -> Pig {
        Pig {
            living_entity: LivingEntity::new(
                EntityTypes::PIG,
                EntityTypes::PIG.get_dimensions(),
                AttributeMap::new(&Pig::create_attributes()),
            ),
        }
    }

    /// `createAttributes()`: an animal's (a living entity's, with what mobs' AI reads), with 10
    /// health and a speed of 0.25.
    pub fn create_attributes() -> Vec<(&'static Attribute, f64)> {
        living_entity::create_living_attributes(&[
            (Attributes::MAX_HEALTH, 10.0),
            (Attributes::MOVEMENT_SPEED, 0.25),
        ])
    }
}

impl Default for Pig {
    fn default() -> Pig {
        Pig::new()
    }
}

impl EntityClass for Pig {
    fn entity(&self) -> &Entity {
        &self.living_entity
    }

    fn entity_mut(&mut self) -> &mut Entity {
        &mut self.living_entity
    }

    fn get_dimensions(&self, pose: Pose) -> EntityDimensions {
        living_entity::get_dimensions(self, pose, EntityTypes::PIG.get_dimensions())
    }

    fn max_up_step(&self) -> f32 {
        living_entity::max_up_step(self)
    }

    fn get_default_gravity(&self) -> f64 {
        living_entity::get_default_gravity(self)
    }

    fn get_effective_gravity(&self) -> f64 {
        living_entity::get_effective_gravity(self)
    }

    fn get_air_drag(&self) -> f32 {
        living_entity::get_air_drag(self)
    }

    fn get_entity_bounciness(&self) -> f64 {
        living_entity::get_entity_bounciness(self)
    }

    fn check_fall_damage(
        &mut self,
        level: &dyn Level,
        ya: f64,
        on_ground: bool,
        on_state: BlockState,
        pos: BlockPos,
    ) {
        living_entity::check_fall_damage(self, level, ya, on_ground, on_state, pos);
    }

    fn get_block_speed_factor(&mut self, level: &dyn Level) -> f32 {
        living_entity::get_block_speed_factor(self, level)
    }

    fn is_pushable(&mut self, level: &dyn Level) -> bool {
        living_entity::is_pushable(self, level)
    }

    fn push(&mut self, level: &dyn Level, entity: &mut (impl EntityClass + Sized)) {
        living_entity::push(self, level, entity);
    }
}

impl LivingEntityClass for Pig {
    fn living_entity(&self) -> &LivingEntity {
        &self.living_entity
    }

    fn living_entity_mut(&mut self) -> &mut LivingEntity {
        &mut self.living_entity
    }
}
