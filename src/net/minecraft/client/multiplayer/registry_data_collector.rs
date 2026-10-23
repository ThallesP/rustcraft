//! net/minecraft/client/multiplayer/RegistryDataCollector.java
//!
//! What it collects is each registry's elements' identifiers in the order they came, which are
//! their network ids (`Registry`), and the biomes' values, which the client reads from the data
//! the server sent, or for an element of a pack it knows, from that pack (`RegistryDataLoader`
//! with `Biome.NETWORK_CODEC`). **Temporary, until DataFixerUpper's codecs come**: the biomes are
//! read by `Biome::from_network_nbt` and `from_network_json`, and the known pack is the vanilla
//! pack's `data/` (`KnownPacksManager.createResourceManager`, which only knows `vanilla`'s
//! elements here). Not ported yet: the other registries' values, and the tags (`appendTags`),
//! which come with `ClientboundUpdateTagsPacket`.

use std::path::Path;
use std::rc::Rc;

use crate::com::google::gson::json_parser::JsonParser;
use crate::net::minecraft::core::registries::built_in_registries::BuiltInRegistries;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::core::registry::Registry;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::core::registry_synchronization::PackedRegistryEntry;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::server::packs::pack_resources::PackResources;
use crate::net::minecraft::server::packs::pack_type::PackType;
use crate::net::minecraft::server::packs::resources::multi_pack_resource_manager::MultiPackResourceManager;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;
use crate::net::minecraft::server::packs::vanilla_pack_resources::VanillaPackResources;
use crate::net::minecraft::world::level::biome::biome::Biome;

/// The registries the server sends during configuration, as they come.
#[derive(Debug, Default)]
pub struct RegistryDataCollector {
    contents_collector: Option<ContentsCollector>,
}

impl RegistryDataCollector {
    pub fn new() -> RegistryDataCollector {
        RegistryDataCollector::default()
    }

    pub fn append_contents(&mut self, registry: &ResourceKey, entries: &[PackedRegistryEntry]) {
        self.contents_collector
            .get_or_insert_with(ContentsCollector::default)
            .append(registry, entries);
    }

    /// The registries the game protocol reads with: the client's static ones the port has
    /// (`ClientRegistryLayer.STATIC`, `BuiltInRegistries`), then the ones the server sent.
    pub fn collect_game_registries(&self) -> RegistryAccess {
        let received = self
            .contents_collector
            .iter()
            .flat_map(|collector| &collector.elements)
            .map(|(registry, entries)| {
                let keys = entries
                    .iter()
                    .map(|entry| ResourceKey::create(registry, entry.id().clone()))
                    .collect();
                if registry == &**Registries::BIOME {
                    let biomes = entries.iter().map(RegistryDataCollector::biome).collect();
                    Registry::with_biomes(registry.clone(), keys, biomes)
                } else {
                    Registry::new(registry.clone(), keys)
                }
            });
        let mut registries = BuiltInRegistries::registries();
        registries.extend(received);
        RegistryAccess::new(registries)
    }
}

impl RegistryDataCollector {
    /// Temporary: a biome from the server's data, or the vanilla pack's.
    pub(crate) fn biome(entry: &PackedRegistryEntry) -> Biome {
        match entry.data() {
            Some(data) => Biome::from_network_nbt(data),
            None => {
                let id = entry.id();
                let location = Identifier::from_namespace_and_path(
                    id.get_namespace(),
                    &format!("worldgen/biome/{}.json", id.get_path()),
                );
                let resource = VANILLA_DATA
                    .with(|data| data.get_resource(&location))
                    .unwrap_or_else(|| panic!("the known packs have no biome {id}"));
                let text = resource
                    .read_all_as_string()
                    .unwrap_or_else(|error| panic!("can't read the biome {id}: {error}"));
                let json = JsonParser::parse_strict(&text)
                    .unwrap_or_else(|error| panic!("the biome {id} isn't JSON: {error}"));
                Biome::from_network_json(&json)
            }
        }
    }
}

thread_local! {
    /// The vanilla pack's data, the known pack the client reads the server's elements from.
    static VANILLA_DATA: MultiPackResourceManager = {
        let vanilla = VanillaPackResources::create_client_pack(
            &VanillaPackResources::vanilla_jar_root(),
            Path::new("no-asset-index"),
        );
        let layers = vanilla
            .resource_layers()
            .iter()
            .map(|layer| layer.clone() as Rc<dyn PackResources>)
            .collect();
        MultiPackResourceManager::new(PackType::SERVER_DATA, layers)
    };
}

/// `RegistryDataCollector.ContentsCollector`: each registry's entries, in the order they came.
#[derive(Debug, Default)]
struct ContentsCollector {
    elements: Vec<(ResourceKey, Vec<PackedRegistryEntry>)>,
}

impl ContentsCollector {
    fn append(&mut self, registry: &ResourceKey, entries: &[PackedRegistryEntry]) {
        match self.elements.iter_mut().find(|(key, _)| key == registry) {
            Some((_, collected)) => collected.extend_from_slice(entries),
            None => self.elements.push((registry.clone(), entries.to_vec())),
        }
    }
}
