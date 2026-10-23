//! net/minecraft/client/multiplayer/RegistryDataCollector.java
//!
//! Not ported yet: the elements' values, which the client loads from the data the server sent or
//! from its known packs (`RegistryDataLoader`), and the tags (`appendTags`), which come with
//! `ClientboundUpdateTagsPacket`. What it collects is each registry's elements' identifiers in
//! the order they came, which are their network ids (`Registry`).

use crate::net::minecraft::core::registries::built_in_registries::BuiltInRegistries;
use crate::net::minecraft::core::registry::Registry;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::core::registry_synchronization::PackedRegistryEntry;
use crate::net::minecraft::resources::resource_key::ResourceKey;

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
                Registry::new(registry.clone(), keys)
            });
        let mut registries = BuiltInRegistries::registries();
        registries.extend(received);
        RegistryAccess::new(registries)
    }
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
