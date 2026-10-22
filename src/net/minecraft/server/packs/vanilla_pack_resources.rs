//! net/minecraft/server/packs/VanillaPackResources.java and VanillaPackResourcesBuilder.java
//!
//! The vanilla pack as `ClientPackSource.createVanillaPackSource` builds it: the game jar's
//! resources (`pushJarResources`), then a layer for the launcher's asset index. Java finds the
//! jar's `assets/` and `data/` through `.mcassetsroot` files on the classpath; the port's jar is
//! `vanilla/<version>/resources/`, the client jar unpacked (`tools/setup_vanilla.py`). The asset
//! index (sounds and translations) isn't downloaded, so that layer has no path, as Java's has
//! none when the directory doesn't exist. The builder is its own module's struct, with the two
//! Java files' parts the client calls.

use crate::net::minecraft::server::packs::fixed_path_pack_resources::{
    Builder as FixedPathBuilder, FixedPathPackResources,
};
use crate::net::minecraft::server::packs::pack_type::PackType;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct VanillaPackResources {
    full_resources: Rc<FixedPathPackResources>,
    resource_layers: Vec<Rc<FixedPathPackResources>>,
}

impl VanillaPackResources {
    pub fn full_resources(&self) -> &Rc<FixedPathPackResources> {
        &self.full_resources
    }

    /// The layers the pack repository opens (`asResourcesSupplier().openResources`).
    pub fn resource_layers(&self) -> &[Rc<FixedPathPackResources>] {
        &self.resource_layers
    }

    /// `ClientPackSource.createVanillaPackSource` over the unpacked jar at `jar_root` and the
    /// asset index at `external_asset_root`.
    pub fn create_client_pack(jar_root: &Path, external_asset_root: &Path) -> VanillaPackResources {
        VanillaPackResourcesBuilder::new()
            .expose_namespace(&["minecraft", "realms"])
            .push_jar_resources(jar_root)
            .push_layer()
            .push_asset_path(PackType::CLIENT_RESOURCES, external_asset_root)
            .build("vanilla")
    }

    /// `vanilla/<version>/resources/`, the client jar unpacked.
    pub fn vanilla_jar_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("vanilla")
            .join(include_str!("../../../../../MINECRAFT_VERSION").trim())
            .join("resources")
    }
}

/// `VanillaPackResourcesBuilder`.
pub struct VanillaPackResourcesBuilder {
    full_builder: FixedPathBuilder,
    layered_builders: Vec<FixedPathBuilder>,
    namespaces: Vec<String>,
}

impl Default for VanillaPackResourcesBuilder {
    fn default() -> Self {
        VanillaPackResourcesBuilder::new()
    }
}

impl VanillaPackResourcesBuilder {
    pub fn new() -> VanillaPackResourcesBuilder {
        VanillaPackResourcesBuilder {
            full_builder: FixedPathBuilder::default(),
            layered_builders: vec![FixedPathBuilder::default()],
            namespaces: Vec::new(),
        }
    }

    fn for_last_layer(&mut self, task: impl Fn(&mut FixedPathBuilder)) {
        task(&mut self.full_builder);
        task(self.layered_builders.last_mut().expect("a layer"));
    }

    fn for_all_layers(&mut self, task: impl Fn(&mut FixedPathBuilder)) {
        task(&mut self.full_builder);
        for layered_builder in &mut self.layered_builders {
            task(layered_builder);
        }
    }

    /// Each type's directory in the jar (`assets/`, `data/`), where its `.mcassetsroot` is.
    pub fn push_jar_resources(mut self, jar_root: &Path) -> Self {
        for pack_type in PackType::VALUES {
            let path = jar_root.join(pack_type.get_directory());
            if path.join(".mcassetsroot").exists() {
                let parent = path.parent().expect("under the jar's root").to_path_buf();
                self.for_last_layer(|builder| builder.push_root_path(&parent));
                self.for_last_layer(|builder| builder.push_path_for_type(pack_type, &path));
            }
        }
        self
    }

    pub fn push_asset_path(mut self, pack_type: PackType, path: &Path) -> Self {
        self.for_last_layer(|builder| builder.push_root_path(path));
        self.for_last_layer(|builder| builder.push_path_for_type(pack_type, path));
        self
    }

    pub fn expose_namespace(mut self, namespaces: &[&str]) -> Self {
        self.namespaces
            .extend(namespaces.iter().map(|namespace| (*namespace).to_owned()));
        self.for_all_layers(|builder| builder.expose_namespace(namespaces));
        self
    }

    pub fn push_layer(mut self) -> Self {
        let mut new_builder = FixedPathBuilder::default();
        let namespaces: Vec<&str> = self.namespaces.iter().map(String::as_str).collect();
        new_builder.expose_namespace(&namespaces);
        self.layered_builders.push(new_builder);
        self
    }

    pub fn build(self, id: &str) -> VanillaPackResources {
        VanillaPackResources {
            full_resources: Rc::new(self.full_builder.build(id)),
            resource_layers: self
                .layered_builders
                .iter()
                .map(|builder| Rc::new(builder.build(id)))
                .collect(),
        }
    }
}
