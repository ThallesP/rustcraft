//! net/minecraft/world/entity/player/Input.java
//!
//! Not ported yet: `STREAM_CODEC`, which comes with `ServerboundPlayerInputPacket`.

/// The movement keys a player holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Input {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub shift: bool,
    pub sprint: bool,
}

impl Input {
    pub const EMPTY: Input = Input {
        forward: false,
        backward: false,
        left: false,
        right: false,
        jump: false,
        shift: false,
        sprint: false,
    };
}
