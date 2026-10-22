//! net/minecraft/server/packs/PackResources.java
//!
//! A pack's resources by type and identifier. `ResourceOutput`, a `BiConsumer`, is an `FnMut`.
//! Its metadata (`PackMetadataResources`, `pack.mcmeta`'s sections) and root resources come with
//! the pack repository; the packs read so far have no filter or overlay sections.

use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::pack_type::PackType;
use crate::net::minecraft::server::packs::resources::io_supplier::IoSupplier;
use std::collections::BTreeSet;

pub const METADATA_EXTENSION: &str = ".mcmeta";
pub const PACK_META: &str = "pack.mcmeta";

pub trait PackResources {
    fn get_resource(&self, pack_type: PackType, location: &Identifier) -> Option<IoSupplier>;

    fn list_resources(
        &self,
        pack_type: PackType,
        namespace: &str,
        directory: &str,
        output: &mut dyn FnMut(Identifier, IoSupplier),
    );

    /// Java's `Set`, whose order nothing sees.
    fn get_namespaces(&self, pack_type: PackType) -> BTreeSet<String>;

    /// `location().id()`.
    fn pack_id(&self) -> &str;
}
