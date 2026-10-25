//! net/minecraft/world/entity/decoration/ArmorStand.java
//!
//! An armor stand on the client: a living entity that nothing pushes (`isPushable`), that pushes
//! nothing itself (`doPush`), and whose own `pushEntities` only pushes the ridable minecarts it's
//! in, which aren't ported. The server moves it, through the stepped interpolation every living
//! entity has.
//!
//! Not ported yet: its synced flags (small, arms, base plate, marker: `DATA_CLIENT_FLAGS`) and
//! poses, so its size is a standard one's; its equipment, interactions and damage, which are the
//! server's.

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

/// What an armor stand has besides a living entity's.
#[derive(Clone, Debug)]
pub struct ArmorStand {
    living_entity: LivingEntity,
}

impl ArmorStand {
    /// `ArmorStand(EntityType, Level)`.
    pub fn new() -> ArmorStand {
        ArmorStand {
            living_entity: LivingEntity::new(
                EntityTypes::ARMOR_STAND,
                EntityTypes::ARMOR_STAND.get_dimensions(),
                AttributeMap::new(&ArmorStand::create_attributes()),
            ),
        }
    }

    /// `createAttributes()`: a living entity's, with no step height.
    pub fn create_attributes() -> Vec<(&'static Attribute, f64)> {
        living_entity::create_living_attributes(&[(Attributes::STEP_HEIGHT, 0.0)])
    }
}

impl Default for ArmorStand {
    fn default() -> ArmorStand {
        ArmorStand::new()
    }
}

impl EntityClass for ArmorStand {
    fn entity(&self) -> &Entity {
        &self.living_entity
    }

    fn entity_mut(&mut self) -> &mut Entity {
        &mut self.living_entity
    }

    fn get_dimensions(&self, pose: Pose) -> EntityDimensions {
        living_entity::get_dimensions(self, pose, EntityTypes::ARMOR_STAND.get_dimensions())
    }

    fn is_living_entity(&self) -> bool {
        true
    }

    fn is_alive(&self) -> bool {
        living_entity::is_alive(self)
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
}

impl LivingEntityClass for ArmorStand {
    fn living_entity(&self) -> &LivingEntity {
        &self.living_entity
    }

    fn living_entity_mut(&mut self) -> &mut LivingEntity {
        &mut self.living_entity
    }

    /// `pushEntities()`: the ridable minecarts within 0.2 blocks' square distance, which aren't
    /// ported, so nothing.
    fn push_entities(
        &mut self,
        _level: &dyn Level,
        _local_player: Option<&mut (impl LivingEntityClass + Sized)>,
    ) {
    }

    /// `doPush(Entity)`: none.
    fn do_push(&mut self, _level: &dyn Level, _entity: &mut (impl LivingEntityClass + Sized)) {}
}
