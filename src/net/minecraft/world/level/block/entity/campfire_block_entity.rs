//! net/minecraft/world/level/block/entity/CampfireBlockEntity.java
//!
//! A campfire's cooking times, as `loadAdditional` reads them, and the smoke the client's ticker
//! sends up while it's lit (`particleTick`). Not ported yet: the items cooking on it (`Items`,
//! item stacks, which the renderer shows, and whose smoke the ticker sends up too: until they
//! come, a campfire holds none), the server's tickers (cooking, cooling down) and placing food.

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::campfire_block::CampfireBlock;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct CampfireBlockEntity {
    cooking_progress: [i32; 4],
    cooking_time: [i32; 4],
}

impl CampfireBlockEntity {
    pub fn new() -> CampfireBlockEntity {
        CampfireBlockEntity::default()
    }

    /// Each array copied in up to its length (Java copies `cookingTime.length` of either, both
    /// 4), or cleared where the tag has no int array.
    pub fn load_additional(&mut self, input: &TagValueInput<'_>) {
        match input.get_int_array("CookingTimes") {
            Some(cooking_times) => {
                let length = self.cooking_time.len().min(cooking_times.len());
                self.cooking_progress[..length].copy_from_slice(&cooking_times[..length]);
            }
            None => self.cooking_progress = [0; 4],
        }
        match input.get_int_array("CookingTotalTimes") {
            Some(cooking_times) => {
                let length = self.cooking_time.len().min(cooking_times.len());
                self.cooking_time[..length].copy_from_slice(&cooking_times[..length]);
            }
            None => self.cooking_time = [0; 4],
        }
    }

    /// `particleTick`, the client's ticker for a lit campfire: now and then two or three puffs
    /// of smoke, from the level's random.
    pub fn particle_tick(level: &mut LevelTicking<'_>, pos: &BlockPos, state: BlockState) {
        let level = level.effects_level();
        if level.get_random().next_float() < 0.11 {
            // Java's loop draws its bound again each time round.
            let mut i = 0;
            while i < level.get_random().next_int_bounded(2) + 2 {
                CampfireBlock::make_particles(
                    &level,
                    pos,
                    state.get_value(CampfireBlock::SIGNAL_FIRE),
                    false,
                );
                i += 1;
            }
        }

        // Each item cooking on it smokes; it holds none (above).
    }

    pub fn cooking_progress(&self) -> &[i32; 4] {
        &self.cooking_progress
    }

    pub fn cooking_time(&self) -> &[i32; 4] {
        &self.cooking_time
    }
}
