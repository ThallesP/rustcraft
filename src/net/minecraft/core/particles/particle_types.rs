//! net/minecraft/core/particles/ParticleTypes.java
//!
//! The particle types, numbered in the order Java registers them in `BuiltInRegistries.PARTICLE_TYPE`.
//! Their codecs aren't ported yet: no packet the port decodes carries particles.

use crate::net::minecraft::core::particles::particle_type::ParticleType;

pub struct ParticleTypes;

impl ParticleTypes {
    pub const ANGRY_VILLAGER: ParticleType = ParticleType(0);
    pub const BLOCK: ParticleType = ParticleType(1);
    pub const BLOCK_MARKER: ParticleType = ParticleType(2);
    pub const BUBBLE: ParticleType = ParticleType(3);
    pub const SULFUR_BUBBLES: ParticleType = ParticleType(4);
    pub const NOXIOUS_GAS: ParticleType = ParticleType(5);
    pub const NOXIOUS_GAS_CLOUD: ParticleType = ParticleType(6);
    pub const GEYSER: ParticleType = ParticleType(7);
    pub const GEYSER_BASE: ParticleType = ParticleType(8);
    pub const GEYSER_POOF: ParticleType = ParticleType(9);
    pub const GEYSER_PLUME: ParticleType = ParticleType(10);
    pub const CLOUD: ParticleType = ParticleType(11);
    pub const COPPER_FIRE_FLAME: ParticleType = ParticleType(12);
    pub const CRIT: ParticleType = ParticleType(13);
    pub const DAMAGE_INDICATOR: ParticleType = ParticleType(14);
    pub const DRAGON_BREATH: ParticleType = ParticleType(15);
    pub const DRIPPING_LAVA: ParticleType = ParticleType(16);
    pub const FALLING_LAVA: ParticleType = ParticleType(17);
    pub const LANDING_LAVA: ParticleType = ParticleType(18);
    pub const DRIPPING_WATER: ParticleType = ParticleType(19);
    pub const FALLING_WATER: ParticleType = ParticleType(20);
    pub const DUST: ParticleType = ParticleType(21);
    pub const DUST_COLOR_TRANSITION: ParticleType = ParticleType(22);
    pub const EFFECT: ParticleType = ParticleType(23);
    pub const ELDER_GUARDIAN: ParticleType = ParticleType(24);
    pub const ENCHANTED_HIT: ParticleType = ParticleType(25);
    pub const ENCHANT: ParticleType = ParticleType(26);
    pub const END_ROD: ParticleType = ParticleType(27);
    pub const ENTITY_EFFECT: ParticleType = ParticleType(28);
    pub const EXPLOSION_EMITTER: ParticleType = ParticleType(29);
    pub const EXPLOSION: ParticleType = ParticleType(30);
    pub const GUST: ParticleType = ParticleType(31);
    pub const SMALL_GUST: ParticleType = ParticleType(32);
    pub const GUST_EMITTER_LARGE: ParticleType = ParticleType(33);
    pub const GUST_EMITTER_SMALL: ParticleType = ParticleType(34);
    pub const SONIC_BOOM: ParticleType = ParticleType(35);
    pub const FALLING_DUST: ParticleType = ParticleType(36);
    pub const FIREWORK: ParticleType = ParticleType(37);
    pub const FISHING: ParticleType = ParticleType(38);
    pub const FLAME: ParticleType = ParticleType(39);
    pub const INFESTED: ParticleType = ParticleType(40);
    pub const CHERRY_LEAVES: ParticleType = ParticleType(41);
    pub const PALE_OAK_LEAVES: ParticleType = ParticleType(42);
    pub const RED_POPLAR_LEAVES: ParticleType = ParticleType(43);
    pub const ORANGE_POPLAR_LEAVES: ParticleType = ParticleType(44);
    pub const YELLOW_POPLAR_LEAVES: ParticleType = ParticleType(45);
    pub const TINTED_LEAVES: ParticleType = ParticleType(46);
    pub const SCULK_SOUL: ParticleType = ParticleType(47);
    pub const SCULK_CHARGE: ParticleType = ParticleType(48);
    pub const SCULK_CHARGE_POP: ParticleType = ParticleType(49);
    pub const SOUL_FIRE_FLAME: ParticleType = ParticleType(50);
    pub const SOUL: ParticleType = ParticleType(51);
    pub const FLASH: ParticleType = ParticleType(52);
    pub const HAPPY_VILLAGER: ParticleType = ParticleType(53);
    pub const COMPOSTER: ParticleType = ParticleType(54);
    pub const HEART: ParticleType = ParticleType(55);
    pub const INSTANT_EFFECT: ParticleType = ParticleType(56);
    pub const ITEM: ParticleType = ParticleType(57);
    pub const VIBRATION: ParticleType = ParticleType(58);
    pub const TRAIL: ParticleType = ParticleType(59);
    pub const PAUSE_MOB_GROWTH: ParticleType = ParticleType(60);
    pub const RESET_MOB_GROWTH: ParticleType = ParticleType(61);
    pub const ITEM_SLIME: ParticleType = ParticleType(62);
    pub const ITEM_COBWEB: ParticleType = ParticleType(63);
    pub const ITEM_SNOWBALL: ParticleType = ParticleType(64);
    pub const LARGE_SMOKE: ParticleType = ParticleType(65);
    pub const LAVA: ParticleType = ParticleType(66);
    pub const MYCELIUM: ParticleType = ParticleType(67);
    pub const NOTE: ParticleType = ParticleType(68);
    pub const POOF: ParticleType = ParticleType(69);
    pub const PORTAL: ParticleType = ParticleType(70);
    pub const RAIN: ParticleType = ParticleType(71);
    pub const SMOKE: ParticleType = ParticleType(72);
    pub const WHITE_SMOKE: ParticleType = ParticleType(73);
    pub const SNEEZE: ParticleType = ParticleType(74);
    pub const SPIT: ParticleType = ParticleType(75);
    pub const SQUID_INK: ParticleType = ParticleType(76);
    pub const SWEEP_ATTACK: ParticleType = ParticleType(77);
    pub const TOTEM_OF_UNDYING: ParticleType = ParticleType(78);
    pub const UNDERWATER: ParticleType = ParticleType(79);
    pub const SPLASH: ParticleType = ParticleType(80);
    pub const WITCH: ParticleType = ParticleType(81);
    pub const BUBBLE_POP: ParticleType = ParticleType(82);
    pub const CURRENT_DOWN: ParticleType = ParticleType(83);
    pub const BUBBLE_COLUMN_UP: ParticleType = ParticleType(84);
    pub const NAUTILUS: ParticleType = ParticleType(85);
    pub const DOLPHIN: ParticleType = ParticleType(86);
    pub const CAMPFIRE_COSY_SMOKE: ParticleType = ParticleType(87);
    pub const CAMPFIRE_SIGNAL_SMOKE: ParticleType = ParticleType(88);
    pub const DRIPPING_HONEY: ParticleType = ParticleType(89);
    pub const FALLING_HONEY: ParticleType = ParticleType(90);
    pub const LANDING_HONEY: ParticleType = ParticleType(91);
    pub const FALLING_NECTAR: ParticleType = ParticleType(92);
    pub const FALLING_SPORE_BLOSSOM: ParticleType = ParticleType(93);
    pub const ASH: ParticleType = ParticleType(94);
    pub const CRIMSON_SPORE: ParticleType = ParticleType(95);
    pub const WARPED_SPORE: ParticleType = ParticleType(96);
    pub const SPORE_BLOSSOM_AIR: ParticleType = ParticleType(97);
    pub const DRIPPING_OBSIDIAN_TEAR: ParticleType = ParticleType(98);
    pub const FALLING_OBSIDIAN_TEAR: ParticleType = ParticleType(99);
    pub const LANDING_OBSIDIAN_TEAR: ParticleType = ParticleType(100);
    pub const REVERSE_PORTAL: ParticleType = ParticleType(101);
    pub const WHITE_ASH: ParticleType = ParticleType(102);
    pub const SMALL_FLAME: ParticleType = ParticleType(103);
    pub const SNOWFLAKE: ParticleType = ParticleType(104);
    pub const DRIPPING_DRIPSTONE_LAVA: ParticleType = ParticleType(105);
    pub const FALLING_DRIPSTONE_LAVA: ParticleType = ParticleType(106);
    pub const DRIPPING_DRIPSTONE_WATER: ParticleType = ParticleType(107);
    pub const FALLING_DRIPSTONE_WATER: ParticleType = ParticleType(108);
    pub const GLOW_SQUID_INK: ParticleType = ParticleType(109);
    pub const GLOW: ParticleType = ParticleType(110);
    pub const WAX_ON: ParticleType = ParticleType(111);
    pub const WAX_OFF: ParticleType = ParticleType(112);
    pub const ELECTRIC_SPARK: ParticleType = ParticleType(113);
    pub const SCRAPE: ParticleType = ParticleType(114);
    pub const SHRIEK: ParticleType = ParticleType(115);
    pub const EGG_CRACK: ParticleType = ParticleType(116);
    pub const DUST_PLUME: ParticleType = ParticleType(117);
    pub const TRIAL_SPAWNER_DETECTED_PLAYER: ParticleType = ParticleType(118);
    pub const TRIAL_SPAWNER_DETECTED_PLAYER_OMINOUS: ParticleType = ParticleType(119);
    pub const VAULT_CONNECTION: ParticleType = ParticleType(120);
    pub const DUST_PILLAR: ParticleType = ParticleType(121);
    pub const OMINOUS_SPAWNING: ParticleType = ParticleType(122);
    pub const RAID_OMEN: ParticleType = ParticleType(123);
    pub const TRIAL_OMEN: ParticleType = ParticleType(124);
    pub const BLOCK_CRUMBLE: ParticleType = ParticleType(125);
    pub const FIREFLY: ParticleType = ParticleType(126);
    pub const SULFUR_CUBE_GOO: ParticleType = ParticleType(127);
}

