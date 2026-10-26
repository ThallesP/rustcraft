//! net/minecraft/world/level/block/entity/ChestBlockEntity.java
//!
//! A chest's lid as the client moves it: the server's block event says whether anyone has it open
//! (`triggerEvent`, `EVENT_SET_OPEN_COUNT`), and the client's ticker moves the lid
//! (`lidAnimateTick`). Its type is the chest's, a copper chest's or the trapped chest's
//! (`TrappedChestBlockEntity`, which adds nothing the client reads). Not ported yet: its items and
//! loot table (`loadAdditional`'s, item stacks), its name, and the openers counter, the menu and the
//! sounds, which are the server's.

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntityData;
use crate::net::minecraft::world::level::block::entity::chest_lid_controller::ChestLidController;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ChestBlockEntity {
    chest_lid_controller: ChestLidController,
}

impl ChestBlockEntity {
    pub const EVENT_SET_OPEN_COUNT: i32 = 1;

    pub fn new() -> ChestBlockEntity {
        ChestBlockEntity::default()
    }

    /// `lidAnimateTick`, the client's ticker.
    pub fn lid_animate_tick(level: &mut LevelTicking<'_>, pos: &BlockPos, _state: BlockState) {
        let mut block_entity = level.block_entity(pos);
        if let BlockEntityData::Chest(entity) = &mut block_entity.data {
            entity.chest_lid_controller.tick_lid();
        }
    }

    /// `triggerEvent`: the lid should be open while anyone has the chest open.
    pub fn trigger_event(&mut self, b0: i32, b1: i32) -> bool {
        if b0 == ChestBlockEntity::EVENT_SET_OPEN_COUNT {
            self.chest_lid_controller.should_be_open(b1 > 0);
            true
        } else {
            false
        }
    }

    /// `getOpenNess(a)`.
    pub fn get_open_ness(&self, a: f32) -> f32 {
        self.chest_lid_controller.get_openness(a)
    }

    pub fn chest_lid_controller(&self) -> &ChestLidController {
        &self.chest_lid_controller
    }
}
