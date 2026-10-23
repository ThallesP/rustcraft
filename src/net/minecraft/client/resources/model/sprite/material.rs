//! net/minecraft/client/resources/model/sprite/Material.java
//!
//! `CODEC` (DataFixerUpper's `Codec.either` of an identifier or `{sprite, force_translucent}`)
//! is decoded by hand from Gson's tree (`Material::parse`), the way the codec reads it.

use std::sync::Arc;

use crate::com::google::gson::json_element::JsonElement;
use crate::net::minecraft::client::renderer::texture::texture_atlas_sprite::TextureAtlasSprite;
use crate::net::minecraft::resources::identifier::Identifier;

/// A texture a model names, and whether it's drawn translucent whatever its pixels.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Material {
    pub sprite: Identifier,
    pub force_translucent: bool,
}

/// `Material.Baked`: the sprite the material found in an atlas.
#[derive(Clone)]
pub struct Baked {
    pub sprite: Arc<TextureAtlasSprite>,
    pub force_translucent: bool,
}

impl Material {
    pub fn new(sprite: Identifier) -> Material {
        Material {
            sprite,
            force_translucent: false,
        }
    }

    pub fn with_suffix(&self, suffix: &str) -> Material {
        Material {
            sprite: self.sprite.with_suffix(suffix),
            force_translucent: self.force_translucent,
        }
    }

    pub fn with_force_translucent(&self, force_translucent: bool) -> Material {
        Material {
            sprite: self.sprite.clone(),
            force_translucent,
        }
    }

    /// `CODEC.parse(JsonOps.INSTANCE, value)`: a JSON string that's an identifier, else an object
    /// with an identifier `sprite` and an optional boolean `force_translucent`.
    pub fn parse(value: &JsonElement) -> Result<Material, String> {
        if let Some(text) = value
            .as_json_primitive()
            .filter(|primitive| primitive.is_string())
        {
            let text = text.get_as_string();
            if let Some(sprite) = Identifier::try_parse(&text) {
                return Ok(Material::new(sprite));
            }
        }
        let object = value
            .as_json_object()
            .ok_or_else(|| format!("Not a material: {value}"))?;
        let sprite = object
            .get("sprite")
            .and_then(JsonElement::as_json_primitive)
            .filter(|primitive| primitive.is_string())
            .map(|primitive| primitive.get_as_string())
            .ok_or("No key sprite in MapLike")?;
        let sprite = Identifier::try_parse(&sprite)
            .ok_or_else(|| format!("Not a valid resource location: {sprite}"))?;
        let force_translucent = match object.get("force_translucent") {
            None => false,
            // JsonOps reads a boolean, or a number as whether its byte isn't 0.
            Some(element) => match element.as_json_primitive() {
                Some(primitive) if primitive.is_boolean() => primitive.get_as_boolean(),
                Some(primitive) if primitive.is_number() => primitive.get_as_int() as i8 != 0,
                _ => return Err(format!("Not a boolean: {element}")),
            },
        };
        Ok(Material {
            sprite,
            force_translucent,
        })
    }
}
