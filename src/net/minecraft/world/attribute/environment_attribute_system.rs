//! net/minecraft/world/attribute/EnvironmentAttributeSystem.java
//!
//! A level's environment attributes: each one's layers, in the order they were added, over its
//! default value. The leading constant layers fold into a base value when the system is built,
//! and an attribute no positional layer changes is worked out once a tick (`getDimensionValue`,
//! its cache in cells until `invalidateTickCache`).
//!
//! A client level's system is `for_client_level`: `addDefaultLayers` (the dimension type's
//! attributes, its biomes', its timelines', then the weather's where the level has weather),
//! then `ClientLevel.addEnvironmentAttributeLayers`' lightning flash, all from the vanilla pack's
//! data, which the client reads for the registries a server syncs from its known pack: the
//! dimension type's `data/<ns>/dimension_type/<path>.json`, each of the synchronized biome
//! registry's `worldgen/biome/<path>.json`, and the timelines its `timelines` tag lists. A level
//! answers what the layers read of it (`LayerLevel`); then
//!
//! ```ignore
//! let sky_color: Vector3f = system.get_value(&EnvironmentAttributes::SKY_COLOR, &pos, None, &level);
//! ```
//!
//! and a camera reads them through an `EnvironmentAttributeProbe`, which blends the biomes around
//! it. Java's builder holds the layers of each attribute in a `HashMap`, and the system its
//! samplers in another, whose order nothing reads; here both are by the attribute's place.
//!
//! Not ported: `EnvironmentAttributeReader`'s overloads (a `BlockPos`, its center), and the
//! testing helpers `getConstantBaseValue` and `isAffectedByPosition`. The lightning flash's two
//! layers are `ClientLevel`'s lambdas, which the goldens can't reach; they're read, not checked.

use std::cell::{Cell, RefCell};
use std::fs;
use std::path::PathBuf;

use crate::com::google::gson::json_element::JsonElement;
use crate::com::google::gson::json_parser::JsonParser;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::server::packs::vanilla_pack_resources::VanillaPackResources;
use crate::net::minecraft::world::attribute::attribute_type::{AttributeValue, Value};
use crate::net::minecraft::world::attribute::environment_attribute::EnvironmentAttribute;
use crate::net::minecraft::world::attribute::environment_attribute_layer::{
    EnvironmentAttributeLayer, LayerLevel, Positional, TimeBased,
};
use crate::net::minecraft::world::attribute::environment_attribute_map::EnvironmentAttributeMap;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::net::minecraft::world::attribute::spatial_attribute_interpolator::SpatialAttributeInterpolator;
use crate::net::minecraft::world::attribute::weather_attributes::WeatherAttributes;
use crate::net::minecraft::world::phys::vec3::Vec3;
use crate::net::minecraft::world::timeline::timeline::Timeline;

pub struct EnvironmentAttributeSystem {
    // Each attribute's sampler, by its place; none for an attribute no layer changes.
    attribute_samplers: Vec<Option<ValueSampler>>,
    // Each biome of the registry and its attribute map: what the biome layers read.
    biomes: Vec<(Holder, EnvironmentAttributeMap)>,
}

impl EnvironmentAttributeSystem {
    pub fn builder() -> Builder {
        Builder {
            layers_by_attribute: (0..EnvironmentAttributes::count())
                .map(|_| Vec::new())
                .collect(),
            biomes: Vec::new(),
        }
    }

    /// `ClientLevel`'s system, for a level of the dimension type whose synchronized biome
    /// registry has `biomes`: the default layers, then the lightning flash's. `can_have_weather`
    /// is the level's `canHaveWeather()`. `Err` for data that doesn't decode.
    pub fn for_client_level(
        dimension_type: &Identifier,
        biomes: &[ResourceKey],
        can_have_weather: bool,
    ) -> Result<EnvironmentAttributeSystem, String> {
        let mut builder = EnvironmentAttributeSystem::builder();
        builder.add_default_layers(dimension_type, biomes, can_have_weather)?;
        // ClientLevel.addEnvironmentAttributeLayers.
        builder.add_time_based_layer(
            EnvironmentAttributes::SKY_COLOR.id(),
            TimeBased::SkyFlashColor,
        );
        builder.add_time_based_layer(
            EnvironmentAttributes::SKY_LIGHT_FACTOR.id(),
            TimeBased::SkyFlashFactor,
        );
        Ok(builder.build())
    }

