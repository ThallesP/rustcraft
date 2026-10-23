//! net/minecraft/world/entity/MobCategory.java
//!
//! Not ported yet: `CODEC`, which waits for DataFixerUpper's codecs. `StringRepresentable`'s
//! `getSerializedName` is its own, like `HumanoidArm`'s.

use std::fmt;

/// What kind of mob an entity type is, which decides how many spawn and when they despawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MobCategory {
    MONSTER,
    CREATURE,
    AMBIENT,
    AXOLOTLS,
    UNDERGROUND_WATER_CREATURE,
    WATER_CREATURE,
    WATER_AMBIENT,
    MISC,
}

// Each constant's name, debug abbreviation, max, isFriendly, isPersistent and despawn distance, in
// declaration order.
const DATA: [(&str, &str, i32, bool, bool, i32); 8] = [
    ("monster", "MO", 70, false, false, 128),
    ("creature", "C", 10, true, true, 128),
    ("ambient", "AM", 15, true, false, 128),
    ("axolotls", "AX", 5, true, false, 128),
    ("underground_water_creature", "UWC", 5, true, false, 128),
    ("water_creature", "WC", 5, true, false, 128),
    ("water_ambient", "WA", 20, true, false, 64),
    ("misc", "MI", -1, true, true, 128),
];

impl MobCategory {
    pub const VALUES: [MobCategory; 8] = [
        MobCategory::MONSTER,
        MobCategory::CREATURE,
        MobCategory::AMBIENT,
        MobCategory::AXOLOTLS,
        MobCategory::UNDERGROUND_WATER_CREATURE,
        MobCategory::WATER_CREATURE,
        MobCategory::WATER_AMBIENT,
        MobCategory::MISC,
    ];

    pub fn values() -> [MobCategory; 8] {
        MobCategory::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> MobCategory {
        MobCategory::VALUES
            .into_iter()
            .find(|category| format!("{category:?}") == name)
            .unwrap_or_else(|| panic!("IllegalArgumentException: no MobCategory constant {name}"))
    }

    pub fn get_name(&self) -> &'static str {
        DATA[*self as usize].0
    }

    pub fn get_debug_abbreviation(&self) -> &'static str {
        DATA[*self as usize].1
    }

    pub fn get_serialized_name(&self) -> &'static str {
        DATA[*self as usize].0
    }

    pub fn get_max_instances_per_chunk(&self) -> i32 {
        DATA[*self as usize].2
    }

    pub fn is_friendly(&self) -> bool {
        DATA[*self as usize].3
    }

    pub fn is_persistent(&self) -> bool {
        DATA[*self as usize].4
    }

    pub fn get_despawn_distance(&self) -> i32 {
        DATA[*self as usize].5
    }

    pub fn get_no_despawn_distance(&self) -> i32 {
        32
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for MobCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::MobCategory;
    use crate::golden::{Golden, encode_names};

    encode_names!(MobCategory {
        MONSTER,
        CREATURE,
        AMBIENT,
        AXOLOTLS,
        UNDERGROUND_WATER_CREATURE,
        WATER_CREATURE,
        WATER_AMBIENT,
        MISC,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.MobCategory");
        golden.field("values", MobCategory::values().to_vec());
        for category in MobCategory::VALUES {
            golden.field(
                &category.to_string(),
                (
                    category as i32,
                    category.get_name().to_owned(),
                    category.get_debug_abbreviation().to_owned(),
                    category.get_serialized_name().to_owned(),
                    category.get_max_instances_per_chunk(),
                    category.is_friendly(),
                    category.is_persistent(),
                    category.get_despawn_distance(),
                    category.get_no_despawn_distance(),
                ),
            );
            assert_eq!(MobCategory::value_of(&category.to_string()), category);
        }
        golden.finish();
    }
}
