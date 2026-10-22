//! net/minecraft/server/packs/PathPackResources.java
//!
//! Only the static helpers the vanilla pack uses so far: `listPath` and `validatePath`. A pack of
//! its own (a resource pack's directory) comes with the pack repository.

use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::io_supplier::IoSupplier;
use crate::net::minecraft::util::file_util::FileUtil;
use std::fs;
use std::path::Path;

pub struct PathPackResources;

impl PathPackResources {
    /// Every regular file under `top_dir`'s `decomposed_prefix_path`, as an identifier of its path
    /// from `top_dir` in `namespace`. Java walks with `Files.find`, whose order the callers sort
    /// away (into `TreeMap`s); the port walks each directory in its names' order. A path that
    /// isn't a valid identifier is skipped, which Java logs.
    pub fn list_path(
        namespace: &str,
        top_dir: &Path,
        decomposed_prefix_path: &[String],
        output: &mut dyn FnMut(Identifier, IoSupplier),
    ) {
        let target_path = FileUtil::resolve_path(top_dir, decomposed_prefix_path);
        let mut pending = vec![target_path];
        while let Some(directory) = pending.pop() {
            let Ok(entries) = fs::read_dir(&directory) else {
                continue;
            };
            let mut entries: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();
            entries.sort();
            for file in entries.into_iter().rev() {
                if file.is_dir() {
                    pending.push(file);
                } else if file.is_file() {
                    let relative = file.strip_prefix(top_dir).expect("under the top directory");
                    let resource_path: Vec<&str> = relative
                        .components()
                        .filter_map(|component| component.as_os_str().to_str())
                        .collect();
                    if let Some(identifier) =
                        Identifier::try_build(namespace, &resource_path.join("/"))
                    {
                        output(identifier, IoSupplier::create(&file));
                    }
                }
            }
        }
    }

    /// Java checks the path's case only with `DEBUG_VALIDATE_RESOURCE_PATH_CASE`, which is off.
    pub fn validate_path(_path: &Path) -> bool {
        true
    }
}