/// Each type's name in the registry, by id.
pub(crate) static KEYS: [&str; 128] = [
    "minecraft:angry_villager",
    "minecraft:block",
    "minecraft:block_marker",
    "minecraft:bubble",
    "minecraft:sulfur_bubbles",
    "minecraft:noxious_gas",
    "minecraft:noxious_gas_cloud",
    "minecraft:geyser",
    "minecraft:geyser_base",
    "minecraft:geyser_poof",
    "minecraft:geyser_plume",
    "minecraft:cloud",
    "minecraft:copper_fire_flame",
    "minecraft:crit",
    "minecraft:damage_indicator",
    "minecraft:dragon_breath",
    "minecraft:dripping_lava",
    "minecraft:falling_lava",
    "minecraft:landing_lava",
    "minecraft:dripping_water",
    "minecraft:falling_water",
    "minecraft:dust",
    "minecraft:dust_color_transition",
    "minecraft:effect",
    "minecraft:elder_guardian",
    "minecraft:enchanted_hit",
    "minecraft:enchant",
    "minecraft:end_rod",
    "minecraft:entity_effect",
    "minecraft:explosion_emitter",
    "minecraft:explosion",
    "minecraft:gust",
    "minecraft:small_gust",
    "minecraft:gust_emitter_large",
    "minecraft:gust_emitter_small",
    "minecraft:sonic_boom",
    "minecraft:falling_dust",
    "minecraft:firework",
    "minecraft:fishing",
    "minecraft:flame",
    "minecraft:infested",
    "minecraft:cherry_leaves",
    "minecraft:pale_oak_leaves",
    "minecraft:red_poplar_leaves",
    "minecraft:orange_poplar_leaves",
    "minecraft:yellow_poplar_leaves",
    "minecraft:tinted_leaves",
    "minecraft:sculk_soul",
    "minecraft:sculk_charge",
    "minecraft:sculk_charge_pop",
    "minecraft:soul_fire_flame",
    "minecraft:soul",
    "minecraft:flash",
    "minecraft:happy_villager",
    "minecraft:composter",
    "minecraft:heart",
    "minecraft:instant_effect",
    "minecraft:item",
    "minecraft:vibration",
    "minecraft:trail",
    "minecraft:pause_mob_growth",
    "minecraft:reset_mob_growth",
    "minecraft:item_slime",
    "minecraft:item_cobweb",
    "minecraft:item_snowball",
    "minecraft:large_smoke",
    "minecraft:lava",
    "minecraft:mycelium",
    "minecraft:note",
    "minecraft:poof",
    "minecraft:portal",
    "minecraft:rain",
    "minecraft:smoke",
    "minecraft:white_smoke",
    "minecraft:sneeze",
    "minecraft:spit",
    "minecraft:squid_ink",
    "minecraft:sweep_attack",
    "minecraft:totem_of_undying",
    "minecraft:underwater",
    "minecraft:splash",
    "minecraft:witch",
    "minecraft:bubble_pop",
    "minecraft:current_down",
    "minecraft:bubble_column_up",
    "minecraft:nautilus",
    "minecraft:dolphin",
    "minecraft:campfire_cosy_smoke",
    "minecraft:campfire_signal_smoke",
    "minecraft:dripping_honey",
    "minecraft:falling_honey",
    "minecraft:landing_honey",
    "minecraft:falling_nectar",
    "minecraft:falling_spore_blossom",
    "minecraft:ash",
    "minecraft:crimson_spore",
    "minecraft:warped_spore",
    "minecraft:spore_blossom_air",
    "minecraft:dripping_obsidian_tear",
    "minecraft:falling_obsidian_tear",
    "minecraft:landing_obsidian_tear",
    "minecraft:reverse_portal",
    "minecraft:white_ash",
    "minecraft:small_flame",
    "minecraft:snowflake",
    "minecraft:dripping_dripstone_lava",
    "minecraft:falling_dripstone_lava",
    "minecraft:dripping_dripstone_water",
    "minecraft:falling_dripstone_water",
    "minecraft:glow_squid_ink",
    "minecraft:glow",
    "minecraft:wax_on",
    "minecraft:wax_off",
    "minecraft:electric_spark",
    "minecraft:scrape",
    "minecraft:shriek",
    "minecraft:egg_crack",
    "minecraft:dust_plume",
    "minecraft:trial_spawner_detection",
    "minecraft:trial_spawner_detection_ominous",
    "minecraft:vault_connection",
    "minecraft:dust_pillar",
    "minecraft:ominous_spawning",
    "minecraft:raid_omen",
    "minecraft:trial_omen",
    "minecraft:block_crumble",
    "minecraft:firefly",
    "minecraft:sulfur_cube_goo",
];

