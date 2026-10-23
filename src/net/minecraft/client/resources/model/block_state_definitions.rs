//! net/minecraft/client/resources/model/BlockStateDefinitions.java
//!
//! Not ported yet: the item frames' fake definitions (`ITEM_FRAME_FAKE_DEFINITION` and the glow
//! one, for `item_frame.json` and `glow_item_frame.json`), which come with entity rendering.

use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::world::level::block::blocks;
use crate::net::minecraft::world::level::block::state::state_definition::StateDefinition;

pub struct BlockStateDefinitions;

impl BlockStateDefinitions {
    /// A block's definition by the block's registered name.
    pub fn definition_location_to_block_state_mapper()
    -> impl Fn(&Identifier) -> Option<&'static StateDefinition> {
        |id| {
            blocks::registry()
                .get_value(id)
                .map(|block| block.get_state_definition())
        }
    }
}
