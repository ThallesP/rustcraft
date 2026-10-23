//! net/minecraft/world/attribute/EnvironmentAttributes.java
//!
//! The environment attributes, in the order they're registered: each constant is its place among
//! the ported ones, and `info` is what Java's builder made. Not ported: the attributes of the
//! types `AttributeType` doesn't have (`UNPORTED`), which data naming them leaves out, and
//! `CODEC`, which is `by_name`.

use std::sync::LazyLock;

use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::world::attribute::attribute_range::AttributeRange;
use crate::net::minecraft::world::attribute::attribute_type::{AttributeType, AttributeValue};
use crate::net::minecraft::world::attribute::environment_attribute::{
    AttributeInfo, EnvironmentAttribute,
};
use crate::net::minecraft::world::level::moon_phase::MoonPhase;
use crate::org::joml::vector3f::Vector3f;
use crate::org::joml::vector4f::Vector4f;

pub struct EnvironmentAttributes;

static ATTRIBUTES: LazyLock<Vec<AttributeInfo>> = LazyLock::new(|| {
    use AttributeType::*;
    let rgb = |color: i32| AttributeValue::Rgb(ARGB::vector3f_from_rgb24(color));
    let argb = |color: i32| AttributeValue::Argb(ARGB::vector4f_from_argb32(color));
    let float = AttributeValue::Float;
    let boolean = AttributeValue::Boolean;
    let attribute = AttributeInfo::builder;
    vec![
        attribute("visual/fog_color", RGB_COLOR, rgb(0))
            .spatially_interpolated()
            .syncable(),
        attribute("visual/fog_start_distance", FLOAT, float(0.0))
            .spatially_interpolated()
            .syncable(),
        attribute("visual/fog_end_distance", FLOAT, float(1024.0))
            .value_range(AttributeRange::NON_NEGATIVE_FLOAT)
            .spatially_interpolated()
            .syncable(),
        attribute("visual/sky_fog_end_distance", FLOAT, float(512.0))
            .value_range(AttributeRange::NON_NEGATIVE_FLOAT)
            .spatially_interpolated()
            .syncable(),
        attribute("visual/cloud_fog_end_distance", FLOAT, float(2048.0))
            .value_range(AttributeRange::NON_NEGATIVE_FLOAT)
            .spatially_interpolated()
            .syncable(),
        attribute("visual/water_fog_color", RGB_COLOR, rgb(-16448205))
            .spatially_interpolated()
            .syncable(),
        attribute("visual/water_fog_start_distance", FLOAT, float(-8.0))
            .spatially_interpolated()
            .syncable(),
        attribute("visual/water_fog_end_distance", FLOAT, float(96.0))
            .value_range(AttributeRange::NON_NEGATIVE_FLOAT)
            .spatially_interpolated()
            .syncable(),
        attribute("visual/sky_color", RGB_COLOR, rgb(0))
            .spatially_interpolated()
            .syncable(),
        attribute("visual/sunrise_sunset_color", ARGB_COLOR, argb(0))
            .spatially_interpolated()
            .syncable(),
        attribute("visual/cloud_color", ARGB_COLOR, argb(0))
            .spatially_interpolated()
            .syncable(),
        attribute("visual/cloud_height", FLOAT, float(192.33))
            .spatially_interpolated()
            .syncable(),
        attribute("visual/sun_angle", ANGLE_DEGREES, float(0.0))
            .spatially_interpolated()
            .syncable(),
        attribute("visual/moon_angle", ANGLE_DEGREES, float(0.0))
            .spatially_interpolated()
            .syncable(),
        attribute("visual/star_angle", ANGLE_DEGREES, float(0.0))
            .spatially_interpolated()
            .syncable(),
        attribute(
            "visual/moon_phase",
            MOON_PHASE,
            AttributeValue::MoonPhase(MoonPhase::FULL_MOON),
        )
        .syncable(),
        attribute("visual/star_brightness", FLOAT, float(0.0))
            .value_range(AttributeRange::UNIT_FLOAT)
            .spatially_interpolated()
            .syncable(),
        attribute("visual/block_light_tint", RGB_COLOR, rgb(-10100))
            .spatially_interpolated()
            .syncable(),
        attribute("visual/sky_light_color", RGB_COLOR, rgb(-1))
            .spatially_interpolated()
            .syncable(),
        attribute("visual/sky_light_factor", FLOAT, float(1.0))
            .value_range(AttributeRange::UNIT_FLOAT)
            .spatially_interpolated()
            .syncable(),
        attribute("visual/night_vision_color", RGB_COLOR, rgb(-6710887))
            .spatially_interpolated()
            .syncable(),
        attribute("visual/ambient_light_color", RGB_COLOR, rgb(-16777216))
            .spatially_interpolated()
            .syncable(),
        attribute("audio/music_volume", FLOAT, float(1.0))
            .value_range(AttributeRange::UNIT_FLOAT)
            .syncable(),
        attribute("audio/firefly_bush_sounds", BOOLEAN, boolean(false)).syncable(),
        attribute("gameplay/sky_light_level", FLOAT, float(15.0))
            .value_range(AttributeRange::of_float(0.0, 15.0))
            .not_positional()
            .syncable(),
        attribute("gameplay/can_start_raid", BOOLEAN, boolean(true)),
        attribute("gameplay/water_evaporates", BOOLEAN, boolean(false)).syncable(),
        attribute("gameplay/respawn_anchor_works", BOOLEAN, boolean(false)),
        attribute(
            "gameplay/nether_portal_spawns_piglin",
            BOOLEAN,
            boolean(false),
        ),
        attribute("gameplay/fast_lava", BOOLEAN, boolean(false))
            .not_positional()
            .syncable(),
        attribute("gameplay/increased_fire_burnout", BOOLEAN, boolean(false)),
        attribute("gameplay/turtle_egg_hatch_chance", FLOAT, float(0.002))
            .value_range(AttributeRange::UNIT_FLOAT),
        attribute("gameplay/piglins_zombify", BOOLEAN, boolean(true)).syncable(),
        attribute("gameplay/snow_golem_melts", BOOLEAN, boolean(false)),
        attribute("gameplay/creaking_active", BOOLEAN, boolean(false)).syncable(),
        attribute("gameplay/surface_slime_spawn_chance", FLOAT, float(0.0))
            .value_range(AttributeRange::UNIT_FLOAT),
        attribute("gameplay/cat_waking_up_gift_chance", FLOAT, float(0.0))
            .value_range(AttributeRange::UNIT_FLOAT),
        attribute("gameplay/bees_stay_in_hive", BOOLEAN, boolean(false)),
        attribute("gameplay/monsters_burn", BOOLEAN, boolean(false)),
        attribute("gameplay/can_pillager_patrol_spawn", BOOLEAN, boolean(true)),
        attribute(
            "gameplay/creature_world_gen_spawn_probability",
            FLOAT,
            float(0.1),
        )
        .value_range(AttributeRange::UNIT_FLOAT_EPSILON),
    ]
});

