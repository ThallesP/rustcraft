//! net/minecraft/world/level/biome/Biome.java
//!
//! A biome as the client knows it: its climate (`ClimateSettings`) and its colours
//! (`BiomeSpecialEffects`), what `NETWORK_CODEC` reads of it. Not ported yet: its environment
//! attributes (`EnvironmentAttributeMap`: the sky's, fog's and water fog's colours, ambient
//! particles and sounds, which come with the sky and fog), the temperature at a block
//! (`getTemperature` with its cache, `getHeightAdjustedTemperature`, `TemperatureModifier`'s
//! `modifyTemperature` and `FROZEN_TEMPERATURE_NOISE`), precipitation, freezing and snow, which
//! come with weather, and the generation settings, which only the server has.
//!
//! **Temporary readers, until DataFixerUpper's codecs come** (the net track's): Java reads a
//! biome with `NETWORK_CODEC`, from the server's NBT (`from_network_nbt`), or for a biome of a
//! pack the client knows, from that pack's JSON (`from_network_json`, through Gson's tree). Each
//! reads the codec's fields with their defaults, and panics where the codec would give an error.
//! A colour (`ExtraCodecs.STRING_RGB_COLOR`) is read as its hex string or an int; its third form,
//! a list of three floats, which no vanilla biome has, panics.
//!
//! `BIOME_INFO_NOISE` is a `SimplexNoise` of a `WorldgenRandom` over a `LegacyRandomSource`,
//! whose bits `WorldgenRandom` draws from the legacy source, so the noise is made from the legacy
//! source itself.

use std::sync::LazyLock;

use crate::com::google::gson::json_element::{JsonElement, JsonObject};
use crate::net::minecraft::nbt::compound_tag::CompoundTag;
use crate::net::minecraft::nbt::tag::Tag;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::biome::biome_special_effects::{
    BiomeSpecialEffects, GrassColorModifier,
};
use crate::net::minecraft::world::level::dry_foliage_color::DryFoliageColor;
use crate::net::minecraft::world::level::foliage_color::FoliageColor;
use crate::net::minecraft::world::level::grass_color::GrassColor;
use crate::net::minecraft::world::level::levelgen::legacy_random_source::LegacyRandomSource;
use crate::net::minecraft::world::level::levelgen::synth::simplex_noise::SimplexNoise;

/// `BIOME_INFO_NOISE`: deprecated in Java, and still what the swamp's grass reads.
pub static BIOME_INFO_NOISE: LazyLock<SimplexNoise> =
    LazyLock::new(|| SimplexNoise::new_discarding_offset(&mut LegacyRandomSource::new(2345)));

/// A biome's climate and colours.
#[derive(Clone, Debug, PartialEq)]
pub struct Biome {
    climate_settings: ClimateSettings,
    special_effects: BiomeSpecialEffects,
}

