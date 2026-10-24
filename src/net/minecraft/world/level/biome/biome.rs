//! net/minecraft/world/level/biome/Biome.java
//!
//! A biome as the client knows it: its climate (`ClimateSettings`), its environment attributes
//! (`EnvironmentAttributeMap`, the syncable ones) and its colours (`BiomeSpecialEffects`), what
//! `NETWORK_CODEC` reads of it (`RegistryDataLoader`, from the server's NBT or a known pack's
//! JSON), and the temperature at a block (`getTemperature`, without its cache, which only saves
//! work) with what rains or snows there (`getPrecipitationAt`). Not ported yet: freezing and
//! snow (`shouldFreeze`, `shouldSnow`), and the generation settings and `DIRECT_CODEC`, which only
//! the server has.
//!
//! `BIOME_INFO_NOISE`, `TEMPERATURE_NOISE` and `FROZEN_TEMPERATURE_NOISE`'s layers are
//! `SimplexNoise`s of a `WorldgenRandom` over a `LegacyRandomSource`, whose bits `WorldgenRandom`
//! draws from the legacy source, so each noise is made from the legacy source itself (the frozen
//! noise's three from one, in turn).

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{BOOL, CodecExt, FLOAT, SharedCodec, StaticCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, SharedMapCodec, StaticMapCodec};
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::string_representable::{self, StringRepresentable};
use crate::net::minecraft::world::attribute::environment_attribute_map::EnvironmentAttributeMap;
use crate::net::minecraft::world::level::biome::biome_special_effects::BiomeSpecialEffects;
use crate::net::minecraft::world::level::dry_foliage_color::DryFoliageColor;
use crate::net::minecraft::world::level::foliage_color::FoliageColor;
use crate::net::minecraft::world::level::grass_color::GrassColor;
use crate::net::minecraft::world::level::levelgen::legacy_random_source::LegacyRandomSource;
use crate::net::minecraft::world::level::levelgen::synth::simplex_noise::SimplexNoise;

/// `BIOME_INFO_NOISE`: deprecated in Java, and still what the swamp's grass reads.
pub static BIOME_INFO_NOISE: LazyLock<SimplexNoise> =
    LazyLock::new(|| SimplexNoise::new_discarding_offset(&mut LegacyRandomSource::new(2345)));

/// `TEMPERATURE_NOISE`: how much colder it gets above the snow line, by column.
static TEMPERATURE_NOISE: LazyLock<SimplexNoise> =
    LazyLock::new(|| SimplexNoise::new_discarding_offset(&mut LegacyRandomSource::new(1234)));

/// `FROZEN_TEMPERATURE_NOISE`: a `NoiseStack` of three simplex noises, each a layer of a frequency
/// and an amplitude, which frozen biomes' ice patches follow.
pub static FROZEN_TEMPERATURE_NOISE: LazyLock<[(SimplexNoise, f64, f32); 3]> =
    LazyLock::new(|| {
        let mut random = LegacyRandomSource::new(3456);
        [
            (
                SimplexNoise::new_discarding_offset(&mut random),
                1.0,
                0.14285715,
            ),
            (
                SimplexNoise::new_discarding_offset(&mut random),
                0.5,
                0.2857143,
            ),
            (
                SimplexNoise::new_discarding_offset(&mut random),
                0.25,
                0.5714286,
            ),
        ]
    });

/// `NoiseStack.get(double, double)` of the frozen noise: its layers' values summed as floats.
pub fn frozen_temperature_noise(x: f64, y: f64) -> f32 {
    let mut value = 0.0_f32;
    for (noise, frequency, amplitude) in FROZEN_TEMPERATURE_NOISE.iter() {
        value += amplitude * noise.get_2d(x * frequency, y * frequency);
    }
    value
}

static NETWORK_CODEC: StaticCodec<Biome> = LazyLock::new(|| {
    record_codec_builder::create(
        (
            CLIMATE_SETTINGS_CODEC
                .shared()
                .for_getter(|o: &Biome| o.climate_settings.clone()),
            EnvironmentAttributeMap::NETWORK_CODEC
                .shared()
                .optional_field_of_default("attributes", EnvironmentAttributeMap::empty())
                .for_getter(|o: &Biome| o.attributes.clone()),
            BiomeSpecialEffects::CODEC
                .shared()
                .field_of("effects")
                .for_getter(|o: &Biome| o.special_effects.clone()),
        ),
        Biome::new,
    )
    .boxed()
});

/// A biome's climate, attributes and colours.
#[derive(Clone, Debug, PartialEq)]
pub struct Biome {
    climate_settings: ClimateSettings,
    attributes: EnvironmentAttributeMap,
    special_effects: BiomeSpecialEffects,
}

