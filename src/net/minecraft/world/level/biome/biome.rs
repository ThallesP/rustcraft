//! net/minecraft/world/level/biome/Biome.java
//!
//! A biome as the client knows it: its climate (`ClimateSettings`), its environment attributes
//! (`EnvironmentAttributeMap`, the syncable ones) and its colours (`BiomeSpecialEffects`), what
//! `NETWORK_CODEC` reads of it (`RegistryDataLoader`, from the server's NBT or a known pack's
//! JSON). Not ported yet: the temperature at a block (`getTemperature` with its cache,
//! `getHeightAdjustedTemperature`, `TemperatureModifier`'s `modifyTemperature` and
//! `FROZEN_TEMPERATURE_NOISE`), precipitation, freezing and snow, which come with weather, and
//! the generation settings and `DIRECT_CODEC`, which only the server has.
//!
//! `BIOME_INFO_NOISE` is a `SimplexNoise` of a `WorldgenRandom` over a `LegacyRandomSource`,
//! whose bits `WorldgenRandom` draws from the legacy source, so the noise is made from the legacy
//! source itself.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{BOOL, CodecExt, FLOAT, SharedCodec, StaticCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, SharedMapCodec, StaticMapCodec};
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

/// `Biome.TemperatureModifier`: how a biome changes its temperature where it is. Not ported yet:
/// `modifyTemperature`, which comes with the temperature at a block.
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
