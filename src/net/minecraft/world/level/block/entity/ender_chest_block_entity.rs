//! net/minecraft/world/level/block/entity/EnderChestBlockEntity.java
//!
//! An ender chest's lid, as the client moves it: like a chest's (`ChestBlockEntity`), the block
//! event saying whether anyone has it open and the client's ticker moving the lid. Not ported yet:
//! the openers counter and its sounds, the server's.

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntityData;
use crate::net::minecraft::world::level::block::entity::chest_lid_controller::ChestLidController;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct EnderChestBlockEntity {
    chest_lid_controller: ChestLidController,
}

impl EnderChestBlockEntity {
    pub fn new() -> EnderChestBlockEntity {
        EnderChestBlockEntity::default()
    }

    /// `lidAnimateTick`, the client's ticker.
    pub fn lid_animate_tick(level: &mut LevelTicking<'_>, pos: &BlockPos, _state: BlockState) {
        let mut block_entity = level.block_entity(pos);
        if let BlockEntityData::EnderChest(entity) = &mut block_entity.data {
            entity.chest_lid_controller.tick_lid();
        }
    }

    pub fn trigger_event(&mut self, b0: i32, b1: i32) -> bool {
        if b0 == 1 {
            self.chest_lid_controller.should_be_open(b1 > 0);
            true
        } else {
            false
        }
    }

    pub fn get_open_ness(&self, a: f32) -> f32 {
        self.chest_lid_controller.get_openness(a)
    }

    pub fn chest_lid_controller(&self) -> &ChestLidController {
        &self.chest_lid_controller
    }
}
