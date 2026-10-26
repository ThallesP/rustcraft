//! net/minecraft/world/level/block/entity/TheEndGatewayBlockEntity.java
//!
//! An end gateway's beam, as the client sees it: its age (loaded, then a tick at a time from the
//! client's ticker, `beamAnimationTick`), which makes the beam while it's young, and the cooldown
//! after something goes through, which the server's block event starts (`triggerEvent`). Not
//! ported yet: its exit (`exit_portal`, `ExactTeleport`), the teleporting and the server's
//! `portalTick`, and `getParticleAmount` (the end portal's faces, which `EndGatewayBlock`
//! reads itself).

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntityData;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TheEndGatewayBlockEntity {
    age: i64,
    teleport_cooldown: i32,
}

impl TheEndGatewayBlockEntity {
    pub fn new() -> TheEndGatewayBlockEntity {
        TheEndGatewayBlockEntity::default()
    }

    pub fn load_additional(&mut self, input: &TagValueInput<'_>) {
        self.age = input.get_long_or("Age", 0);
    }

    /// `beamAnimationTick`, the client's ticker.
    pub fn beam_animation_tick(level: &mut LevelTicking<'_>, pos: &BlockPos, _state: BlockState) {
        let mut block_entity = level.block_entity(pos);
        if let BlockEntityData::EndGateway(entity) = &mut block_entity.data {
            entity.age = entity.age.wrapping_add(1);
            if entity.is_cooling_down() {
                entity.teleport_cooldown -= 1;
            }
        }
    }

    pub fn is_spawning(&self) -> bool {
        self.age < 200
    }

    pub fn is_cooling_down(&self) -> bool {
        self.teleport_cooldown > 0
    }

    pub fn get_spawn_percent(&self, a: f32) -> f32 {
        Mth::clamp_f32((self.age as f32 + a) / 200.0, 0.0, 1.0)
    }

    pub fn get_cooldown_percent(&self, a: f32) -> f32 {
        1.0 - Mth::clamp_f32((self.teleport_cooldown as f32 - a) / 40.0, 0.0, 1.0)
    }

    /// `triggerEvent`: something went through, so it cools down for 40 ticks.
    pub fn trigger_event(&mut self, b0: i32, _b1: i32) -> bool {
        if b0 == 1 {
            self.teleport_cooldown = 40;
            true
        } else {
            false
        }
    }

    pub fn age(&self) -> i64 {
        self.age
    }

    pub fn teleport_cooldown(&self) -> i32 {
        self.teleport_cooldown
    }
}
