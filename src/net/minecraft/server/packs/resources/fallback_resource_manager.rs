//! net/minecraft/server/packs/resources/FallbackResourceManager.java
//!
//! One namespace's resources over a stack of packs, the last pushed winning. Packs' filters
//! (`ResourceFilterSection`) and filter-only entries come with the pack repository: the vanilla
//! pack has none. `LeakedResourceWarningInputStream`, a debug log's, isn't ported.

use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::pack_resources::PackResources;
use crate::net::minecraft::server::packs::pack_type::PackType;
use crate::net::minecraft::server::packs::resources::io_supplier::IoSupplier;
use crate::net::minecraft::server::packs::resources::resource::Resource;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;

pub struct FallbackResourceManager {
    fallbacks: Vec<Rc<dyn PackResources>>,
    pack_type: PackType,
    namespace: String,
}

impl FallbackResourceManager {
    pub fn new(pack_type: PackType, namespace: &str) -> FallbackResourceManager {
        FallbackResourceManager {
            fallbacks: Vec::new(),
            pack_type,
            namespace: namespace.to_owned(),
        }
    }

    pub fn push(&mut self, pack: Rc<dyn PackResources>) {
        self.fallbacks.push(pack);
    }

    fn is_metadata(location: &Identifier) -> bool {
        location.get_path().ends_with(".mcmeta")
    }

    fn get_identifier_from_metadata(identifier: &Identifier) -> Identifier {
        let path = identifier.get_path();
        identifier.with_path(&path[..path.len() - ".mcmeta".len()])
    }

    fn get_metadata_location(identifier: &Identifier) -> Identifier {
        identifier.with_path(&format!("{}.mcmeta", identifier.get_path()))
    }

    /// `createStackMetadataFinder`: the topmost `.mcmeta` from the resource's pack up.
    fn find_stack_metadata(
        &self,
        location: &Identifier,
        final_pack_index: usize,
    ) -> Option<IoSupplier> {
        let metadata_location = FallbackResourceManager::get_metadata_location(location);
        self.fallbacks[final_pack_index..]
            .iter()
            .rev()
            .find_map(|pack| pack.get_resource(self.pack_type, &metadata_location))
    }
}

impl ResourceManager for FallbackResourceManager {
    fn get_namespaces(&self) -> BTreeSet<String> {
        BTreeSet::from([self.namespace.clone()])
    }

    fn get_resource(&self, location: &Identifier) -> Option<Resource> {
        for (i, fallback) in self.fallbacks.iter().enumerate().rev() {
            if let Some(resource) = fallback.get_resource(self.pack_type, location) {
                let metadata = self.find_stack_metadata(location, i);
                return Some(Resource::new(fallback.pack_id(), resource, metadata));
            }
        }
        None
    }

    /// Every pack's resource at the location, bottom first, each with its own pack's metadata.
    fn get_resource_stack(&self, location: &Identifier) -> Vec<Resource> {
        let metadata_location = FallbackResourceManager::get_metadata_location(location);
        let mut result = Vec::new();
        for file_source in self.fallbacks.iter().rev() {
            if let Some(resource) = file_source.get_resource(self.pack_type, location) {
                let metadata = file_source.get_resource(self.pack_type, &metadata_location);
                result.push(Resource::new(file_source.pack_id(), resource, metadata));
            }
        }
        result.reverse();
        result
    }

    /// The topmost pack's file for each location, with the `.mcmeta` beside it when that comes
    /// from the same pack or one above.
    fn list_resources(
        &self,
        directory: &str,
        selector: &dyn Fn(&Identifier) -> bool,
    ) -> BTreeMap<Identifier, Resource> {
        let mut top_resource_for_file_location: HashMap<Identifier, (usize, IoSupplier)> =
            HashMap::new();
        let mut top_resource_for_meta_location: HashMap<Identifier, (usize, IoSupplier)> =
            HashMap::new();
        for (pack_index, pack) in self.fallbacks.iter().enumerate() {
            pack.list_resources(
                self.pack_type,
                &self.namespace,
                directory,
                &mut |resource, stream_supplier| {
                    if FallbackResourceManager::is_metadata(&resource) {
                        if selector(&FallbackResourceManager::get_identifier_from_metadata(
                            &resource,
                        )) {
                            top_resource_for_meta_location
                                .insert(resource, (pack_index, stream_supplier));
                        }
                    } else if selector(&resource) {
                        top_resource_for_file_location
                            .insert(resource, (pack_index, stream_supplier));
                    }
                },
            );
        }

        let mut result = BTreeMap::new();
        for (location, (pack_index, resource)) in top_resource_for_file_location {
            let metadata_location = FallbackResourceManager::get_metadata_location(&location);
            let metadata = top_resource_for_meta_location
                .get(&metadata_location)
                .filter(|(meta_pack_index, _)| *meta_pack_index >= pack_index)
                .map(|(_, supplier)| supplier.clone());
            let pack_id = self.fallbacks[pack_index].pack_id();
            result.insert(location, Resource::new(pack_id, resource, metadata));
        }
        result
    }
}