impl Biome {
    pub const NETWORK_CODEC: &'static StaticCodec<Biome> = &NETWORK_CODEC;

    /// The constructor `NETWORK_CODEC` makes a biome with, without generation settings.
    pub fn new(
        climate_settings: ClimateSettings,
        attributes: EnvironmentAttributeMap,
        special_effects: BiomeSpecialEffects,
    ) -> Biome {
        Biome {
            climate_settings,
            attributes,
            special_effects,
        }
    }

    pub fn get_attributes(&self) -> &EnvironmentAttributeMap {
        &self.attributes
    }

    pub fn has_precipitation(&self) -> bool {
        self.climate_settings.has_precipitation
    }

    pub fn get_grass_color(&self, x: f64, z: f64) -> i32 {
        let base_grass_color = self.get_base_grass_color();
        self.special_effects
            .grass_color_modifier()
            .modify_color(x, z, base_grass_color)
    }

    fn get_base_grass_color(&self) -> i32 {
        match self.special_effects.grass_color_override() {
            Some(color_override) => color_override,
            None => self.get_grass_color_from_texture(),
        }
    }

    fn get_grass_color_from_texture(&self) -> i32 {
        let temp = f64::from(Mth::clamp_f32(self.climate_settings.temperature, 0.0, 1.0));
        let rain = f64::from(Mth::clamp_f32(self.climate_settings.downfall, 0.0, 1.0));
        GrassColor::get(temp, rain)
    }

    pub fn get_foliage_color(&self) -> i32 {
        self.special_effects
            .foliage_color_override()
            .unwrap_or_else(|| self.get_foliage_color_from_texture())
    }

    fn get_foliage_color_from_texture(&self) -> i32 {
        let temp = f64::from(Mth::clamp_f32(self.climate_settings.temperature, 0.0, 1.0));
        let rain = f64::from(Mth::clamp_f32(self.climate_settings.downfall, 0.0, 1.0));
        FoliageColor::get(temp, rain)
    }

    pub fn get_dry_foliage_color(&self) -> i32 {
        self.special_effects
            .dry_foliage_color_override()
            .unwrap_or_else(|| self.get_dry_foliage_color_from_texture())
    }

    fn get_dry_foliage_color_from_texture(&self) -> i32 {
        let temp = f64::from(Mth::clamp_f32(self.climate_settings.temperature, 0.0, 1.0));
        let rain = f64::from(Mth::clamp_f32(self.climate_settings.downfall, 0.0, 1.0));
        DryFoliageColor::get(temp, rain)
    }

    pub fn get_base_temperature(&self) -> f32 {
        self.climate_settings.temperature
    }

    /// `getPrecipitationAt`: none where it never rains, snow where it's cold enough, rain
    /// elsewhere.
    pub fn get_precipitation_at(&self, pos: &BlockPos, sea_level: i32) -> Precipitation {
        if !self.has_precipitation() {
            Precipitation::NONE
        } else if self.cold_enough_to_snow(pos, sea_level) {
            Precipitation::SNOW
        } else {
            Precipitation::RAIN
        }
    }

    /// `getHeightAdjustedTemperature`: the modified temperature, colder higher above the snow
    /// line (17 above the sea).
    fn get_height_adjusted_temperature(&self, pos: &BlockPos, sea_level: i32) -> f32 {
        let adjusted_temperature = self
            .climate_settings
            .temperature_modifier
            .modify_temperature(pos, self.get_base_temperature());
        let snow_level = sea_level.wrapping_add(17);
        if pos.get_y() > snow_level {
            let v = TEMPERATURE_NOISE.get_2d(
                f64::from(pos.get_x() as f32 / 8.0),
                f64::from(pos.get_z() as f32 / 8.0),
            ) * 8.0;
            adjusted_temperature - (v + pos.get_y() as f32 - snow_level as f32) * 0.05 / 40.0
        } else {
            adjusted_temperature
        }
    }

    /// `getTemperature`: Java keeps the last 1,024 blocks' in a cache, which only saves work.
    pub fn get_temperature(&self, pos: &BlockPos, sea_level: i32) -> f32 {
        self.get_height_adjusted_temperature(pos, sea_level)
    }

    /// `warmEnoughToRain`.
    pub fn warm_enough_to_rain(&self, pos: &BlockPos, sea_level: i32) -> bool {
        self.get_temperature(pos, sea_level) >= 0.15
    }

    /// `coldEnoughToSnow`.
    pub fn cold_enough_to_snow(&self, pos: &BlockPos, sea_level: i32) -> bool {
        !self.warm_enough_to_rain(pos, sea_level)
    }

