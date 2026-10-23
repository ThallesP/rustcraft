//! com/mojang/blaze3d/font/SpaceProvider.java
//!
//! Glyphs that only advance (`include/space`: the space and the zero-width non-joiner). Its
//! definition's `advances` is decoded by hand from serde_json, like the other providers'.

use std::collections::BTreeMap;
use std::rc::Rc;

use crate::com::mojang::blaze3d::font::glyph_provider::GlyphProvider;
use crate::com::mojang::blaze3d::font::unbaked_glyph::UnbakedGlyph;
use crate::net::minecraft::client::gui::font::glyphs::empty_glyph::EmptyGlyph;

pub struct SpaceProvider {
    glyphs: BTreeMap<i32, Rc<EmptyGlyph>>,
}

impl SpaceProvider {
    pub fn new(advances: &[(i32, f32)]) -> SpaceProvider {
        SpaceProvider {
            glyphs: advances
                .iter()
                .map(|&(codepoint, advance)| (codepoint, Rc::new(EmptyGlyph::new(advance))))
                .collect(),
        }
    }
}

impl GlyphProvider for SpaceProvider {
    fn get_glyph(&self, codepoint: i32) -> Option<Rc<dyn UnbakedGlyph>> {
        self.glyphs
            .get(&codepoint)
            .map(|glyph| Rc::clone(glyph) as Rc<dyn UnbakedGlyph>)
    }

    fn get_supported_glyphs(&self) -> Vec<i32> {
        self.glyphs.keys().copied().collect()
    }
}

/// `SpaceProvider.Definition`: each codepoint's advance (`ExtraCodecs.CODEPOINT` keys: strings of
/// one codepoint).
#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    pub advances: Vec<(i32, f32)>,
}

impl Definition {
    pub fn decode(
        object: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<Definition, String> {
        let advances = object
            .get("advances")
            .ok_or("No key advances in MapLike")?
            .as_object()
            .ok_or("advances isn't a map")?;
        let mut result = Vec::new();
        for (key, value) in advances {
            let mut codepoints = key.chars();
            let (Some(codepoint), None) = (codepoints.next(), codepoints.next()) else {
                return Err(format!("Expected one codepoint, got: {key}"));
            };
            let advance = value
                .as_f64()
                .ok_or_else(|| format!("Not a number: {value}"))? as f32;
            result.push((codepoint as i32, advance));
        }
        Ok(Definition { advances: result })
    }

    pub fn load(&self) -> Rc<dyn GlyphProvider> {
        Rc::new(SpaceProvider::new(&self.advances))
    }
}