    pub fn invalidate_tick_cache(&self) {
        for sampler in self.attribute_samplers.iter().flatten() {
            sampler.invalidate_tick_cache();
        }
    }

    /// `getDimensionValue`: the value without the layers of a position.
    pub fn get_dimension_value<V: Value>(
        &self,
        attribute: &EnvironmentAttribute<V>,
        level: &dyn LayerLevel,
    ) -> V {
        V::from_attribute_value(self.get_dimension_value_of(attribute.id(), level))
    }

    pub fn get_dimension_value_of(
        &self,
        attribute: usize,
        level: &dyn LayerLevel,
    ) -> AttributeValue {
        match &self.attribute_samplers[attribute] {
            None => EnvironmentAttributes::info(attribute).default_value,
            Some(sampler) => sampler.get_dimension_value(level),
        }
    }

    /// `getValue(attribute, pos, biomeInterpolator)`: the biomes blended by the interpolator's
    /// weights where it's given and the attribute is spatially interpolated, or the biome at the
    /// position.
    pub fn get_value<V: Value>(
        &self,
        attribute: &EnvironmentAttribute<V>,
        pos: &Vec3,
        biome_interpolator: Option<&SpatialAttributeInterpolator>,
        level: &dyn LayerLevel,
    ) -> V {
        V::from_attribute_value(self.get_value_of(attribute.id(), pos, biome_interpolator, level))
    }

    pub fn get_value_of(
        &self,
        attribute: usize,
        pos: &Vec3,
        biome_interpolator: Option<&SpatialAttributeInterpolator>,
        level: &dyn LayerLevel,
    ) -> AttributeValue {
        match &self.attribute_samplers[attribute] {
            None => EnvironmentAttributes::info(attribute).default_value,
            Some(sampler) => {
                if !sampler.is_affected_by_position {
                    sampler.get_dimension_value(level)
                } else {
                    sampler.compute_value_positional(pos, biome_interpolator, level, &self.biomes)
                }
            }
        }
    }
}

/// The vanilla pack's data file of an element of a registry directory.
fn pack_json(id: &Identifier, directory: &str) -> Result<JsonElement, String> {
    let path: PathBuf = VanillaPackResources::vanilla_jar_root()
        .join("data")
        .join(id.get_namespace())
        .join(directory)
        .join(format!("{}.json", id.get_path()));
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("can't read {}: {error}", path.display()))?;
    JsonParser::parse_strict(&text)
}

/// A dimension type's `timelines`, a `HolderSet` of timelines: a tag (`#ns:path`, its file's
/// values in order, tags within it in their place), one timeline or a list of them.
fn timelines_of(json: &JsonElement) -> Result<Vec<Identifier>, String> {
    if let Some(array) = json.as_json_array() {
        return Ok(array
            .iter()
            .map(|element| Identifier::parse(&element.get_as_string()))
            .collect());
    }
    let name = json.get_as_string();
    match name.strip_prefix('#') {
        Some(tag) => {
            let tag = Identifier::parse(tag);
            let file = pack_json(&tag, "tags/timeline")?;
            let mut timelines = Vec::new();
            for value in file
                .get_as_json_object()
                .get("values")
                .and_then(JsonElement::as_json_array)
                .ok_or("a tag file has values")?
            {
                timelines.extend(timelines_of(value)?);
            }
            Ok(timelines)
        }
        None => Ok(vec![Identifier::parse(&name)]),
    }
}

/// `EnvironmentAttributeSystem.Builder`.
pub struct Builder {
    layers_by_attribute: Vec<Vec<EnvironmentAttributeLayer>>,
    biomes: Vec<(Holder, EnvironmentAttributeMap)>,
}

