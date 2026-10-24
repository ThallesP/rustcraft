//! net/minecraft/world/entity/item/ItemEntity.java
//!
//! An item on the ground, on the client: its tick, which moves it on both sides
//! (`MoveSimulationType.SERVER_AND_CLIENT`): `Entity`'s tick, then floating in water and lava or
//! falling, `move` with the air's drag and the block's friction under it, a bounce off the
//! ground, and its age. Its interpolation handler is `Entity`'s, none, so the server's moves put
//! it where they say.
//!
//! Not ported yet: its item (`DATA_ITEM`, an `ItemStack`, whose serializer is the net track's
//! data components), so it's taken as not empty: an empty one discards itself (on the client
//! too), and the server discards an item entity whose stack is empty before any player tracks
//! it. What only the server does: merging with its neighbours, being picked up, despawning,
//! moving out of blocks (`moveTowardsClosestSpace`). `applyEffectsFromBlocks` (see `Entity`), the
//! bobbing and spin that only rendering reads (`bobOffs`, `getSpin`).

// Java's negated comparisons are kept as they are: they differ from the plain ones for NaN.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use crate::net::minecraft::tags::fluid_tags::FluidTags;
use crate::net::minecraft::world::entity::entity::{self, Entity, EntityClass};
use crate::net::minecraft::world::entity::entity_dimensions::EntityDimensions;
use crate::net::minecraft::world::entity::entity_types::EntityTypes;
use crate::net::minecraft::world::entity::move_simulation_type::MoveSimulationType;
use crate::net::minecraft::world::entity::mover_type::MoverType;
use crate::net::minecraft::world::entity::pose::Pose;
use crate::net::minecraft::world::level::level::Level;

/// What an item on the ground has besides an entity's.
#[derive(Clone, Debug)]
pub struct ItemEntity {
    entity: Entity,
    age: i32,
    pickup_delay: i32,
}

impl ItemEntity {
    /// `INFINITE_PICKUP_DELAY`.
    const INFINITE_PICKUP_DELAY: i32 = 32767;
    /// `INFINITE_LIFETIME`.
    const INFINITE_LIFETIME: i32 = -32768;

    /// `ItemEntity(EntityType, Level)`. Java turns it to a random rotation and bob, which the add
    /// packet's rotation replaces and only rendering reads.
    pub fn new() -> ItemEntity {
        ItemEntity {
            entity: Entity::new(EntityTypes::ITEM, EntityTypes::ITEM.get_dimensions()),
            age: 0,
            pickup_delay: 0,
        }
    }

    pub fn get_age(&self) -> i32 {
        self.age
    }

    /// `tick()` on the client.
    pub fn tick(&mut self, level: &dyn Level) {
        // `getItem().isEmpty()`: never, until item stacks are ported (above).
        entity::base_tick(self, level);
        if self.pickup_delay > 0 && self.pickup_delay != ItemEntity::INFINITE_PICKUP_DELAY {
            self.pickup_delay -= 1;
        }

        let position = self.entity.position();
        self.entity.set_old_position(position);
        if self.entity.is_in_water()
            && self.entity.get_fluid_height(FluidTags::WATER) > f64::from(0.1_f32)
        {
            self.set_underwater_movement();
        } else if self.entity.is_in_lava()
            && self.entity.get_fluid_height(FluidTags::LAVA) > f64::from(0.1_f32)
        {
            self.set_under_lava_movement();
        } else {
            entity::apply_gravity(self);
        }

        // On the client, its physics are always on.
        self.entity.no_physics = false;
        if self.entity.on_ground()
            && !(self.entity.get_delta_movement().horizontal_distance_sqr() > f64::from(1.0E-5_f32))
            && (self.entity.tick_count.wrapping_add(self.entity.get_id())) % 4 != 0
        {
            // `applyEffectsFromBlocksForLastMovements()` (see `Entity`).
        } else {
            let delta = self.entity.get_delta_movement();
            self.r#move(level, MoverType::SELF, delta);
            let air_drag = self.get_air_drag();
            let mut ground_friction = air_drag;
            if self.entity.on_ground() {
                ground_friction *= level
                    .get_block_state(&self.get_block_pos_below_that_affects_my_movement(level))
                    .get_block()
                    .get_friction();
            }

            let delta = self.entity.get_delta_movement().multiply_xyz(
                f64::from(ground_friction),
                f64::from(air_drag),
                f64::from(ground_friction),
            );
            self.entity.set_delta_movement(delta);
            if self.entity.on_ground() {
                let movement = self.entity.get_delta_movement();
                if movement.y < 0.0 {
                    self.entity
                        .set_delta_movement(movement.multiply_xyz(1.0, -0.5, 1.0));
                }
            }
        }

        // Merging with its neighbours is the server's.
        if self.age != ItemEntity::INFINITE_LIFETIME {
            self.age += 1;
        }

        // `needsSync`, which the fluids' update sets, is the server's.
        self.update_fluid_interaction(level);
    }

    /// `getBlockPosBelowThatAffectsMyMovement()`: just under it.
    fn get_block_pos_below_that_affects_my_movement(
        &self,
        level: &dyn Level,
    ) -> crate::net::minecraft::core::block_pos::BlockPos {
        self.entity.get_on_pos_offset(level, 0.999999)
    }

    fn set_underwater_movement(&mut self) {
        self.set_fluid_movement(f64::from(0.99_f32));
    }

    fn set_under_lava_movement(&mut self) {
        self.set_fluid_movement(f64::from(0.95_f32));
    }

    fn set_fluid_movement(&mut self, multiplier: f64) {
        let movement = self.entity.get_delta_movement();
        let rise = if movement.y < f64::from(0.06_f32) {
            5.0E-4_f32
        } else {
            0.0
        };
        self.entity.set_delta_movement_xyz(
            movement.x * multiplier,
            movement.y + f64::from(rise),
            movement.z * multiplier,
        );
    }
}

impl Default for ItemEntity {
    fn default() -> ItemEntity {
        ItemEntity::new()
    }
}

impl EntityClass for ItemEntity {
    fn entity(&self) -> &Entity {
        &self.entity
    }

    fn entity_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }

    fn get_dimensions(&self, _pose: Pose) -> EntityDimensions {
        EntityTypes::ITEM.get_dimensions()
    }

    fn get_default_gravity(&self) -> f64 {
        0.04
    }

    fn get_move_simulation_type(&self) -> MoveSimulationType {
        MoveSimulationType::SERVER_AND_CLIENT
    }
}
