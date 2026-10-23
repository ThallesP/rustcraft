//! net/minecraft/world/level/block/entity/CampfireBlockEntity.java
//!
//! A campfire's cooking times, as `loadAdditional` reads them. Not ported yet: the items cooking
//! on it (`Items`, item stacks, which the renderer shows), the tickers (cooking on the server, the
//! smoke the client's ticker sends up, `particleTick`) and placing food.

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

    pub fn cooking_progress(&self) -> &[i32; 4] {
        &self.cooking_progress
    }

    pub fn cooking_time(&self) -> &[i32; 4] {
        &self.cooking_time
    }
}
