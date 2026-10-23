//! net/minecraft/client/multiplayer/RegistryDataCollector.java
//!
//! What it collects is each registry's elements in the order they came, which are their network
//! ids, loaded by `RegistryDataLoader` (`loadNewElementsAndTags`): from the data the server sent,
//! or for an element of a pack it knows, from that pack. The known pack is the vanilla pack's
//! `data/` (`KnownPacksManager.createResourceManager`, which only knows `vanilla`'s elements
//! here). Not ported yet: the tags (`appendTags`), which come with `ClientboundUpdateTagsPacket`,
//! and the layered access (`ClientRegistryLayer`), which is the static registries the port has,
//! then the loaded ones in the order they came.

use std::path::Path;
use std::rc::Rc;

use crate::net::minecraft::core::registries::built_in_registries::BuiltInRegistries;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::core::registry_synchronization::PackedRegistryEntry;
use crate::net::minecraft::resources::registry_data_loader::RegistryDataLoader;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::server::packs::pack_resources::PackResources;
use crate::net::minecraft::server::packs::pack_type::PackType;
use crate::net::minecraft::server::packs::resources::multi_pack_resource_manager::MultiPackResourceManager;
use crate::net::minecraft::server::packs::vanilla_pack_resources::VanillaPackResources;

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
    /// (`ClientRegistryLayer.STATIC`, `BuiltInRegistries`), then the ones the server sent, loaded
    /// with the static ones as their context. Panics where Java throws the loader's
    /// `ReportedException`.
    pub fn collect_game_registries(&self) -> RegistryAccess {
        let static_registries = RegistryAccess::new(BuiltInRegistries::registries());
        let received = match &self.contents_collector {
            Some(collector) => VANILLA_DATA.with(|known_data_source| {
                RegistryDataLoader::load(&collector.elements, known_data_source, &static_registries)
            }),
            None => Vec::new(),
        };
        let mut registries = BuiltInRegistries::registries();
        registries.extend(received);
        RegistryAccess::new(registries)
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

#[cfg(test)]
pub(crate) mod tests {
    use super::RegistryDataCollector;
    use crate::net::minecraft::core::registries::registries::Registries;
    use crate::net::minecraft::core::registry::Registry;
    use crate::net::minecraft::core::registry_synchronization::PackedRegistryEntry;
    use crate::net::minecraft::resources::identifier::Identifier;

    /// The vanilla dimension types' entries, as a vanilla server sends them: by name.
    pub(crate) fn dimension_type_entries() -> Vec<PackedRegistryEntry> {
        ["overworld", "overworld_caves", "the_end", "the_nether"]
            .into_iter()
            .map(|name| PackedRegistryEntry::new(Identifier::with_default_namespace(name), None))
            .collect()
    }

    /// The vanilla pack's dimension types, loaded like a known pack's elements.
    pub(crate) fn dimension_types() -> Registry {
        let mut collector = RegistryDataCollector::new();
        collector.append_contents(Registries::DIMENSION_TYPE, &dimension_type_entries());
        collector
            .collect_game_registries()
            .lookup_or_throw(Registries::DIMENSION_TYPE)
            .clone()
    }
}
