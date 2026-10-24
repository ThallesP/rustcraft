//! net/minecraft/sounds/SoundEvents.java
//!
//! Only the sounds blocks' `animateTick`s and the fluids' play so far, each a variable range event
//! (`register`, `registerForHolder`; a holder's value is the event itself). Not ported yet: the
//! others, and their registration in `BuiltInRegistries.SOUND_EVENT`.

use crate::net::minecraft::sounds::sound_event::SoundEvent;

pub struct SoundEvents;

impl SoundEvents {
    pub const BLASTFURNACE_FIRE_CRACKLE: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.blastfurnace.fire_crackle");
    pub const BUBBLE_COLUMN_UPWARDS_AMBIENT: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.bubble_column.upwards_ambient");
    pub const BUBBLE_COLUMN_WHIRLPOOL_AMBIENT: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.bubble_column.whirlpool_ambient");
    pub const CAMPFIRE_CRACKLE: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.campfire.crackle");
    pub const CANDLE_AMBIENT: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.candle.ambient");
    pub const CREAKING_HEART_IDLE: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.creaking_heart.idle");
    pub const DEAD_BUSH_IDLE: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.deadbush.idle");
    pub const DRIED_GHAST_AMBIENT: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.dried_ghast.ambient");
    pub const DRIED_GHAST_AMBIENT_WATER: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.dried_ghast.ambient_water");
    pub const DRY_GRASS: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.dry_grass.ambient");
    pub const EYEBLOSSOM_IDLE: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.eyeblossom.idle");
    pub const FIRE_AMBIENT: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.fire.ambient");
    pub const FIREFLY_BUSH_IDLE: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.firefly_bush.idle");
    pub const FURNACE_FIRE_CRACKLE: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.furnace.fire_crackle");
    pub const LAVA_AMBIENT: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.lava.ambient");
    pub const LAVA_POP: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.lava.pop");
    pub const NOXIOUS_GAS: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.potent_sulfur.noxious_gas");
    pub const PALE_HANGING_MOSS_IDLE: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.pale_hanging_moss.idle");
    pub const POPLAR_LEAVES_AMBIENT: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.poplar_leaves.ambient");
    pub const PORTAL_AMBIENT: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.portal.ambient");
    pub const RESPAWN_ANCHOR_AMBIENT: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.respawn_anchor.ambient");
    pub const SAND_IDLE: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.sand.idle");
    pub const SMOKER_SMOKE: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.smoker.smoke");
    pub const WATER_AMBIENT: SoundEvent =
        SoundEvent::create_variable_range_event("minecraft:block.water.ambient");
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::SoundEvents;
    use crate::golden::{Encode, Golden};
    use crate::net::minecraft::sounds::sound_event::SoundEvent;

    /// The sounds ported, each by its field's name: its location and fixed range.
    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.sounds.SoundEvents");
        let ported: [(&str, SoundEvent); 24] = [
            (
                "BLASTFURNACE_FIRE_CRACKLE",
                SoundEvents::BLASTFURNACE_FIRE_CRACKLE,
            ),
            (
                "BUBBLE_COLUMN_UPWARDS_AMBIENT",
                SoundEvents::BUBBLE_COLUMN_UPWARDS_AMBIENT,
            ),
            (
                "BUBBLE_COLUMN_WHIRLPOOL_AMBIENT",
                SoundEvents::BUBBLE_COLUMN_WHIRLPOOL_AMBIENT,
            ),
            ("CAMPFIRE_CRACKLE", SoundEvents::CAMPFIRE_CRACKLE),
            ("CANDLE_AMBIENT", SoundEvents::CANDLE_AMBIENT),
            ("CREAKING_HEART_IDLE", SoundEvents::CREAKING_HEART_IDLE),
            ("DEAD_BUSH_IDLE", SoundEvents::DEAD_BUSH_IDLE),
            ("DRIED_GHAST_AMBIENT", SoundEvents::DRIED_GHAST_AMBIENT),
            (
                "DRIED_GHAST_AMBIENT_WATER",
                SoundEvents::DRIED_GHAST_AMBIENT_WATER,
            ),
            ("DRY_GRASS", SoundEvents::DRY_GRASS),
            ("EYEBLOSSOM_IDLE", SoundEvents::EYEBLOSSOM_IDLE),
            ("FIRE_AMBIENT", SoundEvents::FIRE_AMBIENT),
            ("FIREFLY_BUSH_IDLE", SoundEvents::FIREFLY_BUSH_IDLE),
            ("FURNACE_FIRE_CRACKLE", SoundEvents::FURNACE_FIRE_CRACKLE),
            ("LAVA_AMBIENT", SoundEvents::LAVA_AMBIENT),
            ("LAVA_POP", SoundEvents::LAVA_POP),
            ("NOXIOUS_GAS", SoundEvents::NOXIOUS_GAS),
            (
                "PALE_HANGING_MOSS_IDLE",
                SoundEvents::PALE_HANGING_MOSS_IDLE,
            ),
            ("POPLAR_LEAVES_AMBIENT", SoundEvents::POPLAR_LEAVES_AMBIENT),
            ("PORTAL_AMBIENT", SoundEvents::PORTAL_AMBIENT),
            (
                "RESPAWN_ANCHOR_AMBIENT",
                SoundEvents::RESPAWN_ANCHOR_AMBIENT,
            ),
            ("SAND_IDLE", SoundEvents::SAND_IDLE),
            ("SMOKER_SMOKE", SoundEvents::SMOKER_SMOKE),
            ("WATER_AMBIENT", SoundEvents::WATER_AMBIENT),
        ];
        for (name, sound) in ported {
            golden.field(
                name,
                Value::from(vec![
                    Value::from(sound.location().to_string()),
                    sound
                        .fixed_range()
                        .map_or(Value::Null, |range| range.encode()),
                ]),
            );
        }
        golden.pending_rest(
            "only the sounds blocks' and fluids' animateTick play are ported; the others come with what plays them",
        );
        golden.finish();
    }
}
