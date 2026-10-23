//! net/minecraft/resources/RegistryDataLoader.java, with NetworkRegistryLoadTask.java and
//! RegistryLoadTask.java's network path
//!
//! How the client makes the synchronized registries' elements from what the server sent
//! (`load` over `NetworkedRegistryData`): each element from its NBT when the server sent it, else
//! from the JSON of a pack both know (`findAndLoadFromResource`), decoded by the registry's codec
//! with registry ops whose context is the client's static registries and the registries being
//! loaded (`createContext`, whose getters hand out a holder for any key). Elements are registered
//! in the order they came, which are their ids.
//!
//! `SYNCHRONIZED_REGISTRIES` lists every one in Java's order. The ones whose elements' classes
//! are ported hold their values (`Registry::with_values`): biomes, wolf variants, painting
//! variants, dimension types, damage types and chat types. The others hold their keys, which is
//! all the game protocol's codecs need of them. Not ported: the tags (`registerTags`,
//! `TagLoader.loadTagsFromNetwork`, with `update_tags`), the validators (`RegistryValidator`, a
//! non-empty check on the variants), the loading from packs alone (`ResourceManagerRegistryLoadTask`,
//! the server's) and the executor: the elements load one after the other.
//!
//! An element that doesn't load is an error Java collects, logs with its cause
//! (`printFullDetailsToLog`) and throws as a `ReportedException` (`Failed to load registries due
//! to errors`, with each element's message) once every registry has loaded; here the log is
//! standard error's and the exception a panic with the same lines.

use std::sync::LazyLock;

use crate::com::google::gson::json_element::JsonElement;
use crate::com::mojang::serialization::codec::BoxedCodec;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::json_ops::JsonOps;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::core::registry::{Registry, RegistryValue};
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::core::registry_synchronization::PackedRegistryEntry;
use crate::net::minecraft::nbt::nbt_ops::NbtOps;
use crate::net::minecraft::nbt::tag::Tag;
use crate::net::minecraft::network::chat::chat_type::ChatType;
use crate::net::minecraft::resources::file_to_id_converter::FileToIdConverter;
use crate::net::minecraft::resources::registry_ops::RegistryOps;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;
use crate::net::minecraft::util::strict_json_parser::StrictJsonParser;
use crate::net::minecraft::world::damagesource::damage_type::DamageType;
use crate::net::minecraft::world::entity::animal::wolf::wolf_variant::WolfVariant;
use crate::net::minecraft::world::entity::decoration::painting::painting_variant::PaintingVariant;
use crate::net::minecraft::world::level::biome::biome::Biome;
use crate::net::minecraft::world::level::dimension::dimension_type::DimensionType;

/// What loads a registry's elements: its codec, for a class the port has.
type ElementLoader = fn(
    &LoadContext<'_>,
    &ResourceKey,
    &[PackedRegistryEntry],
) -> Result<Registry, Vec<LoadingError>>;

/// An element that didn't load: its key, the exception's message and its cause's.
struct LoadingError {
    element_key: ResourceKey,
    message: String,
    cause: String,
}

/// `RegistryDataLoader.RegistryData`: a registry, and how its elements load.
pub struct RegistryData {
    key: &'static LazyLock<ResourceKey>,
    loader: Option<ElementLoader>,
}

impl RegistryData {
    pub fn key(&self) -> &ResourceKey {
        self.key
    }
}

/// The context every element decodes with.
struct LoadContext<'a> {
    context: &'a RegistryAccess,
    registering: &'a [ResourceKey],
    known_data_source: &'a dyn ResourceManager,
}

macro_rules! registry_data {
    ($key:ident) => {
        RegistryData {
            key: Registries::$key,
            loader: None,
        }
    };
    ($key:ident, $class:ident :: $codec:ident) => {
        RegistryData {
            key: Registries::$key,
            loader: Some(|context, key, entries| {
                load_elements::<$class>(context, key, entries, &$class::$codec)
            }),
        }
    };
}

pub struct RegistryDataLoader;

