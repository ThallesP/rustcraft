//! net/minecraft/client/gui/font/providers/GlyphProviderDefinition.java (with GlyphProviderType.java
//! and ProviderReferenceDefinition.java)
//!
//! A font file's provider: a kind (`type`), what the kind reads, and a filter of font options. A
//! definition either loads a provider or references another font's. Decoded by hand from
//! serde_json, like the metadata sections, until DataFixerUpper's JSON ops come. The `ttf` and
//! `unihex` kinds aren't ported: loading one fails like a provider whose file is missing, which
//! Java logs and rejects. The vanilla pack's own fonts have neither (the Unicode font is the
//! launcher's asset index's, which the port doesn't read yet).

use std::rc::Rc;

use crate::com::mojang::blaze3d::font::glyph_provider::GlyphProvider;
use crate::com::mojang::blaze3d::font::space_provider;
use crate::net::minecraft::client::gui::font::font_option::Filter;
use crate::net::minecraft::client::gui::font::providers::bitmap_provider;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;

#[derive(Clone, Debug, PartialEq)]
pub enum GlyphProviderDefinition {
    Bitmap(bitmap_provider::Definition),
    Space(space_provider::Definition),
    /// `ProviderReferenceDefinition`: another font's providers.
    Reference(Identifier),
    /// A kind that isn't ported, by its name.
    Unported(String),
}

/// `GlyphProviderDefinition.unpack()`'s two sides.
pub enum Unpacked<'a> {
    Loader(&'a GlyphProviderDefinition),
    Reference(Identifier),
}

/// `GlyphProviderDefinition.Conditional`.
#[derive(Clone, Debug, PartialEq)]
pub struct Conditional {
    pub definition: GlyphProviderDefinition,
    pub filter: Filter,
}

impl Conditional {
    /// `CODEC`: the definition's fields by its `type`, and `filter`, always passing when absent.
    pub fn decode(value: &serde_json::Value) -> Result<Conditional, String> {
        let object = value
            .as_object()
            .ok_or_else(|| format!("Not a map: {value}"))?;
        let definition = match object.get("type").and_then(|kind| kind.as_str()) {
            Some("bitmap") => {
                GlyphProviderDefinition::Bitmap(bitmap_provider::Definition::decode(object)?)
            }
            Some("space") => {
                GlyphProviderDefinition::Space(space_provider::Definition::decode(object)?)
            }
            Some("reference") => {
                let id = object
                    .get("id")
                    .and_then(|id| id.as_str())
                    .ok_or("No key id in MapLike")?;
                GlyphProviderDefinition::Reference(
                    Identifier::try_parse(id)
                        .ok_or_else(|| format!("Not a valid resource location: {id}"))?,
                )
            }
            Some(kind @ ("ttf" | "unihex")) => GlyphProviderDefinition::Unported(kind.to_owned()),
            Some(kind) => return Err(format!("Unknown element name:{kind}")),
            None => return Err("No key type in MapLike".to_owned()),
        };
        let filter = match object.get("filter") {
            Some(filter) => Filter::decode(filter)?,
            None => Filter::ALWAYS_PASS,
        };
        Ok(Conditional { definition, filter })
    }
}

impl GlyphProviderDefinition {
    pub fn unpack(&self) -> Unpacked<'_> {
        match self {
            GlyphProviderDefinition::Reference(id) => Unpacked::Reference(id.clone()),
            _ => Unpacked::Loader(self),
        }
    }

    /// `Loader.load`: Java's `IOException` (or an unported kind) is the `Err`.
    pub fn load(
        &self,
        resource_manager: &dyn ResourceManager,
    ) -> Result<Rc<dyn GlyphProvider>, String> {
        match self {
            GlyphProviderDefinition::Bitmap(definition) => definition.load(resource_manager),
            GlyphProviderDefinition::Space(definition) => Ok(definition.load()),
            GlyphProviderDefinition::Reference(_) => unreachable!("a reference doesn't load"),
            GlyphProviderDefinition::Unported(kind) => {
                Err(format!("the {kind} provider isn't ported"))
            }
        }
    }
}
