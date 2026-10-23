//! net/minecraft/network/chat/FontDescription.java
//!
//! Which font a text is drawn in: a font resource, or a sprite drawn as a glyph. Java's interface
//! of three records is an enum of them. `PlayerSprite`, which holds a `ResolvableProfile`, isn't
//! ported yet; only `Resource` has a codec in Java too.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::SharedCodec;
use crate::com::mojang::serialization::codec::{CodecExt, StaticCodec};
use crate::com::mojang::serialization::data_result::DataResult;
use crate::net::minecraft::resources::identifier::Identifier;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FontDescription {
    AtlasSprite {
        atlas_id: Identifier,
        sprite_id: Identifier,
    },
    Resource {
        id: Identifier,
    },
}

// `Identifier.CODEC.flatComapMap(Resource::new, …)`: only a resource is written.
static CODEC: StaticCodec<FontDescription> = LazyLock::new(|| {
    Identifier::CODEC
        .shared()
        .flat_comap_map(
            |id| FontDescription::Resource { id },
            |font_description: &FontDescription| match font_description {
                FontDescription::Resource { id } => DataResult::success(id.clone()),
                other => {
                    let message = format!("Unsupported font description type: {other}");
                    DataResult::error(|| message)
                }
            },
        )
        .boxed()
});

impl FontDescription {
    pub const CODEC: &'static StaticCodec<FontDescription> = &CODEC;

    /// `DEFAULT`: `minecraft:default`.
    pub fn default_font() -> FontDescription {
        FontDescription::Resource {
            id: Identifier::with_default_namespace("default"),
        }
    }
}

/// The records' `toString`.
impl fmt::Display for FontDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontDescription::AtlasSprite {
                atlas_id,
                sprite_id,
            } => write!(f, "AtlasSprite[atlasId={atlas_id}, spriteId={sprite_id}]"),
            FontDescription::Resource { id } => write!(f, "Resource[id={id}]"),
        }
    }
}