impl RegistryDataLoader {
    /// `SYNCHRONIZED_REGISTRIES`, in Java's order.
    pub fn synchronized_registries() -> &'static [RegistryData] {
        static SYNCHRONIZED_REGISTRIES: LazyLock<Vec<RegistryData>> = LazyLock::new(|| {
            vec![
                registry_data!(BIOME, Biome::NETWORK_CODEC),
                registry_data!(CHAT_TYPE, ChatType::DIRECT_CODEC),
                registry_data!(TRIM_PATTERN),
                registry_data!(TRIM_MATERIAL),
                registry_data!(WOLF_VARIANT, WolfVariant::NETWORK_CODEC),
                registry_data!(WOLF_SOUND_VARIANT),
                registry_data!(PIG_VARIANT),
                registry_data!(PIG_SOUND_VARIANT),
                registry_data!(FROG_VARIANT),
                registry_data!(CAT_VARIANT),
                registry_data!(CAT_SOUND_VARIANT),
                registry_data!(COW_SOUND_VARIANT),
                registry_data!(COW_VARIANT),
                registry_data!(CHICKEN_SOUND_VARIANT),
                registry_data!(CHICKEN_VARIANT),
                registry_data!(ZOMBIE_NAUTILUS_VARIANT),
                registry_data!(PAINTING_VARIANT, PaintingVariant::DIRECT_CODEC),
                registry_data!(SULFUR_CUBE_ARCHETYPE),
                registry_data!(DIMENSION_TYPE, DimensionType::NETWORK_CODEC),
                registry_data!(DAMAGE_TYPE, DamageType::DIRECT_CODEC),
                registry_data!(BANNER_PATTERN),
                registry_data!(ENCHANTMENT),
                registry_data!(JUKEBOX_SONG),
                registry_data!(INSTRUMENT),
                registry_data!(TEST_ENVIRONMENT),
                registry_data!(TEST_INSTANCE),
                registry_data!(DIALOG),
                registry_data!(WORLD_CLOCK),
                registry_data!(TIMELINE),
                registry_data!(DECORATED_POT_PATTERN),
                registry_data!(BLOCK_TRANSFORMER),
                registry_data!(BLOCK_STATE_PROVIDER),
            ]
        });
        &SYNCHRONIZED_REGISTRIES
    }

    /// `load(entries, knownDataSource, contextRegistries, SYNCHRONIZED_REGISTRIES, executor)`: the
    /// registries the server sent elements of, each in the order its entries came, the others
    /// left out. A registry the port doesn't know is loaded as its keys.
    pub fn load(
        entries: &[(ResourceKey, Vec<PackedRegistryEntry>)],
        known_data_source: &dyn ResourceManager,
        context: &RegistryAccess,
    ) -> Vec<Registry> {
        let registering: Vec<ResourceKey> = RegistryDataLoader::synchronized_registries()
            .iter()
            .map(|data| data.key().clone())
            .collect();
        let load_context = LoadContext {
            context,
            registering: &registering,
            known_data_source,
        };
        let mut loading_errors = Vec::new();
        let mut registries = Vec::new();
        for (registry_key, elements) in entries {
            let loader = RegistryDataLoader::synchronized_registries()
                .iter()
                .find(|data| data.key() == registry_key)
                .and_then(|data| data.loader);
            let keys = || {
                elements
                    .iter()
                    .map(|entry| ResourceKey::create(registry_key, entry.id().clone()))
                    .collect()
            };
            match loader {
                Some(loader) => match loader(&load_context, registry_key, elements) {
                    Ok(registry) => registries.push(registry),
                    Err(errors) => loading_errors.extend(errors),
                },
                None => registries.push(Registry::new(registry_key.clone(), keys())),
            }
        }
        if !loading_errors.is_empty() {
            // logErrors: by registry, then element (ERROR_KEY_COMPARATOR).
            loading_errors.sort_by(|a, b| {
                (a.element_key.registry(), a.element_key.identifier())
                    .cmp(&(b.element_key.registry(), b.element_key.identifier()))
            });
            let mut details = String::new();
            let mut registry = None;
            for error in &loading_errors {
                if registry != Some(error.element_key.registry()) {
                    registry = Some(error.element_key.registry());
                    details.push_str(&format!(
                        "> Errors in registry {}:\n",
                        error.element_key.registry()
                    ));
                }
                details.push_str(&format!(
                    ">> Errors in element {}:\n{}\nCaused by: {}\n",
                    error.element_key.identifier(),
                    error.message,
                    error.cause
                ));
            }
            eprintln!("Registry loading errors:\n{details}");
            panic!(
                "Failed to load registries due to errors{}",
                loading_errors
                    .iter()
                    .map(|error| format!(
                        "\n\t\t{}/{}: {}",
                        error.element_key.registry(),
                        error.element_key.identifier(),
                        error.message
                    ))
                    .collect::<String>()
            );
        }
        registries
    }
}

