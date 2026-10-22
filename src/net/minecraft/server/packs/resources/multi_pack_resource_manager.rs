//! net/minecraft/server/packs/resources/MultiPackResourceManager.java
//!
//! The client's resources over its packs, bottom first: a `FallbackResourceManager` per
//! namespace any pack has. Java's `HashMap` of them is only ever looked up or merged into a
//! `TreeMap`, so its order doesn't show. Packs' filter sections come with the pack repository.

use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::pack_resources::PackResources;
use crate::net::minecraft::server::packs::pack_type::PackType;
use crate::net::minecraft::server::packs::resources::fallback_resource_manager::FallbackResourceManager;
use crate::net::minecraft::server::packs::resources::resource::Resource;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

pub struct MultiPackResourceManager {
    namespaced_managers: BTreeMap<String, FallbackResourceManager>,
    packs: Vec<Rc<dyn PackResources>>,
}

impl MultiPackResourceManager {
    pub fn new(pack_type: PackType, packs: Vec<Rc<dyn PackResources>>) -> MultiPackResourceManager {
        let mut namespaces: Vec<String> = Vec::new();
        for pack in &packs {
            for namespace in pack.get_namespaces(pack_type) {
                if !namespaces.contains(&namespace) {
                    namespaces.push(namespace);
                }
            }
        }
        let mut namespaced_managers: BTreeMap<String, FallbackResourceManager> = BTreeMap::new();
        for pack in &packs {
            let provided_namespaces = pack.get_namespaces(pack_type);
            for namespace in &namespaces {
                if provided_namespaces.contains(namespace) {
                    namespaced_managers
                        .entry(namespace.clone())
                        .or_insert_with(|| FallbackResourceManager::new(pack_type, namespace))
                        .push(pack.clone());
                }
            }
        }
        MultiPackResourceManager {
            namespaced_managers,
            packs,
        }
    }

    pub fn list_packs(&self) -> &[Rc<dyn PackResources>] {
        &self.packs
    }

    /// Panics where Java throws `IllegalArgumentException` for a directory ending in a slash.
    fn check_trailing_directory_path(directory: &str) {
        assert!(
            !directory.ends_with('/'),
            "Trailing slash in path {directory}"
        );
    }
}

impl ResourceManager for MultiPackResourceManager {
    fn get_namespaces(&self) -> BTreeSet<String> {
        self.namespaced_managers.keys().cloned().collect()
    }

    fn get_resource(&self, location: &Identifier) -> Option<Resource> {
        self.namespaced_managers
            .get(location.get_namespace())?
            .get_resource(location)
    }

    fn get_resource_stack(&self, location: &Identifier) -> Vec<Resource> {
        self.namespaced_managers
            .get(location.get_namespace())
            .map_or_else(Vec::new, |pack| pack.get_resource_stack(location))
    }

    fn list_resources(
        &self,
        directory: &str,
        selector: &dyn Fn(&Identifier) -> bool,
    ) -> BTreeMap<Identifier, Resource> {
        MultiPackResourceManager::check_trailing_directory_path(directory);
        let mut result = BTreeMap::new();
        for manager in self.namespaced_managers.values() {
            result.extend(manager.list_resources(directory, selector));
        }
        result
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::golden::Golden;
    use crate::net::minecraft::server::packs::vanilla_pack_resources::VanillaPackResources;
    use std::path::Path;

    /// The client's resources over the vanilla pack, like `TextureSuites.vanillaResources`: its
    /// layers, the asset index's without a path.
    pub fn vanilla_resources() -> MultiPackResourceManager {
        let vanilla = VanillaPackResources::create_client_pack(
            &VanillaPackResources::vanilla_jar_root(),
            Path::new("no-asset-index"),
        );
        let layers = vanilla
            .resource_layers()
            .iter()
            .map(|layer| layer.clone() as Rc<dyn PackResources>)
            .collect();
        MultiPackResourceManager::new(PackType::CLIENT_RESOURCES, layers)
    }

    #[test]
    fn lists_the_vanilla_packs_resources_like_vanilla() {
        let golden = Golden::load("net.minecraft.server.packs.resources.MultiPackResourceManager");
        let manager = vanilla_resources();
        golden.check("getNamespaces()Ljava/util/Set;", || {
            manager.get_namespaces().into_iter().collect::<Vec<_>>()
        });
        golden.check(
            "listResources(Ljava/lang/String;Lnet/minecraft/server/packs/resources/ResourceManager$Selector;)Ljava/util/Map;",
            |directory: String| {
                manager
                    .list_resources(&directory, &|_| true)
                    .keys()
                    .map(Identifier::to_string)
                    .collect::<Vec<_>>()
            },
        );
        golden.check(
            "getResourceStack(Lnet/minecraft/resources/Identifier;)Ljava/util/List;",
            |id: String| manager.get_resource_stack(&Identifier::parse(&id)).len() as i32,
        );
        golden.finish();
    }
}
