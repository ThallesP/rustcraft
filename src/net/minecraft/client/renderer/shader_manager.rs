//! net/minecraft/client/renderer/ShaderManager.java
//!
//! Only `loadConfigs`' shaders and includes so far, the `ShaderSource` pipelines compile from,
//! read from a resource pack's directory (the vanilla pack's is `vanilla/<version>/resources/`)
//! until the `ResourceManager` comes. Not ported yet: post chains, reloading and compiling every
//! pipeline (`reload`, `apply`), which come with the device and the `ResourceManager`.

use crate::com::mojang::renderpearl::api::pipeline::shader_source::{
    CachedIncludeSource, ShaderSource,
};
use crate::com::mojang::renderpearl::api::pipeline::shader_type::ShaderType;
use crate::net::minecraft::resources::file_to_id_converter::FileToIdConverter;
use crate::net::minecraft::resources::identifier::Identifier;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct ShaderManager;

impl ShaderManager {
    pub const MAX_LOG_LENGTH: i32 = 32768;
    pub const SHADER_PATH: &str = "shaders";
    pub const SHADER_INCLUDE_PATH: &str = "shaders/include/";
    pub const SHADER_INCLUDE_EXTENSION: &str = ".glsl";

    pub fn shader_include_converter() -> FileToIdConverter {
        FileToIdConverter::new("shaders/include", ".glsl")
    }

    /// The shaders and includes under `assets/<namespace>/shaders/` in a pack's directory.
    /// Java's `listResources("shaders", …)` lists every file there, which this walks.
    pub fn load_configs(pack: &Path) -> std::io::Result<Configs> {
        let mut configs = Configs::default();
        for namespace in fs::read_dir(pack.join("assets"))? {
            let namespace = namespace?;
            let Some(namespace_name) = namespace.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let root = namespace.path();
            let mut pending = vec![root.join(ShaderManager::SHADER_PATH)];
            while let Some(directory) = pending.pop() {
                let Ok(entries) = fs::read_dir(&directory) else {
                    continue;
                };
                for entry in entries {
                    let path = entry?.path();
                    if path.is_dir() {
                        pending.push(path);
                        continue;
                    }
                    let relative = path.strip_prefix(&root).expect("under the namespace");
                    let Some(location) = relative
                        .to_str()
                        .and_then(|relative| Identifier::try_build(&namespace_name, relative))
                    else {
                        continue;
                    };
                    if let Some(shader_type) = ShaderType::by_location(&location) {
                        ShaderManager::load_shader(&location, &path, shader_type, &mut configs)?;
                    } else if ShaderManager::shader_include_converter().matches(&location) {
                        ShaderManager::load_include(&location, &path, &mut configs)?;
                    }
                }
            }
        }
        Ok(configs)
    }

    fn load_shader(
        location: &Identifier,
        path: &Path,
        shader_type: ShaderType,
        configs: &mut Configs,
    ) -> std::io::Result<()> {
        let contents = fs::read_to_string(path)?;
        let id = shader_type.id_converter().file_to_id(location);
        configs.shader_sources.insert((id, shader_type), contents);
        Ok(())
    }

    fn load_include(
        location: &Identifier,
        path: &Path,
        configs: &mut Configs,
    ) -> std::io::Result<()> {
        let contents = fs::read_to_string(path)?;
        let id = ShaderManager::include_shader_location_to_id(location);
        configs
            .include_sources
            .insert(id.clone(), CachedIncludeSource::create(&id, &contents));
        Ok(())
    }

    /// `minecraft:shaders/include/fog.glsl` is `minecraft:fog.glsl`.
    fn include_shader_location_to_id(location: &Identifier) -> Identifier {
        ShaderManager::shader_include_converter()
            .file_to_id(location)
            .with_suffix(".glsl")
    }
}

/// `ShaderManager.Configs`: the shaders by id and stage, and the includes by id.
#[derive(Default)]
pub struct Configs {
    shader_sources: HashMap<(Identifier, ShaderType), String>,
    include_sources: HashMap<Identifier, CachedIncludeSource>,
}

impl ShaderSource for Configs {
    fn get_shader(&self, id: &Identifier, shader_type: ShaderType) -> Option<&str> {
        self.shader_sources
            .get(&(id.clone(), shader_type))
            .map(String::as_str)
    }

    fn get_include(&self, id: &Identifier) -> Option<&CachedIncludeSource> {
        self.include_sources.get(id)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::LazyLock;

    /// The vanilla pack's directory, which `tools/setup_vanilla.py` extracts from the client jar.
    pub fn vanilla_pack() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vanilla")
            .join(include_str!("../../../../../MINECRAFT_VERSION").trim())
            .join("resources")
    }

    /// The vanilla pack's shaders, loaded once.
    pub fn vanilla_configs() -> &'static Configs {
        static CONFIGS: LazyLock<Configs> = LazyLock::new(|| {
            ShaderManager::load_configs(&vanilla_pack()).expect("the vanilla pack's shaders load")
        });
        &CONFIGS
    }

    #[test]
    fn loads_the_vanilla_packs_shaders_and_includes() {
        let configs = vanilla_configs();
        let gui = Identifier::with_default_namespace("core/gui");
        assert!(configs.get_shader(&gui, ShaderType::VERTEX).is_some());
        assert!(configs.get_shader(&gui, ShaderType::FRAGMENT).is_some());
        assert!(
            configs
                .get_include(&Identifier::with_default_namespace("fog.glsl"))
                .is_some()
        );
        assert!(
            configs
                .get_include(&Identifier::with_default_namespace("fog"))
                .is_none()
        );
    }
}