/// `NetworkRegistryLoadTask.load` of one registry: each element from the server's NBT, or from
/// the known pack's JSON, then registered in order.
fn load_elements<T: RegistryValue + 'static>(
    load_context: &LoadContext<'_>,
    registry_key: &ResourceKey,
    entries: &[PackedRegistryEntry],
    codec: &'static LazyLock<BoxedCodec<T>>,
) -> Result<Registry, Vec<LoadingError>> {
    let nbt_ops = RegistryOps::create_loading(
        &NbtOps::INSTANCE,
        load_context.context,
        load_context.registering,
    );
    let json_ops = RegistryOps::create_loading(
        &JsonOps::INSTANCE,
        load_context.context,
        load_context.registering,
    );
    let known_data_path_converter = FileToIdConverter::registry(registry_key);
    let mut keys = Vec::with_capacity(entries.len());
    let mut values = Vec::with_capacity(entries.len());
    let mut errors = Vec::new();
    for entry in entries {
        let element_key = ResourceKey::create(registry_key, entry.id().clone());
        let value = match entry.data() {
            Some(network_contents) => {
                load_from_network(codec, &nbt_ops, &element_key, network_contents)
            }
            None => find_and_load_from_resource(
                codec,
                &json_ops,
                &element_key,
                &known_data_path_converter,
                load_context.known_data_source,
            ),
        };
        match value {
            Ok(value) => {
                keys.push(element_key);
                values.push(value);
            }
            Err((message, cause)) => errors.push(LoadingError {
                element_key,
                message,
                cause,
            }),
        }
    }
    if errors.is_empty() {
        Ok(Registry::with_values(
            registry_key.clone(),
            keys,
            T::wrap(values),
        ))
    } else {
        Err(errors)
    }
}

/// `getOrThrow()`'s value, or its exception's message.
fn get_or_throw<T>(result: DataResult<T>) -> Result<T, String> {
    match result {
        DataResult::Success(success) => Ok(success.into_value()),
        DataResult::Error(error) => Err(error.message().to_owned()),
    }
}

/// `PendingRegistration.loadFromNetwork`: the element's NBT through the codec, or the exception
/// Java wraps its failure in, and the failure.
fn load_from_network<T>(
    decoder: &BoxedCodec<T>,
    ops: &RegistryOps<'_, Tag>,
    element_key: &ResourceKey,
    contents: &Tag,
) -> Result<T, (String, String)> {
    get_or_throw(decoder.parse(ops, contents)).map_err(|cause| {
        (
            format!(
                "Failed to parse value {contents} for key {} from server",
                element_key.identifier()
            ),
            cause,
        )
    })
}

