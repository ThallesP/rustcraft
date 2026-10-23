//! net/minecraft/client/resources/metadata/gui/GuiMetadataSection.java
//!
//! A GUI sprite's `gui` section: how it scales. Decoded by hand from serde_json until
//! DataFixerUpper's JSON ops come (PORTING.md), like `TextureMetadataSection`, with the codec's
//! default (`stretch`) for a section without `scaling`.

use crate::net::minecraft::client::resources::metadata::gui::gui_sprite_scaling::GuiSpriteScaling;
use crate::net::minecraft::server::packs::resources::resource_metadata::MetadataSectionType;
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuiMetadataSection {
    pub scaling: GuiSpriteScaling,
}

impl GuiMetadataSection {
    pub const DEFAULT: GuiMetadataSection = GuiMetadataSection {
        scaling: GuiSpriteScaling::DEFAULT,
    };
    pub const TYPE: MetadataSectionType<GuiMetadataSection> = MetadataSectionType {
        name: "gui",
        codec: GuiMetadataSection::codec,
    };

    /// `CODEC`: `scaling`, optional, failing when it's there but doesn't decode.
    pub fn codec(value: &Value) -> Result<GuiMetadataSection, String> {
        let object = value
            .as_object()
            .ok_or_else(|| format!("Not a map: {value}"))?;
        let scaling = match object.get("scaling") {
            Some(scaling) => GuiSpriteScaling::codec(scaling)?,
            None => GuiSpriteScaling::DEFAULT,
        };
        Ok(GuiMetadataSection { scaling })
    }
}
