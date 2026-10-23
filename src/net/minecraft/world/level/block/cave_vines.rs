//! net/minecraft/world/level/block/CaveVines.java
//!
//! Only `BERRIES`, `emission` and `SHAPE`, which decide cave vines' states, light and shape. Not
//! ported yet: `use` and `hasGlowBerries`, the vines' behaviour.

use std::sync::LazyLock;

use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(14.0, 0.0, 16.0));

/// What both parts of cave vines share: their berries.
pub struct CaveVines;

impl CaveVines {
    pub const BERRIES: &'static BooleanProperty = BlockStateProperties::BERRIES;
    pub const SHAPE: &'static LazyLock<VoxelShape> = &SHAPE;

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
