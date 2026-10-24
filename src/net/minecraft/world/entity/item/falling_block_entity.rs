//! net/minecraft/world/entity/item/FallingBlockEntity.java
//!
//! A falling block on the client: the block it is (the add packet's data), and its tick, which
//! moves it on both sides (`MoveSimulationType.SERVER_AND_CLIENT`): gravity, `move`, the air's
//! drag. Its interpolation handler is `Entity`'s, none, so the server's moves put it where they
//! say.
//!
//! Not ported yet: landing, breaking and hurting what it lands on, which only the server does
//! (`ServerLevel`'s branch of `tick`); `applyEffectsFromBlocks` (see `Entity`), the portals
//! (`handlePortal`), the start position (`DATA_START_POS`, synced data only rendering reads),
//! `requiresPrecisePosition` (which only the server's tracker reads), `blockData` and the block
//! entity it carries.

use crate::net::minecraft::network::protocol::game::clientbound_add_entity_packet::ClientboundAddEntityPacket;
use crate::net::minecraft::world::entity::entity::{self, Entity, EntityClass};
use crate::net::minecraft::world::entity::entity_dimensions::EntityDimensions;
use crate::net::minecraft::world::entity::entity_types::EntityTypes;
use crate::net::minecraft::world::entity::move_simulation_type::MoveSimulationType;
use crate::net::minecraft::world::entity::mover_type::MoverType;
use crate::net::minecraft::world::entity::pose::Pose;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::level::Level;

/// What a falling block has besides an entity's.
#[derive(Clone, Debug)]
pub struct FallingBlockEntity {
    entity: Entity,
    block_state: BlockState,
    pub time: i32,
    /// Whether it was removed (`discard()`), which the level's tick list would drop it for; on
    /// the client only a falling block of air removes itself.
    removed: bool,
}

impl FallingBlockEntity {
    /// `FallingBlockEntity(EntityType, Level)`, falling sand until the add packet says.
    pub fn new() -> FallingBlockEntity {
        FallingBlockEntity {
            entity: Entity::new(
                EntityTypes::FALLING_BLOCK,
                EntityTypes::FALLING_BLOCK.get_dimensions(),
            ),
            block_state: Blocks::SAND.default_block_state(),
            time: 0,
            removed: false,
        }
    }

    pub fn get_block_state(&self) -> BlockState {
        self.block_state
    }

    pub fn is_removed(&self) -> bool {
        self.removed
    }

    /// `recreateFromPacket`: `Entity`'s, then the block from the packet's data, and the position
    /// set again (`blocksBuilding` and the start position aren't ported).
    pub fn recreate_from_packet(&mut self, packet: &ClientboundAddEntityPacket) {
        self.entity.recreate_from_packet(packet);
        self.block_state = Block::state_by_id(packet.get_data());
        self.entity
            .set_pos(packet.get_x(), packet.get_y(), packet.get_z());
    }

    /// `tick()` on the client: a falling block of air is gone; anything else falls, moves, and
    /// keeps a fiftieth less of its movement. The landing is the server's.
    pub fn tick(&mut self, level: &dyn Level) {
        if self.block_state.is_air() {
            self.removed = true;
        } else {
            self.time += 1;
            entity::apply_gravity(self);
            let delta = self.entity.get_delta_movement();
            self.r#move(level, MoverType::SELF, delta);
            let air_drag = f64::from(self.get_air_drag());
            let delta = self.entity.get_delta_movement();
            self.entity.set_delta_movement(delta.scale(air_drag));
        }
    }
}

impl Default for FallingBlockEntity {
    fn default() -> FallingBlockEntity {
        FallingBlockEntity::new()
    }
}

impl EntityClass for FallingBlockEntity {
    fn entity(&self) -> &Entity {
        &self.entity
    }

    fn entity_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }

    fn get_dimensions(&self, _pose: Pose) -> EntityDimensions {
        EntityTypes::FALLING_BLOCK.get_dimensions()
    }

    fn get_default_gravity(&self) -> f64 {
        0.04
    }

    fn get_move_simulation_type(&self) -> MoveSimulationType {
        MoveSimulationType::SERVER_AND_CLIENT
    }
}