impl Biome {
    pub fn new(climate_settings: ClimateSettings, special_effects: BiomeSpecialEffects) -> Biome {
        Biome {
            climate_settings,
            special_effects,
        }
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

    /// Temporary: `NETWORK_CODEC` over Gson's tree of a pack's JSON. Panics where the codec
    /// gives an error.
    pub fn from_network_json(json: &JsonElement) -> Biome {
        let object = json.get_as_json_object();
        let field = |name: &str| {
            object
                .get(name)
                .unwrap_or_else(|| panic!("No key {name} in {}", json.to_json_string()))
        };
        let climate_settings = ClimateSettings {
            has_precipitation: field("has_precipitation").get_as_boolean(),
            temperature: field("temperature").get_as_float(),
            temperature_modifier: object
                .get("temperature_modifier")
                .map_or(TemperatureModifier::NONE, |name| {
                    TemperatureModifier::by_name_or_panic(&name.get_as_string())
                }),
            downfall: field("downfall").get_as_float(),
        };
        let effects: &JsonObject = field("effects").get_as_json_object();
        let color = |name: &str| effects.get(name).map(Biome::json_color);
        let special_effects = BiomeSpecialEffects::new(
            color("water_color").unwrap_or_else(|| panic!("No key water_color in effects")),
            color("foliage_color"),
            color("dry_foliage_color"),
            color("grass_color"),
            effects
                .get("grass_color_modifier")
                .map_or(GrassColorModifier::NONE, |name| {
                    let name = name.get_as_string();
                    GrassColorModifier::by_name(&name)
                        .unwrap_or_else(|| panic!("Unknown element name: {name}"))
                }),
        );
        Biome::new(climate_settings, special_effects)
    }

    fn json_color(color: &JsonElement) -> i32 {
        let primitive = color.as_json_primitive().unwrap_or_else(|| {
            panic!(
                "a colour the temporary reader takes: {}",
                color.to_json_string()
            )
        });
        if primitive.is_string() {
            Biome::hex_color(&primitive.get_as_string())
        } else {
            primitive.get_as_int()
        }
    }

    /// Temporary: `NETWORK_CODEC` over the server's NBT. Panics where the codec gives an error.
    pub fn from_network_nbt(tag: &Tag) -> Biome {
        let Tag::Compound(compound) = tag else {
            panic!("Not a map: {tag:?}");
        };
        let number = |compound: &CompoundTag, name: &str| {
            compound
                .get(name)
                .and_then(Tag::as_numeric_tag)
                .unwrap_or_else(|| panic!("No number {name}"))
                .float_value()
        };
        let climate_settings = ClimateSettings {
            has_precipitation: compound
                .get("has_precipitation")
                .and_then(Tag::as_numeric_tag)
                .unwrap_or_else(|| panic!("No boolean has_precipitation"))
                .byte_value()
                != 0,
            temperature: number(compound, "temperature"),
            temperature_modifier: compound.get_string("temperature_modifier").map_or(
                TemperatureModifier::NONE,
                TemperatureModifier::by_name_or_panic,
            ),
            downfall: number(compound, "downfall"),
        };
        let effects = compound
            .get_compound("effects")
            .unwrap_or_else(|| panic!("No key effects"));
        let color = |name: &str| effects.get(name).map(Biome::nbt_color);
        let special_effects = BiomeSpecialEffects::new(
            color("water_color").unwrap_or_else(|| panic!("No key water_color in effects")),
            color("foliage_color"),
            color("dry_foliage_color"),
            color("grass_color"),
            effects
                .get_string("grass_color_modifier")
                .map_or(GrassColorModifier::NONE, |name| {
                    GrassColorModifier::by_name(name)
                        .unwrap_or_else(|| panic!("Unknown element name: {name}"))
                }),
        );
        Biome::new(climate_settings, special_effects)
    }

    fn nbt_color(color: &Tag) -> i32 {
        match color {
            Tag::String(string) => Biome::hex_color(string.value()),
            other => other
                .as_numeric_tag()
                .unwrap_or_else(|| panic!("a colour the temporary reader takes: {other:?}"))
                .int_value(),
        }
    }

    /// `ExtraCodecs.hexColor(6)`, made opaque: `#` and six hex digits.
    fn hex_color(string: &str) -> i32 {
        let digits = string
            .strip_prefix('#')
            .unwrap_or_else(|| panic!("Hex color must begin with #"));
        if digits.chars().count() != 6 {
            panic!(
                "Hex color is wrong size, expected 6 digits but got {}",
                digits.chars().count()
            );
        }
        let value = i64::from_str_radix(digits, 16)
            .unwrap_or_else(|_| panic!("Invalid color value: {string}"));
        ARGB::opaque(value as i32)
    }
}

/// `Biome.ClimateSettings`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClimateSettings {
    has_precipitation: bool,
    temperature: f32,
    temperature_modifier: TemperatureModifier,
    downfall: f32,
}

impl ClimateSettings {
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

impl TemperatureModifier {
    pub const VALUES: [TemperatureModifier; 2] =
        [TemperatureModifier::NONE, TemperatureModifier::FROZEN];

    fn by_name_or_panic(name: &str) -> TemperatureModifier {
        TemperatureModifier::VALUES
            .into_iter()
            .find(|modifier| modifier.get_serialized_name() == name)
            .unwrap_or_else(|| panic!("Unknown element name: {name}"))
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
    use crate::golden::{Decode, Golden, unhex};
    use crate::io::netty::buffer::byte_buf_input_stream::ByteBufInputStream;
    use crate::io::netty::buffer::unpooled;
    use crate::net::minecraft::client::multiplayer::registry_data_collector::RegistryDataCollector;
    use crate::net::minecraft::core::registries::registries::Registries;
    use crate::net::minecraft::core::registry_access::RegistryAccess;
    use crate::net::minecraft::core::registry_synchronization::PackedRegistryEntry;
    use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
    use crate::net::minecraft::nbt::nbt_io::NbtIo;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::util::string_representable::StringRepresentable;
    use crate::net::minecraft::world::level::color_map_color_util::tests::load_colormaps;

    /// The registries the client collects from the walk's registry data: the biomes by name, none
    /// sent whole, so each is read from the vanilla pack, like a known pack's.
    pub(crate) fn biome_registries(biomes: &Value) -> RegistryAccess {
        let entries: Vec<PackedRegistryEntry> = Vec::<String>::decode(biomes)
            .iter()
            .map(|name| PackedRegistryEntry::new(Identifier::parse(name), None))
            .collect();
        let mut collector = RegistryDataCollector::new();
        collector.append_contents(Registries::BIOME, &entries);
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
            biomes.value_biome(&biomes.get_or_throw(&key)).clone()
        };
        let json_cases = golden.cases("json");
        assert_eq!(
            json_cases.len(),
            golden.input("BIOMES").as_array().expect("names").len()
        );
        for (args, expected) in json_cases {
            assert_eq!(fields(&biome(&args[0])), expected[0], "{}", args[0]);
        }
        for (args, expected) in golden.cases("nbt") {
            assert_eq!(
                fields(&Biome::from_network_nbt(&read_nbt(&args[0]))),
                expected
            );
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