impl Builder {
    /// `addDefaultLayers(level)`: the static ones (the dimension type's attributes, then the
    /// biomes'), then the dynamic ones (the timelines the dimension type's `timelines` list, in
    /// order, then the weather's where the level has weather), from the vanilla pack's data.
    pub fn add_default_layers(
        &mut self,
        dimension_type: &Identifier,
        biomes: &[ResourceKey],
        can_have_weather: bool,
    ) -> Result<&mut Self, String> {
        let dimension = pack_json(dimension_type, "dimension_type")?;
        let dimension = dimension.get_as_json_object();
        // addDimensionLayer.
        if let Some(attributes) = dimension.get("attributes") {
            self.add_constant_layer(&EnvironmentAttributeMap::from_json(
                attributes.get_as_json_object(),
            )?);
        }
        // addBiomeLayer.
        let mut biome_maps = Vec::new();
        for biome in biomes {
            let data = pack_json(biome.identifier(), "worldgen/biome")?;
            let attributes = match data.get_as_json_object().get("attributes") {
                Some(attributes) => {
                    EnvironmentAttributeMap::from_json(attributes.get_as_json_object())?
                }
                None => EnvironmentAttributeMap::empty(),
            };
            biome_maps.push((Holder::Reference(biome.clone()), attributes));
        }
        self.add_biome_layer(biome_maps);
        // addDynamicLayers.
        if let Some(timelines) = dimension.get("timelines") {
            for timeline in timelines_of(timelines)? {
                self.add_timeline_layer(&Timeline::from_json(
                    pack_json(&timeline, "timeline")?.get_as_json_object(),
                )?);
            }
        }
        if can_have_weather {
            WeatherAttributes::add_builtin_layers(self);
        }
        Ok(self)
    }

    /// `addConstantLayer(EnvironmentAttributeMap)`: each of the map's entries.
    pub fn add_constant_layer(&mut self, attribute_map: &EnvironmentAttributeMap) -> &mut Self {
        for attribute in attribute_map.key_set() {
            let entry = *attribute_map.get(attribute).expect("a key of the map");
            self.add_layer(attribute, EnvironmentAttributeLayer::Constant(entry));
        }
        self
    }

    pub fn add_time_based_layer(&mut self, attribute: usize, layer: TimeBased) -> &mut Self {
        self.add_layer(attribute, EnvironmentAttributeLayer::TimeBased(layer))
    }

    pub fn add_positional_layer(&mut self, attribute: usize, layer: Positional) -> &mut Self {
        self.add_layer(attribute, EnvironmentAttributeLayer::Positional(layer))
    }

    fn add_layer(&mut self, attribute: usize, layer: EnvironmentAttributeLayer) -> &mut Self {
        self.layers_by_attribute[attribute].push(layer);
        self
    }

    /// `addBiomeLayer`: a positional layer for each attribute any of the registry's biomes has,
    /// over the biomes' maps, each biome with its own.
    pub fn add_biome_layer(&mut self, biomes: Vec<(Holder, EnvironmentAttributeMap)>) -> &mut Self {
        let mut attributes_provided_by_biomes: Vec<usize> = Vec::new();
        for (_, attributes) in &biomes {
            for attribute in attributes.key_set() {
                if !attributes_provided_by_biomes.contains(&attribute) {
                    attributes_provided_by_biomes.push(attribute);
                }
            }
        }
        for attribute in attributes_provided_by_biomes {
            self.add_positional_layer(attribute, Positional::Biome);
        }
        self.biomes = biomes;
        self
    }

    /// `addTimelineLayer`: a layer for each of the timeline's tracks.
    pub fn add_timeline_layer(&mut self, timeline: &Timeline) -> &mut Self {
        let attributes: Vec<usize> = timeline.attributes().collect();
        for attribute in attributes {
            self.add_time_based_layer(
                attribute,
                TimeBased::Track(timeline.create_track_sampler(attribute)),
            );
        }
        self
    }

    pub fn build(self) -> EnvironmentAttributeSystem {
        let attribute_samplers = self
            .layers_by_attribute
            .into_iter()
            .enumerate()
            .map(|(attribute, layers)| {
                (!layers.is_empty()).then(|| Builder::bake_layer_sampler(attribute, layers))
            })
            .collect();
        EnvironmentAttributeSystem {
            attribute_samplers,
            biomes: self.biomes,
        }
    }

