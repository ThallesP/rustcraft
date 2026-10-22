//! net/minecraft/server/packs/FixedPathPackResources.java
//!
//! The vanilla pack's layers: resources by type from a list of directories, the last one pushed
//! first. Its root resources and metadata (`pack.mcmeta`, and the built-in `PackMetadataSection`)
//! come with the pack repository.

use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::pack_resources::PackResources;
use crate::net::minecraft::server::packs::pack_type::PackType;
use crate::net::minecraft::server::packs::path_pack_resources::PathPackResources;
use crate::net::minecraft::server::packs::resources::io_supplier::IoSupplier;
use crate::net::minecraft::util::file_util::FileUtil;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub struct FixedPathPackResources {
    id: String,
    namespaces: BTreeSet<String>,
    root_paths: Vec<PathBuf>,
    paths_for_type: BTreeMap<PackType, Vec<PathBuf>>,
}

impl FixedPathPackResources {
    pub fn root_paths(&self) -> &[PathBuf] {
        &self.root_paths
    }

    fn paths(&self, pack_type: PackType) -> &[PathBuf] {
        self.paths_for_type
            .get(&pack_type)
            .map_or(&[], Vec::as_slice)
    }

    fn get_resources(
        result: &mut dyn FnMut(Identifier, IoSupplier),
        namespace: &str,
        root: &Path,
        directory: &[String],
    ) {
        let namespace_dir = root.join(namespace);
        PathPackResources::list_path(namespace, &namespace_dir, directory, result);
    }
}

impl PackResources for FixedPathPackResources {
    /// Java logs an invalid path and returns null.
    fn get_resource(&self, pack_type: PackType, location: &Identifier) -> Option<IoSupplier> {
        let decomposed_path = FileUtil::decompose_path(location.get_path()).ok()?;
        let namespace = location.get_namespace();
        self.paths(pack_type).iter().find_map(|type_path| {
            let resource = FileUtil::resolve_path(&type_path.join(namespace), &decomposed_path);
            (resource.exists() && PathPackResources::validate_path(&resource))
                .then(|| IoSupplier::create(&resource))
        })
    }

    /// With several directories, the first one listing a resource wins (Java's `putIfAbsent` into
    /// a `HashMap`, handed on in its order, which the callers sort away).
    fn list_resources(
        &self,
        pack_type: PackType,
        namespace: &str,
        directory: &str,
        output: &mut dyn FnMut(Identifier, IoSupplier),
    ) {
        let Ok(decomposed_path) = FileUtil::decompose_path(directory) else {
            return;
        };
        let paths = self.paths(pack_type);
        match paths.len() {
            0 => {}
            1 => FixedPathPackResources::get_resources(
                output,
                namespace,
                &paths[0],
                &decomposed_path,
            ),
            _ => {
                let mut resources: BTreeMap<Identifier, IoSupplier> = BTreeMap::new();
                for path in paths {
                    FixedPathPackResources::get_resources(
                        &mut |id, supplier| {
                            resources.entry(id).or_insert(supplier);
                        },
                        namespace,
                        path,
                        &decomposed_path,
                    );
                }
                for (id, supplier) in resources {
                    output(id, supplier);
                }
            }
        }
    }

    fn get_namespaces(&self, _pack_type: PackType) -> BTreeSet<String> {
        self.namespaces.clone()
    }

    fn pack_id(&self) -> &str {
        &self.id
    }
}

/// `FixedPathPackResources.Builder`.
#[derive(Clone, Default)]
pub struct Builder {
    root_paths: Vec<PathBuf>,
    paths_for_type: BTreeMap<PackType, Vec<PathBuf>>,
    namespaces: BTreeSet<String>,
}

impl Builder {
    /// Panics where Java throws `IllegalArgumentException` for a path that isn't a directory.
    fn validate_dir_path(path: &Path) -> bool {
        if !path.exists() {
            return false;
        }
        assert!(path.is_dir(), "Path {} is not directory", path.display());
        true
    }

    pub fn push_root_path(&mut self, path: &Path) {
        if Builder::validate_dir_path(path)
            && !self.root_paths.iter().any(|existing| existing == path)
        {
            self.root_paths.push(path.to_path_buf());
        }
    }

    pub fn push_path_for_type(&mut self, pack_type: PackType, path: &Path) {
        if Builder::validate_dir_path(path) {
            let paths = self.paths_for_type.entry(pack_type).or_default();
            if !paths.iter().any(|existing| existing == path) {
                paths.push(path.to_path_buf());
            }
        }
    }

    pub fn expose_namespace(&mut self, namespaces: &[&str]) {
        self.namespaces
            .extend(namespaces.iter().map(|namespace| (*namespace).to_owned()));
    }

    /// The paths last pushed come first.
    pub fn build(&self, id: &str) -> FixedPathPackResources {
        let reversed = |paths: &[PathBuf]| paths.iter().rev().cloned().collect::<Vec<_>>();
        FixedPathPackResources {
            id: id.to_owned(),
            namespaces: self.namespaces.clone(),
            root_paths: reversed(&self.root_paths),
            paths_for_type: PackType::VALUES
                .into_iter()
                .map(|pack_type| {
                    let paths = self
                        .paths_for_type
                        .get(&pack_type)
                        .map_or(&[][..], Vec::as_slice);
                    (pack_type, reversed(paths))
                })
                .collect(),
        }
    }
}
