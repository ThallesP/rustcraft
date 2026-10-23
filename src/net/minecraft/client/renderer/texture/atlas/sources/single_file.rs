//! net/minecraft/client/renderer/texture/atlas/sources/SingleFile.java

use crate::net::minecraft::client::renderer::texture::atlas::sprite_source::{
    Output, texture_id_converter,
};
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;

/// One texture, named by its id or the sprite id given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SingleFile {
    pub resource_id: Identifier,
    pub sprite_id: Option<Identifier>,
}

impl SingleFile {
    /// Java logs a texture that's missing.
    pub fn run(&self, resource_manager: &dyn ResourceManager, output: &mut Output) {
        let full_resource_id = texture_id_converter().id_to_file(&self.resource_id);
        match resource_manager.get_resource(&full_resource_id) {
            Some(resource) => output.add(
                self.sprite_id
                    .clone()
                    .unwrap_or_else(|| self.resource_id.clone()),
                resource,
            ),
            None => eprintln!("Missing sprite: {full_resource_id}"),
        }
    }
}
