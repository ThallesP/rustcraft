//! net/minecraft/server/packs/resources/ResourceManager.java and ResourceProvider.java
//!
//! `listResources`' map is Java's `TreeMap`, a `BTreeMap` of identifiers in `compareTo`'s order.
//! `Selector` is an `Fn(&Identifier) -> bool`. `listPacks` comes with its callers.

use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource::Resource;
use std::collections::{BTreeMap, BTreeSet};

pub trait ResourceManager {
    fn get_namespaces(&self) -> BTreeSet<String>;

    /// `ResourceProvider.getResource`.
    fn get_resource(&self, location: &Identifier) -> Option<Resource>;

    fn get_resource_stack(&self, location: &Identifier) -> Vec<Resource>;

    fn list_resources(
        &self,
        directory: &str,
        selector: &dyn Fn(&Identifier) -> bool,
    ) -> BTreeMap<Identifier, Resource>;

    /// `listResourceStacks`: every listed resource's stack, bottom first. Java gathers the stacks
    /// as it lists, which is the same while no pack filters another's resources out, and the
    /// port's packs have no filter sections.
    fn list_resource_stacks(
        &self,
        directory: &str,
        selector: &dyn Fn(&Identifier) -> bool,
    ) -> BTreeMap<Identifier, Vec<Resource>> {
        self.list_resources(directory, selector)
            .into_keys()
            .map(|id| {
                let stack = self.get_resource_stack(&id);
                (id, stack)
            })
            .collect()
    }
}
