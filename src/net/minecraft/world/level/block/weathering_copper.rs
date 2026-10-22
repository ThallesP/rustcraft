//! net/minecraft/world/level/block/WeatheringCopper.java
//!
//! Only `NEXT_BY_BLOCK` and `getNext`, which decide whether a copper block still weathers, and
//! `WeatherState`'s constants and names. Not ported yet: `PREVIOUS_BY_BLOCK`, `getPrevious`,
//! `getFirst`, `getChanceModifier` and `getAge`, and `WeatherState`'s `BY_ID`, `CODEC` and
//! `STREAM_CODEC`.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::blocks::Blocks;

/// `WeatheringCopper.WeatherState`: how far a copper block has weathered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WeatherState {
    UNAFFECTED,
    EXPOSED,
    WEATHERED,
    OXIDIZED,
}

impl WeatherState {
    pub const VALUES: [WeatherState; 4] = [
        WeatherState::UNAFFECTED,
        WeatherState::EXPOSED,
        WeatherState::WEATHERED,
        WeatherState::OXIDIZED,
    ];

    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            WeatherState::UNAFFECTED => "unaffected",
            WeatherState::EXPOSED => "exposed",
            WeatherState::WEATHERED => "weathered",
            WeatherState::OXIDIZED => "oxidized",
        }
    }
}

// Each unwaxed copper block's next state of weathering, across every collection of them.
static NEXT_BY_BLOCK: LazyLock<HashMap<Block, Block>> = LazyLock::new(|| {
    let mut next_by_block = HashMap::new();
    for collection in [
        Blocks::COPPER_BLOCK,
        Blocks::CUT_COPPER,
        Blocks::CHISELED_COPPER,
        Blocks::CUT_COPPER_SLAB,
        Blocks::CUT_COPPER_STAIRS,
        Blocks::COPPER_DOOR,
        Blocks::COPPER_TRAPDOOR,
        Blocks::COPPER_BARS,
        Blocks::COPPER_GRATE,
        Blocks::COPPER_BULB,
        Blocks::COPPER_LANTERN,
        Blocks::COPPER_CHEST,
        Blocks::COPPER_GOLEM_STATUE,
        Blocks::LIGHTNING_ROD,
        Blocks::COPPER_CHAIN,
    ] {
        collection.weathering().progress_mapping(|&block, &next| {
            next_by_block.insert(block, next);
        });
    }
    next_by_block
});

pub struct WeatheringCopper;

impl WeatheringCopper {
    /// The block this one weathers into, if it still does.
    pub fn get_next(block: Block) -> Option<Block> {
        NEXT_BY_BLOCK.get(&block).copied()
    }

    /// `isRandomlyTicking(BlockState)` of every weathering copper block: while it can still
    /// weather.
    pub fn is_randomly_ticking(
        _block: &crate::net::minecraft::world::level::block::block::BlockData,
        state: crate::net::minecraft::world::level::block::state::block_state::BlockState,
    ) -> bool {
        WeatheringCopper::get_next(state.get_block()).is_some()
    }
}
