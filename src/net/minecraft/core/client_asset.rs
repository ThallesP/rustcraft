//! net/minecraft/core/ClientAsset.java
//!
//! Only `ResourceTexture`, the only kind of `ClientAsset.Texture` so far (the downloaded skins'
//! textures come with online mode). Not ported yet: its codecs.

use crate::net::minecraft::resources::identifier::Identifier;

/// `ClientAsset.ResourceTexture`: an asset id and its texture's path,
/// `textures/<path>.png`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceTexture {
    pub id: Identifier,
    pub texture_path: Identifier,
}

impl ResourceTexture {
    pub fn new(texture: Identifier) -> ResourceTexture {
        let texture_path = texture.with_path(&format!("textures/{}.png", texture.get_path()));
        ResourceTexture {
            id: texture,
            texture_path,
        }
    }

    pub fn texture_path(&self) -> &Identifier {
        &self.texture_path
    }
}
