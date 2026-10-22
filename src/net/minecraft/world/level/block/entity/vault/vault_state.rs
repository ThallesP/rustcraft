//! net/minecraft/world/level/block/entity/vault/VaultState.java
//!
//! Only `lightLevel`, which a vault glows with: half lit (6) inactive, lit (12) otherwise, its
//! private `LightLevel`'s values. Not ported yet: `tickAndGetNext` and `onTransition`.

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// What a vault is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VaultState {
    INACTIVE,
    ACTIVE,
    UNLOCKING,
    EJECTING,
}

impl VaultState {
    pub const VALUES: [VaultState; 4] = [
        VaultState::INACTIVE,
        VaultState::ACTIVE,
        VaultState::UNLOCKING,
        VaultState::EJECTING,
    ];

    pub fn values() -> [VaultState; 4] {
        VaultState::VALUES
    }

    pub fn light_level(&self) -> i32 {
        match self {
            VaultState::INACTIVE => 6,
            VaultState::ACTIVE | VaultState::UNLOCKING | VaultState::EJECTING => 12,
        }
    }
}

impl Enum for VaultState {
    const ENUM_CONSTANTS: &'static [VaultState] = &VaultState::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for VaultState {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            VaultState::INACTIVE => "inactive",
            VaultState::ACTIVE => "active",
            VaultState::UNLOCKING => "unlocking",
            VaultState::EJECTING => "ejecting",
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for VaultState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PropertyEnum for VaultState {
    const CLASS: &'static str = "net.minecraft.world.level.block.entity.vault.VaultState";
}