/// Each type's `overrideLimiter`, by id: whether its particles show however far and however few
/// the options ask for.
pub(crate) static OVERRIDE_LIMITER: [bool; 128] = [
    false, false, true, false, false, false, false, true, true, true, true, false, false, false,
    true, false, false, false, false, false, false, false, false, false, true, false, false, false,
    false, true, true, true, false, true, true, true, false, false, false, false, false, false,
    false, false, false, false, false, false, true, true, false, false, false, false, false, false,
    false, false, true, false, false, false, false, false, false, false, false, false, false, true,
    false, false, false, false, false, true, true, true, false, false, false, false, false, false,
    false, false, false, true, true, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, true, true, true,
    true, true, true, false, false, false, true, true, true, false, true, false, false, false,
    false, false,
];

/// The simple name of each type's options class, by id (`SimpleParticleType`'s are `Simple`).
pub(crate) static OPTIONS: [&str; 128] = [
    "Simple",
    "BlockParticleOption",
    "BlockParticleOption",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "GeyserParticleOptions",
    "GeyserBaseParticleOptions",
    "GeyserBaseParticleOptions",
    "GeyserParticleOptions",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "PowerParticleOption",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "DustParticleOptions",
    "DustColorTransitionOptions",
    "SpellParticleOption",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "ColorParticleOption",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "BlockParticleOption",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "ColorParticleOption",
    "Simple",
    "SculkChargeParticleOptions",
    "Simple",
    "Simple",
    "Simple",
    "ColorParticleOption",
    "Simple",
    "Simple",
    "Simple",
    "SpellParticleOption",
    "ItemParticleOption",
    "VibrationParticleOption",
    "TrailParticleOption",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "ShriekParticleOption",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "Simple",
    "BlockParticleOption",
    "Simple",
    "Simple",
    "Simple",
    "BlockParticleOption",
    "Simple",
    "Simple",
];

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::golden::{Encode, Golden};
    use crate::net::minecraft::core::particles::particle_type::ParticleType;

    /// Every type by its registry id: its name, whether it overrides the limiter, and its
    /// options' class.
    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.particles.ParticleTypes");
        assert_eq!(
            golden.cases("types").len(),
            ParticleType::registry_size() as usize
        );
        golden.check("types", |id: i32| {
            let r#type = ParticleType::by_id(id).expect("a type for each id");
            Value::from(vec![
                Value::from(r#type.key()),
                r#type.get_override_limiter().encode(),
                Value::from(r#type.options_class()),
            ])
        });
        golden.finish();
    }
}
