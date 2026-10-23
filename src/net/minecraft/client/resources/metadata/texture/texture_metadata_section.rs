//! net/minecraft/client/resources/metadata/texture/TextureMetadataSection.java
//!
//! A texture's `texture` section, decoded by hand from serde_json until DataFixerUpper's codecs
//! come (PORTING.md), with the codec's defaults. `alpha_cutoff_bias` is serde_json's double
//! narrowed to a float, where Gson parses the float from the text: the same for every value
//! vanilla's files hold (0.1).

use crate::net::minecraft::client::renderer::texture::mipmap_strategy::MipmapStrategy;
use crate::net::minecraft::server::packs::resources::resource_metadata::MetadataSectionType;
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextureMetadataSection {
    pub blur: bool,
    pub clamp: bool,
    pub mipmap_strategy: MipmapStrategy,
    pub alpha_cutoff_bias: f32,
}

impl TextureMetadataSection {
    pub const DEFAULT_BLUR: bool = false;
    pub const DEFAULT_CLAMP: bool = false;
    pub const DEFAULT_ALPHA_CUTOFF_BIAS: f32 = 0.0;
    pub const TYPE: MetadataSectionType<TextureMetadataSection> = MetadataSectionType {
        name: "texture",
        codec: TextureMetadataSection::codec,
    };

    pub fn codec(value: &Value) -> Result<TextureMetadataSection, String> {
        let object = value
            .as_object()
            .ok_or_else(|| format!("Not a map: {value}"))?;
        let boolean = |key: &str| -> Result<bool, String> {
            match object.get(key) {
                Some(Value::Bool(value)) => Ok(*value),
                Some(other) => Err(format!("Not a boolean: {other}")),
                None => Ok(false),
            }
        };
        let mipmap_strategy = match object.get("mipmap_strategy") {
            Some(Value::String(name)) => MipmapStrategy::by_name(name)
                .ok_or_else(|| format!("Unknown element name:{name}"))?,
            Some(other) => return Err(format!("Not a string: {other}")),
            None => MipmapStrategy::AUTO,
        };
        let alpha_cutoff_bias = match object.get("alpha_cutoff_bias") {
            Some(value) => value
                .as_f64()
                .ok_or_else(|| format!("Not a number: {value}"))? as f32,
            None => TextureMetadataSection::DEFAULT_ALPHA_CUTOFF_BIAS,
        };
        Ok(TextureMetadataSection {
            blur: boolean("blur")?,
            clamp: boolean("clamp")?,
            mipmap_strategy,
            alpha_cutoff_bias,
        })
    }
}