impl EnvironmentAttributes {
    pub const FOG_COLOR: EnvironmentAttribute<Vector3f> = EnvironmentAttribute::new(0);
    pub const FOG_START_DISTANCE: EnvironmentAttribute<f32> = EnvironmentAttribute::new(1);
    pub const FOG_END_DISTANCE: EnvironmentAttribute<f32> = EnvironmentAttribute::new(2);
    pub const SKY_FOG_END_DISTANCE: EnvironmentAttribute<f32> = EnvironmentAttribute::new(3);
    pub const CLOUD_FOG_END_DISTANCE: EnvironmentAttribute<f32> = EnvironmentAttribute::new(4);
    pub const WATER_FOG_COLOR: EnvironmentAttribute<Vector3f> = EnvironmentAttribute::new(5);
    pub const WATER_FOG_START_DISTANCE: EnvironmentAttribute<f32> = EnvironmentAttribute::new(6);
    pub const WATER_FOG_END_DISTANCE: EnvironmentAttribute<f32> = EnvironmentAttribute::new(7);
    pub const SKY_COLOR: EnvironmentAttribute<Vector3f> = EnvironmentAttribute::new(8);
    pub const SUNRISE_SUNSET_COLOR: EnvironmentAttribute<Vector4f> = EnvironmentAttribute::new(9);
    pub const CLOUD_COLOR: EnvironmentAttribute<Vector4f> = EnvironmentAttribute::new(10);
    pub const CLOUD_HEIGHT: EnvironmentAttribute<f32> = EnvironmentAttribute::new(11);
    pub const SUN_ANGLE: EnvironmentAttribute<f32> = EnvironmentAttribute::new(12);
    pub const MOON_ANGLE: EnvironmentAttribute<f32> = EnvironmentAttribute::new(13);
    pub const STAR_ANGLE: EnvironmentAttribute<f32> = EnvironmentAttribute::new(14);
    pub const MOON_PHASE: EnvironmentAttribute<MoonPhase> = EnvironmentAttribute::new(15);
    pub const STAR_BRIGHTNESS: EnvironmentAttribute<f32> = EnvironmentAttribute::new(16);
    pub const BLOCK_LIGHT_TINT: EnvironmentAttribute<Vector3f> = EnvironmentAttribute::new(17);
    pub const SKY_LIGHT_COLOR: EnvironmentAttribute<Vector3f> = EnvironmentAttribute::new(18);
    pub const SKY_LIGHT_FACTOR: EnvironmentAttribute<f32> = EnvironmentAttribute::new(19);
    pub const NIGHT_VISION_COLOR: EnvironmentAttribute<Vector3f> = EnvironmentAttribute::new(20);
    pub const AMBIENT_LIGHT_COLOR: EnvironmentAttribute<Vector3f> = EnvironmentAttribute::new(21);
    pub const MUSIC_VOLUME: EnvironmentAttribute<f32> = EnvironmentAttribute::new(22);
    pub const FIREFLY_BUSH_SOUNDS: EnvironmentAttribute<bool> = EnvironmentAttribute::new(23);
    pub const SKY_LIGHT_LEVEL: EnvironmentAttribute<f32> = EnvironmentAttribute::new(24);
    pub const CAN_START_RAID: EnvironmentAttribute<bool> = EnvironmentAttribute::new(25);
    pub const WATER_EVAPORATES: EnvironmentAttribute<bool> = EnvironmentAttribute::new(26);
    pub const RESPAWN_ANCHOR_WORKS: EnvironmentAttribute<bool> = EnvironmentAttribute::new(27);
    pub const NETHER_PORTAL_SPAWNS_PIGLINS: EnvironmentAttribute<bool> =
        EnvironmentAttribute::new(28);
    pub const FAST_LAVA: EnvironmentAttribute<bool> = EnvironmentAttribute::new(29);
    pub const INCREASED_FIRE_BURNOUT: EnvironmentAttribute<bool> = EnvironmentAttribute::new(30);
    pub const TURTLE_EGG_HATCH_CHANCE: EnvironmentAttribute<f32> = EnvironmentAttribute::new(31);
    pub const PIGLINS_ZOMBIFY: EnvironmentAttribute<bool> = EnvironmentAttribute::new(32);
    pub const SNOW_GOLEM_MELTS: EnvironmentAttribute<bool> = EnvironmentAttribute::new(33);
    pub const CREAKING_ACTIVE: EnvironmentAttribute<bool> = EnvironmentAttribute::new(34);
    pub const SURFACE_SLIME_SPAWN_CHANCE: EnvironmentAttribute<f32> = EnvironmentAttribute::new(35);
    pub const CAT_WAKING_UP_GIFT_CHANCE: EnvironmentAttribute<f32> = EnvironmentAttribute::new(36);
    pub const BEES_STAY_IN_HIVE: EnvironmentAttribute<bool> = EnvironmentAttribute::new(37);
    pub const MONSTERS_BURN: EnvironmentAttribute<bool> = EnvironmentAttribute::new(38);
    pub const CAN_PILLAGER_PATROL_SPAWN: EnvironmentAttribute<bool> = EnvironmentAttribute::new(39);
    pub const CREATURE_WORLD_GEN_SPAWN_PROBABILITY: EnvironmentAttribute<f32> =
        EnvironmentAttribute::new(40);

