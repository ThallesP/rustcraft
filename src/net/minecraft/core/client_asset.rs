//! net/minecraft/core/ClientAsset.java
//!
//! Only `ResourceTexture`, the only kind of `ClientAsset.Texture` so far (the downloaded skins'
//! textures come with online mode), with its `CODEC` (a wolf variant's textures). Not ported yet:
//! `DEFAULT_FIELD_CODEC` and `STREAM_CODEC`, which come with their callers.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, SharedCodec, StaticCodec};
use crate::net::minecraft::resources::identifier::Identifier;

static RESOURCE_TEXTURE_CODEC: StaticCodec<ResourceTexture> = LazyLock::new(|| {
    Identifier::CODEC
        .shared()
        .xmap(ResourceTexture::new, |texture: &ResourceTexture| {
            texture.id.clone()
        })
        .boxed()
});

/// `ClientAsset.ResourceTexture`: an asset id and its texture's path,
/// `textures/<path>.png`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceTexture {
    pub id: Identifier,
    pub texture_path: Identifier,
}

impl ResourceTexture {
    pub const CODEC: &'static StaticCodec<ResourceTexture> = &RESOURCE_TEXTURE_CODEC;

    pub fn new(texture: Identifier) -> ResourceTexture {
        let texture_path = texture.with_path(&format!("textures/{}.png", texture.get_path()));
        ResourceTexture {
            id: texture,
            texture_path,
        }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn texture_path(&self) -> &Identifier {
        &self.texture_path
    }
}