    /// The leading constant layers applied to the default value, which is the sampler's base.
    fn bake_layer_sampler(
        attribute: usize,
        layers: Vec<EnvironmentAttributeLayer>,
    ) -> ValueSampler {
        let mut constant_base_value = EnvironmentAttributes::info(attribute).default_value;
        let mut layers = layers.into_iter().peekable();
        while let Some(EnvironmentAttributeLayer::Constant(entry)) = layers.peek() {
            constant_base_value = entry.apply_modifier(constant_base_value);
            layers.next();
        }

        let layers: Vec<EnvironmentAttributeLayer> = layers.collect();
        let is_affected_by_position = layers
            .iter()
            .any(|layer| matches!(layer, EnvironmentAttributeLayer::Positional(_)));
        ValueSampler {
            attribute,
            base_value: constant_base_value,
            layers,
            is_affected_by_position,
            cached_tick_value: RefCell::new(None),
            cache_tick_id: Cell::new(0),
        }
    }
}

/// `EnvironmentAttributeSystem.ValueSampler`.
struct ValueSampler {
    attribute: usize,
    base_value: AttributeValue,
    layers: Vec<EnvironmentAttributeLayer>,
    is_affected_by_position: bool,
    cached_tick_value: RefCell<Option<AttributeValue>>,
    cache_tick_id: Cell<i32>,
}

impl ValueSampler {
    fn invalidate_tick_cache(&self) {
        *self.cached_tick_value.borrow_mut() = None;
        self.cache_tick_id
            .set(self.cache_tick_id.get().wrapping_add(1));
    }

    fn get_dimension_value(&self, level: &dyn LayerLevel) -> AttributeValue {
        if let Some(cached) = *self.cached_tick_value.borrow() {
            return cached;
        }

        let result = self.compute_value_not_positional(level);
        *self.cached_tick_value.borrow_mut() = Some(result);
        result
    }

    fn compute_value_positional(
        &self,
        pos: &Vec3,
        biome_interpolator: Option<&SpatialAttributeInterpolator>,
        level: &dyn LayerLevel,
        biomes: &[(Holder, EnvironmentAttributeMap)],
    ) -> AttributeValue {
        let mut result = self.base_value;

        for layer in &self.layers {
            result =
                match layer {
                    EnvironmentAttributeLayer::Constant(entry) => entry.apply_modifier(result),
                    EnvironmentAttributeLayer::TimeBased(time_based) => time_based
                        .apply_time_based(self.attribute, result, self.cache_tick_id.get(), level),
                    EnvironmentAttributeLayer::Positional(positional) => positional
                        .apply_positional(
                            self.attribute,
                            result,
                            pos,
                            biome_interpolator,
                            level,
                            biomes,
                        ),
                };
        }

        EnvironmentAttributes::info(self.attribute).sanitize_value(result)
    }

