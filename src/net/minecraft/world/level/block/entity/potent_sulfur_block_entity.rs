//! net/minecraft/world/level/block/entity/PotentSulfurBlockEntity.java
//!
//! When a potent sulfur last erupted, which its block's block event sets
//! (`PotentSulfurBlock.triggerEvent`). Not ported yet: its tickers (the client's gas and geyser
//! plumes, the server's countdowns and launches), so a level ticks none for it.

#[derive(Clone, Debug, PartialEq)]
pub struct PotentSulfurBlockEntity {
    pub eruption_tick: i64,
}

impl PotentSulfurBlockEntity {
    pub fn new() -> PotentSulfurBlockEntity {
        PotentSulfurBlockEntity { eruption_tick: -1 }
    }
}

impl Default for PotentSulfurBlockEntity {
    fn default() -> PotentSulfurBlockEntity {
        PotentSulfurBlockEntity::new()
    }
}