    /// The registered attributes whose types aren't ported, by name.
    pub const UNPORTED: [&'static str; 10] = [
        "visual/default_dripstone_particle",
        "visual/ambient_particles",
        "audio/background_music",
        "audio/ambient_sounds",
        "gameplay/bed_rule",
        "gameplay/straw_bed_rule",
        "gameplay/eyeblossom_open",
        "gameplay/natural_mob_spawns",
        "gameplay/villager_activity",
        "gameplay/baby_villager_activity",
    ];

    /// How many attributes are ported, the places' bound.
    pub fn count() -> usize {
        ATTRIBUTES.len()
    }

    pub fn info(id: usize) -> &'static AttributeInfo {
        &ATTRIBUTES[id]
    }

    /// `CODEC`: the ported attribute's place, `Ok(None)` for one whose type isn't ported, or Java's
    /// error for a name the registry doesn't have.
    pub fn by_name(name: &Identifier) -> Result<Option<usize>, String> {
        if name.get_namespace() == "minecraft" {
            let path = name.get_path();
            if let Some(id) = ATTRIBUTES.iter().position(|info| info.name == path) {
                return Ok(Some(id));
            }
            if EnvironmentAttributes::UNPORTED.contains(&path) {
                return Ok(None);
            }
        }
        Err(format!(
            "Unknown registry key in minecraft:environment_attribute: {name}"
        ))
    }
}
