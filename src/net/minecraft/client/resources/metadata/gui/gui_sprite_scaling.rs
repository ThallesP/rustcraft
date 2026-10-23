//! net/minecraft/client/resources/metadata/gui/GuiSpriteScaling.java
//!
//! How a GUI sprite fills a size other than its own: stretched, tiled, or nine-sliced. The
//! interface's three records are an enum's variants, and `CODEC`, a dispatch on `type`, is decoded
//! by hand from serde_json (`GuiMetadataSection`).

use serde_json::{Map, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiSpriteScaling {
    Stretch,
    Tile(Tile),
    NineSlice(NineSlice),
}

/// `GuiSpriteScaling.Tile`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tile {
    pub width: i32,
    pub height: i32,
}

/// `GuiSpriteScaling.NineSlice`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NineSlice {
    pub width: i32,
    pub height: i32,
    pub border: Border,
    pub stretch_inner: bool,
}

/// `GuiSpriteScaling.NineSlice.Border`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Border {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// `GuiSpriteScaling.Type`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    STRETCH,
    TILE,
    NINE_SLICE,
}

impl Type {
    pub fn get_serialized_name(self) -> &'static str {
        match self {
            Type::STRETCH => "stretch",
            Type::TILE => "tile",
            Type::NINE_SLICE => "nine_slice",
        }
    }
}

impl GuiSpriteScaling {
    pub const DEFAULT: GuiSpriteScaling = GuiSpriteScaling::Stretch;

    pub fn r#type(&self) -> Type {
        match self {
            GuiSpriteScaling::Stretch => Type::STRETCH,
            GuiSpriteScaling::Tile(_) => Type::TILE,
            GuiSpriteScaling::NineSlice(_) => Type::NINE_SLICE,
        }
    }

    /// `CODEC`: `Type.CODEC.dispatch(type, Type::codec)`.
    pub fn codec(value: &Value) -> Result<GuiSpriteScaling, String> {
        let object = value
            .as_object()
            .ok_or_else(|| format!("Not a map: {value}"))?;
        let r#type = match object.get("type") {
            Some(Value::String(name)) => match name.as_str() {
                "stretch" => Type::STRETCH,
                "tile" => Type::TILE,
                "nine_slice" => Type::NINE_SLICE,
                _ => return Err(format!("Unknown element name:{name}")),
            },
            Some(other) => return Err(format!("Not a string: {other}")),
            None => return Err("No key type in MapLike".to_owned()),
        };
        match r#type {
            Type::STRETCH => Ok(GuiSpriteScaling::Stretch),
            Type::TILE => Ok(GuiSpriteScaling::Tile(Tile {
                width: positive_int(field(object, "width")?)?,
                height: positive_int(field(object, "height")?)?,
            })),
            Type::NINE_SLICE => {
                let nine_slice = NineSlice {
                    width: positive_int(field(object, "width")?)?,
                    height: positive_int(field(object, "height")?)?,
                    border: Border::codec(field(object, "border")?)?,
                    stretch_inner: match object.get("stretch_inner") {
                        Some(Value::Bool(stretch_inner)) => *stretch_inner,
                        Some(other) => return Err(format!("Not a boolean: {other}")),
                        None => false,
                    },
                };
                nine_slice.validate().map(GuiSpriteScaling::NineSlice)
            }
        }
    }
}

impl NineSlice {
    fn validate(self) -> Result<NineSlice, String> {
        let border = self.border;
        if border.left + border.right >= self.width {
            Err(format!(
                "Nine-sliced texture has no horizontal center slice: {} + {} >= {}",
                border.left, border.right, self.width
            ))
        } else if border.top + border.bottom >= self.height {
            Err(format!(
                "Nine-sliced texture has no vertical center slice: {} + {} >= {}",
                border.top, border.bottom, self.height
            ))
        } else {
            Ok(self)
        }
    }
}

impl Border {
    /// `CODEC`: either one size for every side (`VALUE_CODEC`) or each side's (`RECORD_CODEC`).
    fn codec(value: &Value) -> Result<Border, String> {
        if let Ok(size) = positive_int(value) {
            return Ok(Border {
                left: size,
                top: size,
                right: size,
                bottom: size,
            });
        }
        let object = value
            .as_object()
            .ok_or_else(|| format!("Not a map: {value}"))?;
        Ok(Border {
            left: non_negative_int(field(object, "left")?)?,
            top: non_negative_int(field(object, "top")?)?,
            right: non_negative_int(field(object, "right")?)?,
            bottom: non_negative_int(field(object, "bottom")?)?,
        })
    }
}

fn field<'a>(object: &'a Map<String, Value>, name: &str) -> Result<&'a Value, String> {
    object
        .get(name)
        .ok_or_else(|| format!("No key {name} in MapLike"))
}

/// `Codec.INT`: a number's `intValue()`.
fn int(value: &Value) -> Result<i32, String> {
    match value {
        Value::Number(number) => Ok(match number.as_i64() {
            Some(int) => int as i32,
            None => number.as_f64().unwrap_or(0.0) as i32,
        }),
        other => Err(format!("Not a number: {other}")),
    }
}

/// `ExtraCodecs.POSITIVE_INT`.
fn positive_int(value: &Value) -> Result<i32, String> {
    let int = int(value)?;
    if int > 0 {
        Ok(int)
    } else {
        Err(format!("Value must be positive: {int}"))
    }
}

/// `ExtraCodecs.NON_NEGATIVE_INT`.
fn non_negative_int(value: &Value) -> Result<i32, String> {
    let int = int(value)?;
    if int >= 0 {
        Ok(int)
    } else {
        Err(format!("Value must be non-negative: {int}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::golden::Golden;
    use crate::net::minecraft::client::renderer::texture::sprite_contents::AdditionalMetadataType;
    use crate::net::minecraft::client::renderer::texture::sprite_loader::SpriteLoader;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::server::packs::resources::multi_pack_resource_manager::tests::vanilla_resources;
    use serde_json::json;

    fn encode(scaling: &GuiSpriteScaling) -> Value {
        match scaling {
            GuiSpriteScaling::Stretch => json!(["stretch"]),
            GuiSpriteScaling::Tile(tile) => json!(["tile", tile.width, tile.height]),
            GuiSpriteScaling::NineSlice(nine_slice) => {
                let border = nine_slice.border;
                json!([
                    "nine_slice",
                    nine_slice.width,
                    nine_slice.height,
                    [border.left, border.top, border.right, border.bottom],
                    nine_slice.stretch_inner
                ])
            }
        }
    }

    /// Every sprite the GUI atlas loads, with its `gui` section's scaling (none without one).
    #[test]
    fn reads_every_gui_sprites_scaling_like_vanilla() {
        let golden = Golden::load("net.minecraft.client.resources.metadata.gui.GuiSpriteScaling");
        let sprites = SpriteLoader::load_sprites(
            &vanilla_resources(),
            &Identifier::with_default_namespace("gui"),
            &[AdditionalMetadataType::Gui],
        );
        golden.check("scaling", |name: String| {
            let name = Identifier::parse(&name);
            let sprite = sprites
                .iter()
                .find(|sprite| *sprite.name() == name)
                .expect("the port loads the sprite");
            sprite
                .get_gui_metadata()
                .map_or(Value::Null, |section| encode(&section.scaling))
        });
        golden.finish();
    }
}
