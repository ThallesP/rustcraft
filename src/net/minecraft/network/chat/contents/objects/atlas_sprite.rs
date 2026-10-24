//! net/minecraft/network/chat/contents/objects/AtlasSprite.java

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, SharedCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, StaticMapCodec};
use crate::net::minecraft::network::chat::contents::objects::object_info::ObjectInfo;
use crate::net::minecraft::network::chat::font_description::FontDescription;
use crate::net::minecraft::resources::identifier::Identifier;

static MAP_CODEC: StaticMapCodec<ObjectInfo> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (
            Identifier::CODEC
                .shared()
                .optional_field_of_default("atlas", AtlasSprite::default_atlas())
                .for_getter(|info: &ObjectInfo| match info {
                    ObjectInfo::Atlas(sprite) => sprite.atlas.clone(),
                    other => panic!("ClassCastException: {other}"),
                }),
            Identifier::CODEC.shared().field_of("sprite").for_getter(
                |info: &ObjectInfo| match info {
                    ObjectInfo::Atlas(sprite) => sprite.sprite.clone(),
                    other => panic!("ClassCastException: {other}"),
                },
            ),
        ),
        |atlas, sprite| ObjectInfo::Atlas(AtlasSprite::new(atlas, sprite)),
    )
    .boxed()
});

/// A sprite of an atlas, drawn as a glyph.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AtlasSprite {
    atlas: Identifier,
    sprite: Identifier,
}

impl AtlasSprite {
    pub const MAP_CODEC: &'static StaticMapCodec<ObjectInfo> = &MAP_CODEC;

    pub fn new(atlas: Identifier, sprite: Identifier) -> AtlasSprite {
        AtlasSprite { atlas, sprite }
    }

    /// `DEFAULT_ATLAS`: `AtlasIds.BLOCKS`.
    pub fn default_atlas() -> Identifier {
        Identifier::with_default_namespace("blocks")
    }

    pub fn atlas(&self) -> &Identifier {
        &self.atlas
    }

    pub fn sprite(&self) -> &Identifier {
        &self.sprite
    }

    pub fn font_description(&self) -> FontDescription {
        FontDescription::AtlasSprite {
            atlas_id: self.atlas.clone(),
            sprite_id: self.sprite.clone(),
        }
    }

    fn to_short_name(id: &Identifier) -> String {
        if id.get_namespace() == "minecraft" {
            id.get_path().to_owned()
        } else {
            id.to_string()
        }
    }

    /// `[stone]`, or `[stone@gui]` from another atlas.
    pub fn default_fallback(&self) -> String {
        let short_name = AtlasSprite::to_short_name(&self.sprite);
        if self.atlas == AtlasSprite::default_atlas() {
            format!("[{short_name}]")
        } else {
            format!("[{short_name}@{}]", AtlasSprite::to_short_name(&self.atlas))
        }
    }
}

/// The record's `toString`.
impl fmt::Display for AtlasSprite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AtlasSprite[atlas={}, sprite={}]",
            self.atlas, self.sprite
        )
    }
}
