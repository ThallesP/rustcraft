//! net/minecraft/world/level/block/CaveVines.java
//!
//! Only `BERRIES` and `emission`, which decide cave vines' states and light. Not ported yet:
//! `use` and `hasGlowBerries`, the vines' behaviour.

use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

/// What both parts of cave vines share: their berries.
pub struct CaveVines;

impl CaveVines {
    pub const BERRIES: &'static BooleanProperty = BlockStateProperties::BERRIES;

    /// The light of vines with berries.
    pub fn emission(light_emission: i32) -> impl Fn(BlockState) -> i32 + Send + Sync + 'static {
        move |state| {
            if state.get_value(BlockStateProperties::BERRIES) {
                light_emission
            } else {
                0
            }
        }
    }
}
