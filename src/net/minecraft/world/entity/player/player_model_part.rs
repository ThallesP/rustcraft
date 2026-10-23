//! net/minecraft/world/entity/player/PlayerModelPart.java
//!
//! Not ported yet: `CODEC` and the parts' names (`Component`s), which the skin options screen
//! shows.

/// A part of a player's skin that a player can show or hide, each a bit of the mask the client
/// sends (`ClientInformation`) and the player's entity data carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayerModelPart {
    CAPE,
    JACKET,
    LEFT_SLEEVE,
    RIGHT_SLEEVE,
    LEFT_PANTS_LEG,
    RIGHT_PANTS_LEG,
    HAT,
}

impl PlayerModelPart {
    pub const VALUES: [PlayerModelPart; 7] = [
        PlayerModelPart::CAPE,
        PlayerModelPart::JACKET,
        PlayerModelPart::LEFT_SLEEVE,
        PlayerModelPart::RIGHT_SLEEVE,
        PlayerModelPart::LEFT_PANTS_LEG,
        PlayerModelPart::RIGHT_PANTS_LEG,
        PlayerModelPart::HAT,
    ];

    pub fn get_bit(&self) -> i32 {
        *self as i32
    }

    pub fn get_mask(&self) -> i32 {
        1 << self.get_bit()
    }

    pub fn get_id(&self) -> &'static str {
        match self {
            PlayerModelPart::CAPE => "cape",
            PlayerModelPart::JACKET => "jacket",
            PlayerModelPart::LEFT_SLEEVE => "left_sleeve",
            PlayerModelPart::RIGHT_SLEEVE => "right_sleeve",
            PlayerModelPart::LEFT_PANTS_LEG => "left_pants_leg",
            PlayerModelPart::RIGHT_PANTS_LEG => "right_pants_leg",
            PlayerModelPart::HAT => "hat",
        }
    }
}