    fn compute_value_not_positional(&self, level: &dyn LayerLevel) -> AttributeValue {
        let mut result = self.base_value;

        for layer in &self.layers {
            result =
                match layer {
                    EnvironmentAttributeLayer::Constant(entry) => entry.apply_modifier(result),
                    EnvironmentAttributeLayer::TimeBased(time_based) => time_based
                        .apply_time_based(self.attribute, result, self.cache_tick_id.get(), level),
                    EnvironmentAttributeLayer::Positional(_) => result,
                };
        }

        EnvironmentAttributes::info(self.attribute).sanitize_value(result)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use std::fs;

    use serde_json::{Value, json};

    use super::EnvironmentAttributeSystem;
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::core::registries::registries::Registries;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::resources::resource_key::ResourceKey;
    use crate::net::minecraft::server::packs::vanilla_pack_resources::VanillaPackResources;
    use crate::net::minecraft::world::attribute::attribute_type::AttributeValue;
    use crate::net::minecraft::world::attribute::environment_attribute_layer::LayerLevel;
    use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
    use crate::net::minecraft::world::attribute::gaussian_sampler::GaussianSampler;
    use crate::net::minecraft::world::attribute::spatial_attribute_interpolator::SpatialAttributeInterpolator;
    use crate::net::minecraft::world::phys::vec3::Vec3;

    /// `AttributeSuites.MIXED_BIOMES`.
    const MIXED_BIOMES: [&str; 14] = [
        "plains",
        "desert",
        "swamp",
        "cherry_grove",
        "pale_garden",
        "nether_wastes",
        "crimson_forest",
        "basalt_deltas",
        "the_end",
        "deep_dark",
        "mushroom_fields",
        "snowy_plains",
        "warm_ocean",
        "soul_sand_valley",
    ];

    fn biome(name: &str) -> Holder {
        Holder::Reference(ResourceKey::create(
            Registries::BIOME,
            Identifier::with_default_namespace(name),
        ))
    }

    /// The suites' level: `AttributeSuites.Clock`, `Weather` and `biomeManager`'s source, one
    /// biome everywhere or "mixed" by a hash of the quart position. No lightning.
    pub(crate) struct TestLevel {
        pub(crate) ticks: Cell<i64>,
        pub(crate) rain: Cell<f32>,
        pub(crate) thunder: Cell<f32>,
        biomes: Vec<Holder>,
    }

    impl TestLevel {
        pub(crate) fn new(source: &str) -> TestLevel {
            let biomes = if source == "mixed" {
                MIXED_BIOMES.iter().map(|name| biome(name)).collect()
            } else {
                vec![biome(source)]
            };
            TestLevel {
                ticks: Cell::new(0),
                rain: Cell::new(0.0),
                thunder: Cell::new(0.0),
                biomes,
            }
        }

        /// `AttributeSuites.mixedIndex`.
        fn mixed_index(quart_x: i32, quart_y: i32, quart_z: i32) -> usize {
            let mut h = (quart_x >> 1).wrapping_mul(73856093)
                ^ (quart_y >> 2).wrapping_mul(19349663)
                ^ (quart_z >> 1).wrapping_mul(83492791);
            h ^= ((h as u32) >> 13) as i32;
            h = h.wrapping_mul(0x5bd1e995);
            h ^= ((h as u32) >> 15) as i32;
            h.rem_euclid(MIXED_BIOMES.len() as i32) as usize
        }
    }

    impl LayerLevel for TestLevel {
        fn clock_total_ticks(&self, _clock: &Holder) -> i64 {
            self.ticks.get()
        }

        fn rain_level(&self) -> f32 {
            self.rain.get()
        }

        fn thunder_level(&self) -> f32 {
            self.thunder.get()
        }

        fn sky_flash_time(&self) -> i32 {
            0
        }

        fn get_noise_biome_at_quart(&self, quart_x: i32, quart_y: i32, quart_z: i32) -> Holder {
            if self.biomes.len() == 1 {
                self.biomes[0].clone()
            } else {
                self.biomes[TestLevel::mixed_index(quart_x, quart_y, quart_z)].clone()
            }
        }
    }

    /// Every biome of the vanilla pack, the suites' registry.
    fn all_biomes() -> Vec<ResourceKey> {
        let directory =
            VanillaPackResources::vanilla_jar_root().join("data/minecraft/worldgen/biome");
        let mut names: Vec<String> = fs::read_dir(&directory)
            .expect("setup_vanilla.py extracts the vanilla pack")
            .map(|entry| {
                let path = entry.expect("a directory entry").path();
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .expect("a file name")
                    .to_owned()
            })
            .collect();
        names.sort();
        names
            .iter()
            .map(|name| {
                ResourceKey::create(Registries::BIOME, Identifier::with_default_namespace(name))
            })
            .collect()
    }

    /// `AttributeSuites.system`: a level of the dimension type's default layers, over every biome
    /// of the pack.
    pub(crate) fn system(dimension_type: &str) -> EnvironmentAttributeSystem {
        // AttributeSuites.LEVELS: only the overworld has weather.
        let can_have_weather = dimension_type == "overworld";
        let mut builder = EnvironmentAttributeSystem::builder();
        builder
            .add_default_layers(
                &Identifier::with_default_namespace(dimension_type),
                &all_biomes(),
                can_have_weather,
            )
            .expect("the vanilla pack's data decodes");
        builder.build()
    }

    /// `AttributeSuites.encode`.
    pub(crate) fn encode(value: AttributeValue) -> Value {
        match value {
            AttributeValue::Boolean(value) => value.encode(),
            AttributeValue::Float(value) => value.encode(),
            AttributeValue::Rgb(value) => value.encode(),
            AttributeValue::Argb(value) => value.encode(),
            AttributeValue::MoonPhase(value) => Value::from(value.get_serialized_name()),
            other => panic!("not an attribute's value: {other:?}"),
        }
    }

    pub(crate) fn attribute_id(name: &str) -> usize {
        EnvironmentAttributes::by_name(&Identifier::parse(name))
            .expect("a registered attribute")
            .expect("a ported attribute")
    }

    /// Every ported attribute's value, at the biome at the position and blended from the biomes
    /// around it, by systems built like a level's over the vanilla pack (`AttributeSuites`'
    /// `SystemSuite`).
    #[test]
    fn values_match_vanilla() {
        let golden = Golden::load("net.minecraft.world.attribute.EnvironmentAttributeSystem");
        let attributes: Vec<usize> = Vec::<String>::decode(&golden.input("ATTRIBUTES"))
            .iter()
            .map(|name| attribute_id(name))
            .collect();
        assert_eq!(attributes.len(), EnvironmentAttributes::count());
        let systems: RefCell<HashMap<(String, String), (EnvironmentAttributeSystem, TestLevel)>> =
            RefCell::new(HashMap::new());
        golden.check(
            "getValue",
            |dimension_type: String,
             source: String,
             ticks: i64,
             rain: f32,
             thunder: f32,
             x: f64,
             y: f64,
             z: f64| {
                let mut systems = systems.borrow_mut();
                let (system, level) = systems
                    .entry((dimension_type.clone(), source.clone()))
                    .or_insert_with(|| (system(&dimension_type), TestLevel::new(&source)));
                level.ticks.set(ticks);
                level.rain.set(rain);
                level.thunder.set(thunder);
                system.invalidate_tick_cache();
                let pos = Vec3::new(x, y, z);
                let mut interpolator = SpatialAttributeInterpolator::new();
                GaussianSampler::sample(
                    &Vec3::new(x * 0.25, y * 0.25, z * 0.25),
                    |x, y, z| level.get_noise_biome_at_quart(x, y, z),
                    |weight, biome| {
                        interpolator.accumulate(weight, &biome);
                    },
                );
                let plain: Vec<Value> = attributes
                    .iter()
                    .map(|&attribute| encode(system.get_value_of(attribute, &pos, None, level)))
                    .collect();
                let interpolated: Vec<Value> = attributes
                    .iter()
                    .map(|&attribute| {
                        encode(system.get_value_of(attribute, &pos, Some(&interpolator), level))
                    })
                    .collect();
                json!([plain, interpolated])
            },
        );
        golden.finish();
    }

    /// `for_client_level` (the dimension type's tag of timelines, the level's own biomes, the
    /// lightning flash's layers) gives a flat world of plains, with no lightning, the values the
    /// golden-checked system gives at the walk's eye.
    #[test]
    fn client_level_systems_read_the_pack_like_the_suites() {
        let client = EnvironmentAttributeSystem::for_client_level(
            &Identifier::with_default_namespace("overworld"),
            &[ResourceKey::create(
                Registries::BIOME,
                Identifier::with_default_namespace("plains"),
            )],
            true,
        )
        .expect("the vanilla pack's data decodes");
        let checked = system("overworld");
        let level = TestLevel::new("plains");
        let pos = Vec3::new(0.5, -60.0 + f64::from(1.62f32), 0.5);
        for ticks in [0, 1000, 6000, 12000, 13000, 18000, 23500] {
            level.ticks.set(ticks);
            client.invalidate_tick_cache();
            checked.invalidate_tick_cache();
            for attribute in 0..EnvironmentAttributes::count() {
                assert_eq!(
                    encode(client.get_value_of(attribute, &pos, None, &level)),
                    encode(checked.get_value_of(attribute, &pos, None, &level)),
                    "{} at {ticks}",
                    EnvironmentAttributes::info(attribute).name
                );
            }
        }
        let sky_color = client.get_value(&EnvironmentAttributes::SKY_COLOR, &pos, None, &level);
        assert!(sky_color.z > 0.0);
    }
}