    pub fn get_special_effects(&self) -> &BiomeSpecialEffects {
        &self.special_effects
    }

    pub fn get_climate_settings(&self) -> &ClimateSettings {
        &self.climate_settings
    }

    pub fn get_water_color(&self) -> i32 {
        self.special_effects.water_color()
    }
}

static CLIMATE_SETTINGS_CODEC: StaticMapCodec<ClimateSettings> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (
            BOOL.field_of("has_precipitation")
                .for_getter(|o: &ClimateSettings| o.has_precipitation),
            FLOAT
                .field_of("temperature")
                .for_getter(|o: &ClimateSettings| o.temperature),
            TemperatureModifier::CODEC
                .shared()
                .optional_field_of_default("temperature_modifier", TemperatureModifier::NONE)
                .for_getter(|o: &ClimateSettings| o.temperature_modifier),
            FLOAT
                .field_of("downfall")
                .for_getter(|o: &ClimateSettings| o.downfall),
        ),
        ClimateSettings::new,
    )
    .boxed()
});

/// `Biome.ClimateSettings`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClimateSettings {
    has_precipitation: bool,
    temperature: f32,
    temperature_modifier: TemperatureModifier,
    downfall: f32,
}

impl ClimateSettings {
    pub const CODEC: &'static StaticMapCodec<ClimateSettings> = &CLIMATE_SETTINGS_CODEC;

    pub fn new(
        has_precipitation: bool,
        temperature: f32,
        temperature_modifier: TemperatureModifier,
        downfall: f32,
    ) -> ClimateSettings {
        ClimateSettings {
            has_precipitation,
            temperature,
            temperature_modifier,
            downfall,
        }
    }

    pub fn has_precipitation(&self) -> bool {
        self.has_precipitation
    }

    pub fn temperature(&self) -> f32 {
        self.temperature
    }

    pub fn temperature_modifier(&self) -> TemperatureModifier {
        self.temperature_modifier
    }

    pub fn downfall(&self) -> f32 {
        self.downfall
    }
}

/// `Biome.Precipitation`: what falls in a biome when it rains.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(clippy::upper_case_acronyms)]
pub enum Precipitation {
    NONE,
    RAIN,
    SNOW,
}

/// `Biome.TemperatureModifier`: how a biome changes its temperature where it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(clippy::upper_case_acronyms)]
pub enum TemperatureModifier {
    NONE,
    FROZEN,
}

static TEMPERATURE_MODIFIER_CODEC: StaticCodec<TemperatureModifier> =
    LazyLock::new(|| string_representable::from_values(&TemperatureModifier::VALUES));

impl TemperatureModifier {
    pub const VALUES: [TemperatureModifier; 2] =
        [TemperatureModifier::NONE, TemperatureModifier::FROZEN];
    pub const CODEC: &'static StaticCodec<TemperatureModifier> = &TEMPERATURE_MODIFIER_CODEC;

    /// `modifyTemperature`: a frozen biome is 0.2 in its ice patches.
    pub fn modify_temperature(&self, pos: &BlockPos, base_temperature: f32) -> f32 {
        match self {
            TemperatureModifier::NONE => base_temperature,
            TemperatureModifier::FROZEN => {
                let ground_value_large_variation = f64::from(
                    frozen_temperature_noise(
                        f64::from(pos.get_x()) * 0.05,
                        f64::from(pos.get_z()) * 0.05,
                    ) * 7.0,
                );
                let ground_value_edge_variation = f64::from(
                    BIOME_INFO_NOISE
                        .get_2d(f64::from(pos.get_x()) * 0.2, f64::from(pos.get_z()) * 0.2),
                );
                let ice_patches = ground_value_large_variation + ground_value_edge_variation;
                if ice_patches < 0.3 {
                    let ground_value_small_variation = f64::from(
                        BIOME_INFO_NOISE
                            .get_2d(f64::from(pos.get_x()) * 0.09, f64::from(pos.get_z()) * 0.09),
                    );
                    if ground_value_small_variation < 0.8 {
                        return 0.2;
                    }
                }

                base_temperature
            }
        }
    }
}

impl StringRepresentable for TemperatureModifier {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            TemperatureModifier::NONE => "none",
            TemperatureModifier::FROZEN => "frozen",
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::{Value, json};

