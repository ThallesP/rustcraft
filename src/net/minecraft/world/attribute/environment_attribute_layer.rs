//! net/minecraft/world/attribute/EnvironmentAttributeLayer.java
//!
//! One step of working an attribute's value out: constant, of the time, or of the position.
//! Java's layers are lambdas over the level; here they're an enum of the layers the game makes,
//! and what they read of the level is a parameter, `LayerLevel`.

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::attribute::attribute_type::AttributeValue;
use crate::net::minecraft::world::attribute::environment_attribute_map::{
    Entry, EnvironmentAttributeMap,
};
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::net::minecraft::world::attribute::spatial_attribute_interpolator::SpatialAttributeInterpolator;
use crate::net::minecraft::world::phys::vec3::Vec3;
use crate::net::minecraft::world::timeline::attribute_track_sampler::AttributeTrackSampler;
use crate::org::joml::vector3f::Vector3f;

/// What a level's layers read of it as they're applied, which Java's lambdas close over: its
/// clocks (`ClockManager`), its weather (`WeatherAttributes.WeatherAccess`, the levels at partial
/// tick 1), its lightning flash (`ClientLevel.getSkyFlashTime`) and its biomes (`BiomeManager`'s
/// noise biomes).
pub trait LayerLevel {
    /// `clockManager.getInstance(clock).totalTicks()`.
    fn clock_total_ticks(&self, clock: &Holder) -> i64;

    fn rain_level(&self) -> f32;

    fn thunder_level(&self) -> f32;

    fn sky_flash_time(&self) -> i32;

    /// `biomeManager.getNoiseBiomeAtQuart`.
    fn get_noise_biome_at_quart(&self, quart_x: i32, quart_y: i32, quart_z: i32) -> Holder;
}

pub enum EnvironmentAttributeLayer {
    /// `Constant`: an attribute map's entry (`addConstantLayer`).
    Constant(Entry),
    TimeBased(TimeBased),
    Positional(Positional),
}

/// The time-based layers the game makes.
pub enum TimeBased {
    /// A timeline's track (`addTimelineLayer`).
    Track(AttributeTrackSampler),
    /// `WeatherAttributes.addLayer`: the rain's entry, then the thunder's, each blended in by
    /// its level.
    Weather {
        rain_entry: Option<Entry>,
        thunder_entry: Option<Entry>,
    },
    /// `ClientLevel.addEnvironmentAttributeLayers`' sky color during a lightning flash.
    SkyFlashColor,
    /// `ClientLevel.addEnvironmentAttributeLayers`' sky light factor during a lightning flash.
    SkyFlashFactor,
}

/// The positional layers the game makes.
pub enum Positional {
    /// `EnvironmentAttributeSystem.addBiomeLayerForAttribute`: the biomes' entries, interpolated
    /// or at the position.
    Biome,
}

impl TimeBased {
    pub fn apply_time_based(
        &self,
        attribute: usize,
        base_value: AttributeValue,
        cache_tick_id: i32,
        level: &dyn LayerLevel,
    ) -> AttributeValue {
        match self {
            TimeBased::Track(sampler) => sampler.apply_time_based(base_value, cache_tick_id, level),
            TimeBased::Weather {
                rain_entry,
                thunder_entry,
            } => {
                let state_change_lerp = EnvironmentAttributes::info(attribute)
                    .r#type
                    .state_change_lerp();
                let mut result = base_value;
                let thunder_level = level.thunder_level();
                let rain_level = level.rain_level() - thunder_level;
                if let Some(rain_entry) = rain_entry
                    && rain_level > 0.0
                {
                    let rain_value = rain_entry.apply_modifier(result);
                    result = state_change_lerp.apply(rain_level, result, rain_value);
                }

                if let Some(thunder_entry) = thunder_entry
                    && thunder_level > 0.0
                {
                    let thunder_value = thunder_entry.apply_modifier(result);
                    result = state_change_lerp.apply(thunder_level, result, thunder_value);
                }

                result
            }
            TimeBased::SkyFlashColor => {
                if level.sky_flash_time() > 0 {
                    let flash_color = Vector3f::from_xyz(0.8, 0.8, 1.0);
                    AttributeValue::Rgb(ARGB::srgb_lerp_vector3f(
                        0.22,
                        &base_value.as_rgb(),
                        &flash_color,
                    ))
                } else {
                    base_value
                }
            }
            TimeBased::SkyFlashFactor => {
                if level.sky_flash_time() > 0 {
                    AttributeValue::Float(1.0)
                } else {
                    base_value
                }
            }
        }
    }
}

impl Positional {
    /// `biomes` are the registry's biomes, each with its attribute map.
    pub fn apply_positional(
        &self,
        attribute: usize,
        base_value: AttributeValue,
        pos: &Vec3,
        biome_interpolator: Option<&SpatialAttributeInterpolator>,
        level: &dyn LayerLevel,
        biomes: &[(Holder, EnvironmentAttributeMap)],
    ) -> AttributeValue {
        match self {
            Positional::Biome => {
                let info = EnvironmentAttributes::info(attribute);
                if let Some(biome_weights) = biome_interpolator
                    && info.is_spatially_interpolated
                {
                    return biome_weights.apply_attribute_layer(attribute, base_value, biomes);
                }

                assert!(
                    !info.full_resolution_biomes,
                    "full resolution biomes (BiomeManager.getBiome) aren't ported"
                );
                // BiomeManager.getNoiseBiomeAtPosition: QuartPos.fromBlock of each floored.
                let biome = level.get_noise_biome_at_quart(
                    Mth::floor_f64(pos.x) >> 2,
                    Mth::floor_f64(pos.y) >> 2,
                    Mth::floor_f64(pos.z) >> 2,
                );
                attributes_of(biomes, &biome).apply_modifier(attribute, base_value)
            }
        }
    }
}

/// The biome's attribute map among the registry's biomes: Java's `biome.value().getAttributes()`.
/// Panics for a biome the registry doesn't have, which Java's holder couldn't be.
pub fn attributes_of<'a>(
    biomes: &'a [(Holder, EnvironmentAttributeMap)],
    biome: &Holder,
) -> &'a EnvironmentAttributeMap {
    biomes
        .iter()
        .find(|(holder, _)| holder == biome)
        .map(|(_, attributes)| attributes)
        .unwrap_or_else(|| panic!("the biome {biome:?} isn't in the registry"))
}
