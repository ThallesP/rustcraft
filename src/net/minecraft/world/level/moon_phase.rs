//! net/minecraft/world/level/MoonPhase.java
//!
//! `CODEC` is `by_serialized_name`, as `StringRepresentable.fromEnum` reads a name.

/// The moon's eight phases, each a day long.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MoonPhase {
    FULL_MOON,
    WANING_GIBBOUS,
    THIRD_QUARTER,
    WANING_CRESCENT,
    NEW_MOON,
    WAXING_CRESCENT,
    FIRST_QUARTER,
    WAXING_GIBBOUS,
}

impl MoonPhase {
    pub const VALUES: [MoonPhase; 8] = [
        MoonPhase::FULL_MOON,
        MoonPhase::WANING_GIBBOUS,
        MoonPhase::THIRD_QUARTER,
        MoonPhase::WANING_CRESCENT,
        MoonPhase::NEW_MOON,
        MoonPhase::WAXING_CRESCENT,
        MoonPhase::FIRST_QUARTER,
        MoonPhase::WAXING_GIBBOUS,
    ];
    pub const COUNT: i32 = 8;
    pub const PHASE_LENGTH: i32 = 24000;

    pub fn index(&self) -> i32 {
        *self as i32
    }

    pub fn start_tick(&self) -> i32 {
        self.index() * 24000
    }

    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            MoonPhase::FULL_MOON => "full_moon",
            MoonPhase::WANING_GIBBOUS => "waning_gibbous",
            MoonPhase::THIRD_QUARTER => "third_quarter",
            MoonPhase::WANING_CRESCENT => "waning_crescent",
            MoonPhase::NEW_MOON => "new_moon",
            MoonPhase::WAXING_CRESCENT => "waxing_crescent",
            MoonPhase::FIRST_QUARTER => "first_quarter",
            MoonPhase::WAXING_GIBBOUS => "waxing_gibbous",
        }
    }

    pub fn by_serialized_name(name: &str) -> Option<MoonPhase> {
        MoonPhase::VALUES
            .into_iter()
            .find(|phase| phase.get_serialized_name() == name)
    }
}
