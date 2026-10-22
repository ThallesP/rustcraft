//! net/minecraft/world/level/block/entity/trialspawner/TrialSpawnerState.java
//!
//! Only `lightLevel` of what a constant holds, which a trial spawner glows with. Not ported yet:
//! `spinningMobSpeed`, `hasSpinningMob`, `isCapableOfSpawning`, `emitParticles`.

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// What a trial spawner is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TrialSpawnerState {
    INACTIVE,
    WAITING_FOR_PLAYERS,
    ACTIVE,
    WAITING_FOR_REWARD_EJECTION,
    EJECTING_REWARD,
    COOLDOWN,
}

impl TrialSpawnerState {
    pub const VALUES: [TrialSpawnerState; 6] = [
        TrialSpawnerState::INACTIVE,
        TrialSpawnerState::WAITING_FOR_PLAYERS,
        TrialSpawnerState::ACTIVE,
        TrialSpawnerState::WAITING_FOR_REWARD_EJECTION,
        TrialSpawnerState::EJECTING_REWARD,
        TrialSpawnerState::COOLDOWN,
    ];

    pub fn values() -> [TrialSpawnerState; 6] {
        TrialSpawnerState::VALUES
    }

    pub fn light_level(&self) -> i32 {
        match self {
            TrialSpawnerState::INACTIVE => 0,
            TrialSpawnerState::WAITING_FOR_PLAYERS => 4,
            TrialSpawnerState::ACTIVE => 8,
            TrialSpawnerState::WAITING_FOR_REWARD_EJECTION => 8,
            TrialSpawnerState::EJECTING_REWARD => 8,
            TrialSpawnerState::COOLDOWN => 0,
        }
    }
}

impl Enum for TrialSpawnerState {
    const ENUM_CONSTANTS: &'static [TrialSpawnerState] = &TrialSpawnerState::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for TrialSpawnerState {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            TrialSpawnerState::INACTIVE => "inactive",
            TrialSpawnerState::WAITING_FOR_PLAYERS => "waiting_for_players",
            TrialSpawnerState::ACTIVE => "active",
            TrialSpawnerState::WAITING_FOR_REWARD_EJECTION => "waiting_for_reward_ejection",
            TrialSpawnerState::EJECTING_REWARD => "ejecting_reward",
            TrialSpawnerState::COOLDOWN => "cooldown",
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for TrialSpawnerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PropertyEnum for TrialSpawnerState {
    const CLASS: &'static str =
        "net.minecraft.world.level.block.entity.trialspawner.TrialSpawnerState";
}
