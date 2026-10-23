//! net/minecraft/client/renderer/texture/atlas/sources/DirectoryLister.java

use crate::net::minecraft::client::renderer::texture::atlas::sprite_source::Output;
use crate::net::minecraft::resources::file_to_id_converter::FileToIdConverter;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;

/// Every PNG under `textures/<source>/`, named by its path there with the prefix before it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryLister {
    pub source_path: String,
    pub id_prefix: String,
}

impl DirectoryLister {
    pub fn run(&self, resource_manager: &dyn ResourceManager, output: &mut Output) {
        let converter = FileToIdConverter::new(&format!("textures/{}", self.source_path), ".png");
        for (identifier, resource) in converter.list_matching_resources(resource_manager) {
            let sprite_location = converter
                .file_to_id(&identifier)
                .with_prefix(&self.id_prefix);
            output.add(sprite_location, resource);
        }
    }
}