/// `PendingRegistration.findAndLoadFromResource` and `loadFromResource`: the element's file in the
/// known packs, read strictly and decoded.
fn find_and_load_from_resource<T>(
    decoder: &BoxedCodec<T>,
    ops: &RegistryOps<'_, JsonElement>,
    element_key: &ResourceKey,
    converter: &FileToIdConverter,
    resource_provider: &dyn ResourceManager,
) -> Result<T, (String, String)> {
    let resource_id = converter.id_to_file(element_key.identifier());
    let Some(resource) = resource_provider.get_resource(&resource_id) else {
        return Err((
            format!(
                "Failed to find resource {resource_id} for element {}",
                element_key.identifier()
            ),
            String::new(),
        ));
    };
    let parsed = resource
        .read_all_as_string()
        .map_err(|error| error.to_string())
        .and_then(|text| StrictJsonParser::parse(&text).map_err(|error| error.to_string()))
        .and_then(|json| get_or_throw(decoder.parse(ops, &json)));
    parsed.map_err(|cause| {
        (
            format!(
                "Failed to parse {} from pack {}",
                element_key.identifier(),
                resource.source_pack_id()
            ),
            cause,
        )
    })
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use crate::com::mojang::serialization::codec::BoxedCodec;
    use crate::com::mojang::serialization::data_result::DataResult;
    use crate::com::mojang::serialization::decoder::Decoder;
    use crate::golden::{Decode, Golden, unhex};
    use crate::io::netty::buffer::byte_buf_input_stream::ByteBufInputStream;
    use crate::io::netty::buffer::unpooled;
    use crate::net::minecraft::client::multiplayer::registry_data_collector::RegistryDataCollector;
    use crate::net::minecraft::core::holder_set::HolderSet;
    use crate::net::minecraft::core::registries::built_in_registries::BuiltInRegistries;
    use crate::net::minecraft::core::registry_access::RegistryAccess;
    use crate::net::minecraft::core::registry_synchronization::PackedRegistryEntry;
    use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
    use crate::net::minecraft::nbt::nbt_io::NbtIo;
    use crate::net::minecraft::nbt::nbt_ops::NbtOps;
    use crate::net::minecraft::nbt::tag::Tag;
    use crate::net::minecraft::network::chat::chat_type::ChatType;
    use crate::net::minecraft::network::chat::chat_type_decoration::ChatTypeDecoration;
    use crate::net::minecraft::network::chat::component::Component;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::resources::registry_ops::RegistryOps;
    use crate::net::minecraft::resources::resource_key::ResourceKey;
    use crate::net::minecraft::util::string_representable::StringRepresentable;
    use crate::net::minecraft::world::attribute::environment_attribute_map::EnvironmentAttributeMap;
    use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
    use crate::net::minecraft::world::damagesource::damage_type::DamageType;
    use crate::net::minecraft::world::entity::animal::wolf::wolf_variant::{
        AssetInfo, WolfVariant,
    };
    use crate::net::minecraft::world::entity::decoration::painting::painting_variant::PaintingVariant;
    use crate::net::minecraft::world::level::biome::biome::Biome;
    use crate::net::minecraft::world::level::color_map_color_util::tests::load_colormaps;
    use crate::net::minecraft::world::level::dimension::dimension_type::DimensionType;

    fn hex_f32(value: f32) -> String {
        format!("{:08x}", value.to_bits())
    }

    /// The map's attributes by name, sorted: vanilla's, but for the ones whose types aren't ported,
    /// which the port leaves out (`EnvironmentAttributes::UNPORTED`) and the check accounts for.
    fn attributes(map: &EnvironmentAttributeMap) -> Vec<String> {
        let mut names: Vec<String> = map
            .key_set()
            .map(|id| format!("minecraft:{}", EnvironmentAttributes::info(id).name))
            .collect();
        names.sort();
        names
    }

    fn holder_set(set: &HolderSet) -> Value {
        match set {
            HolderSet::Named(key) => json!(format!("#{}", key.location())),
            HolderSet::Direct(holders) => Value::Array(
                holders
                    .iter()
                    .map(|holder| {
                        json!(
                            holder
                                .unwrap_key()
                                .expect("a reference")
                                .identifier()
                                .to_string()
                        )
                    })
                    .collect(),
            ),
        }
    }

    fn decoration(decoration: &ChatTypeDecoration) -> Value {
        json!([
            decoration.translation_key(),
            decoration
                .parameters()
                .iter()
                .map(|parameter| parameter.get_serialized_name())
                .collect::<Vec<_>>(),
            decoration.style().to_string()
        ])
    }

    fn assets(info: &AssetInfo) -> Value {
        json!([
            info.wild().id().to_string(),
            info.wild().texture_path().to_string(),
            info.tame().id().to_string(),
            info.tame().texture_path().to_string(),
            info.angry().id().to_string(),
            info.angry().texture_path().to_string()
        ])
    }

    fn component(component: Option<&Component>) -> Value {
        component.map_or(Value::Null, |component| json!(component.to_string()))
    }

    /// The suite's fields of the element, and its attributes' names where it has some.
    fn fields(
        registries: &RegistryAccess,
        registry: &str,
        id: &str,
    ) -> (Value, Option<Vec<String>>) {
        let registry_key = ResourceKey::create_registry_key(Identifier::parse(registry));
        let lookup = registries.lookup_or_throw(&registry_key);
        let holder =
            lookup.get_or_throw(&ResourceKey::create(&registry_key, Identifier::parse(id)));
        match registry {
            "minecraft:worldgen/biome" => {
                let biome = lookup.value::<Biome>(&holder);
                let climate = biome.get_climate_settings();
                let effects = biome.get_special_effects();
                let fields = json!([
                    climate.has_precipitation(),
                    hex_f32(climate.temperature()),
                    climate.temperature_modifier().get_serialized_name(),
                    hex_f32(climate.downfall()),
                    effects.water_color(),
                    effects.foliage_color_override(),
                    effects.dry_foliage_color_override(),
                    effects.grass_color_override(),
                    effects.grass_color_modifier().get_serialized_name(),
                ]);
                (fields, Some(attributes(biome.get_attributes())))
            }
            "minecraft:chat_type" => {
                let chat_type = lookup.value::<ChatType>(&holder);
                (
                    json!([
                        decoration(chat_type.chat()),
                        decoration(chat_type.narration())
                    ]),
                    None,
                )
            }
            "minecraft:wolf_variant" => {
                let variant = lookup.value::<WolfVariant>(&holder);
                (
                    json!([assets(variant.adult_info()), assets(variant.baby_info())]),
                    None,
                )
            }
            "minecraft:painting_variant" => {
                let variant = lookup.value::<PaintingVariant>(&holder);
                (
                    json!([
                        variant.width(),
                        variant.height(),
                        variant.asset_id().to_string(),
                        component(variant.title()),
                        component(variant.author())
                    ]),
                    None,
                )
            }
            "minecraft:dimension_type" => {
                let t = lookup.value::<DimensionType>(&holder);
                let fields = json!([
                    t.has_fixed_time(),
                    t.has_sky_light(),
                    t.has_ceiling(),
                    t.has_ender_dragon_fight(),
                    format!("{:016x}", t.coordinate_scale().to_bits()),
                    t.min_y(),
                    t.height(),
                    t.logical_height(),
                    holder_set(t.infiniburn()),
                    hex_f32(t.ambient_light()),
                    t.monster_spawn_light_test().to_string(),
                    t.monster_spawn_block_light_limit(),
                    t.skybox().get_serialized_name(),
                    t.cardinal_light_type().get_serialized_name(),
                    "attributes",
                    holder_set(t.timelines()),
                    t.default_clock().map(|clock| {
                        clock
                            .unwrap_key()
                            .expect("a reference")
                            .identifier()
                            .to_string()
                    }),
                ]);
                (fields, Some(attributes(t.attributes())))
            }
            "minecraft:damage_type" => {
                let t = lookup.value::<DamageType>(&holder);
                (
                    json!([
                        t.msg_id(),
                        t.scaling().get_serialized_name(),
                        hex_f32(t.exhaustion()),
                        t.effects().get_serialized_name(),
                        t.death_message_type().get_serialized_name()
                    ]),
                    None,
                )
            }
            other => panic!("the suite has no fields for {other}"),
        }
    }

    /// Vanilla's fields against the port's: the attributes' names in their place (the biome's
    /// last, the dimension type's 15th) as `attributes` accounts for them.
    fn check(registries: &RegistryAccess, args: &[Value], expected: &Value, how: &str) {
        let (registry, id) = (
            args[0].as_str().expect("a registry"),
            args[1].as_str().expect("an id"),
        );
        let (actual, attribute_names) = fields(registries, registry, id);
        let mut expected = expected.clone();
        if let Some(names) = attribute_names {
            let index = if registry == "minecraft:dimension_type" {
                14
            } else {
                9
            };
            let vanilla = Vec::<String>::decode(&expected[index]);
            let (ported, unported): (Vec<String>, Vec<String>) =
                vanilla.into_iter().partition(|name| {
                    !EnvironmentAttributes::UNPORTED
                        .contains(&name.trim_start_matches("minecraft:"))
                });
            assert!(
                unported.iter().all(|name| name.starts_with("minecraft:")),
                "{registry} {id}: {unported:?}"
            );
            assert_eq!(names, ported, "{how}: {registry} {id}'s attributes");
            if registry == "minecraft:dimension_type" {
                expected[index] = json!("attributes");
            } else {
                expected.as_array_mut().expect("fields").pop();
            }
        }
        assert_eq!(actual, expected, "{how}: {registry} {id}");
    }

    fn read_nbt(hex: &Value) -> Tag {
        let mut buffer = unpooled::wrapped_buffer(unhex(hex.as_str().expect("hex")));
        NbtIo::read_any_tag(
            &mut ByteBufInputStream::new(&mut buffer),
            &mut NbtAccounter::default_quota(),
        )
    }

    /// The registries the client loads from the walk's registry data: from the vanilla pack, or
    /// with each element of the suite's registries sent as NBT.
    fn load(
        registries: &[(Identifier, Vec<Identifier>)],
        sent: &[(Vec<Value>, Value)],
    ) -> RegistryAccess {
        let mut collector = RegistryDataCollector::new();
        for (registry, ids) in registries {
            let registry_key = ResourceKey::create_registry_key(registry.clone());
            let entries: Vec<PackedRegistryEntry> = ids
                .iter()
                .map(|id| {
                    let data = sent
                        .iter()
                        .find(|(args, _)| {
                            args[0] == json!(registry.to_string())
                                && args[1] == json!(id.to_string())
                        })
                        .map(|(args, _)| read_nbt(&args[2]));
                    PackedRegistryEntry::new(id.clone(), data)
                })
                .collect();
            collector.append_contents(&registry_key, &entries);
        }
        collector.collect_game_registries()
    }

    fn error_of<T>(codec: &BoxedCodec<T>, ops: &RegistryOps<'_, Tag>, tag: &Tag) -> Value {
        match codec.parse(ops, tag) {
            DataResult::Success(_) => Value::Null,
            DataResult::Error(error) => json!(error.message()),
        }
    }

    /// The walk's synchronized registries loaded like vanilla's client loads them: from the
    /// vanilla pack (`json`) and from what a server sends of its own elements (`nbt`), every
    /// element of the registries whose classes are ported, and the codecs' errors.
    #[test]
    fn loads_the_synchronized_registries_like_vanilla() {
        load_colormaps();
        let golden = Golden::load("net.minecraft.resources.RegistryDataLoader");
        let registries: Vec<(Identifier, Vec<Identifier>)> =
            Vec::<(String, Vec<String>)>::decode(&golden.input("REGISTRIES"))
                .into_iter()
                .map(|(registry, ids)| {
                    (
                        Identifier::parse(&registry),
                        ids.iter().map(|id| Identifier::parse(id)).collect(),
                    )
                })
                .collect();

        let from_pack = load(&registries, &[]);
        let json_cases = golden.cases("json");
        assert!(json_cases.len() > 100);
        for (args, expected) in &json_cases {
            check(&from_pack, args, expected, "json");
        }

        let sent = golden.cases("nbt");
        assert_eq!(sent.len(), json_cases.len());
        let from_network = load(&registries, &sent);
        for (args, expected) in &sent {
            check(&from_network, args, expected, "nbt");
        }

        // The loading context: the static registries, and every synchronized one being loaded.
        let context = RegistryAccess::new(BuiltInRegistries::registries());
        let registering: Vec<ResourceKey> = super::RegistryDataLoader::synchronized_registries()
            .iter()
            .map(|data| data.key().clone())
            .collect();
        let ops = RegistryOps::create_loading(&NbtOps::INSTANCE, &context, &registering);
        for (args, expected) in golden.cases("errors") {
            let tag = read_nbt(&args[1]);
            let actual = match args[0].as_str().expect("a registry") {
                "minecraft:worldgen/biome" => error_of(Biome::NETWORK_CODEC, &ops, &tag),
                "minecraft:chat_type" => error_of(ChatType::DIRECT_CODEC, &ops, &tag),
                "minecraft:wolf_variant" => error_of(WolfVariant::NETWORK_CODEC, &ops, &tag),
                "minecraft:painting_variant" => error_of(PaintingVariant::DIRECT_CODEC, &ops, &tag),
                "minecraft:dimension_type" => error_of(DimensionType::NETWORK_CODEC, &ops, &tag),
                "minecraft:damage_type" => error_of(DamageType::DIRECT_CODEC, &ops, &tag),
                other => panic!("no codec for {other}"),
            };
            assert_eq!(actual, expected, "{}: {tag}", args[0]);
        }
        golden.finish();
    }
}