    use super::Biome;
    use crate::com::mojang::serialization::decoder::Decoder;
    use crate::golden::{Decode, Golden, unhex};
    use crate::io::netty::buffer::byte_buf_input_stream::ByteBufInputStream;
    use crate::io::netty::buffer::unpooled;
    use crate::net::minecraft::client::multiplayer::registry_data_collector::RegistryDataCollector;
    use crate::net::minecraft::core::registries::built_in_registries::BuiltInRegistries;
    use crate::net::minecraft::core::registries::registries::Registries;
    use crate::net::minecraft::core::registry_access::RegistryAccess;
    use crate::net::minecraft::core::registry_synchronization::PackedRegistryEntry;
    use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
    use crate::net::minecraft::nbt::nbt_io::NbtIo;
    use crate::net::minecraft::nbt::nbt_ops::NbtOps;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::resources::registry_ops::RegistryOps;
    use crate::net::minecraft::util::string_representable::StringRepresentable;
    use crate::net::minecraft::world::level::color_map_color_util::tests::load_colormaps;

    /// The registries the client collects from the walk's registry data: the biomes by name, none
    /// sent whole, so each is read from the vanilla pack, like a known pack's, and the vanilla
    /// dimension types, which a level reads its height from.
    pub(crate) fn biome_registries(biomes: &Value) -> RegistryAccess {
        let entries: Vec<PackedRegistryEntry> = Vec::<String>::decode(biomes)
            .iter()
            .map(|name| PackedRegistryEntry::new(Identifier::parse(name), None))
            .collect();
        let mut collector = RegistryDataCollector::new();
        collector.append_contents(Registries::BIOME, &entries);
        collector.append_contents(
            Registries::DIMENSION_TYPE,
            &crate::net::minecraft::client::multiplayer::registry_data_collector::tests::dimension_type_entries(),
        );
        collector.collect_game_registries()
    }

    /// The suite's encoding of a biome.
    fn fields(biome: &Biome) -> Value {
        let climate = biome.get_climate_settings();
        let effects = biome.get_special_effects();
        json!([
            climate.has_precipitation(),
            format!("{:08x}", climate.temperature().to_bits()),
            climate.temperature_modifier().get_serialized_name(),
            format!("{:08x}", climate.downfall().to_bits()),
            effects.water_color(),
            effects.foliage_color_override(),
            effects.dry_foliage_color_override(),
            effects.grass_color_override(),
            effects.grass_color_modifier().get_serialized_name(),
        ])
    }

    fn read_nbt(hex: &Value) -> crate::net::minecraft::nbt::tag::Tag {
        let mut buffer = unpooled::wrapped_buffer(unhex(hex.as_str().expect("hex")));
        NbtIo::read_any_tag(
            &mut ByteBufInputStream::new(&mut buffer),
            &mut NbtAccounter::default_quota(),
        )
    }

    /// Every biome of the walk's registry, read from the vanilla pack's JSON and from the NBT a
    /// server would send, and its colours anywhere, as vanilla's `NETWORK_CODEC` reads them.
    #[test]
    fn biomes_match_vanilla() {
        load_colormaps();
        let golden = Golden::load("net.minecraft.world.level.biome.Biome");
        let registries = biome_registries(&golden.input("BIOMES"));
        let biomes = registries.lookup_or_throw(Registries::BIOME);
        let biome = |id: &Value| {
            let key = crate::net::minecraft::resources::resource_key::ResourceKey::create(
                Registries::BIOME,
                Identifier::parse(id.as_str().expect("an identifier")),
            );
            biomes.value::<Biome>(&biomes.get_or_throw(&key)).clone()
        };
        let json_cases = golden.cases("json");
        assert_eq!(
            json_cases.len(),
            golden.input("BIOMES").as_array().expect("names").len()
        );
        for (args, expected) in json_cases {
            assert_eq!(fields(&biome(&args[0])), expected[0], "{}", args[0]);
        }
        // NETWORK_CODEC over registry ops of the static registries, as the harness reads it.
        let static_registries = RegistryAccess::new(BuiltInRegistries::registries());
        let ops = RegistryOps::create(&NbtOps::INSTANCE, &static_registries);
        for (args, expected) in golden.cases("nbt") {
            let biome = Biome::NETWORK_CODEC
                .parse(&ops, &read_nbt(&args[0]))
                .get_or_throw();
            assert_eq!(fields(&biome), expected);
        }
        for (args, expected) in golden.cases("colors") {
            let biome = biome(&args[0]);
            let (x, z) = (f64::decode(&args[1]), f64::decode(&args[2]));
            let actual = json!([
                biome.get_grass_color(x, z),
                biome.get_foliage_color(),
                biome.get_dry_foliage_color(),
                biome.get_water_color()
            ]);
            assert_eq!(actual, expected, "{} at {x}, {z}", args[0]);
        }
        golden.finish();
    }
}
