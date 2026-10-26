//! net/minecraft/world/level/block/entity/DecoratedPotBlockEntity.java
//!
//! A decorated pot's wobble, which the server's block event starts (`triggerEvent`) and the
//! renderer shows. Not ported yet: its sherds (`PotDecorations`), its item and loot table, which
//! need item stacks.

use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;

/// `DecoratedPotBlockEntity.WobbleStyle`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum WobbleStyle {
    POSITIVE,
    NEGATIVE,
}

impl WobbleStyle {
    pub const VALUES: [WobbleStyle; 2] = [WobbleStyle::POSITIVE, WobbleStyle::NEGATIVE];

    pub fn duration(self) -> i32 {
        match self {
            WobbleStyle::POSITIVE => 7,
            WobbleStyle::NEGATIVE => 10,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct DecoratedPotBlockEntity {
    pub wobble_started_at_tick: i64,
    pub last_wobble_style: Option<WobbleStyle>,
}

impl DecoratedPotBlockEntity {
    pub fn new() -> DecoratedPotBlockEntity {
        DecoratedPotBlockEntity::default()
    }

    /// `triggerEvent`: a wobble of the style `data` names, from now.
    pub fn trigger_event(&mut self, level: &ClientLevel, event: i32, data: i32) -> bool {
        if event == 1 && data >= 0 && (data as usize) < WobbleStyle::VALUES.len() {
            self.wobble_started_at_tick = level.get_game_time();
            self.last_wobble_style = Some(WobbleStyle::VALUES[data as usize]);
            true
        } else {
            false
        }
    }
}
