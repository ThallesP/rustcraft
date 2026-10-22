//! net/minecraft/server/packs/resources/ResourceManager.java and ResourceProvider.java
//!
//! `listResources`' map is Java's `TreeMap`, a `BTreeMap` of identifiers in `compareTo`'s order.
//! `Selector` is an `Fn(&Identifier) -> bool`. `listResourceStacks` and `listPacks` come with
//! their callers.

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
}
