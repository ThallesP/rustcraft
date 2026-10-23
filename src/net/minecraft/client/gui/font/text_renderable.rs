//! net/minecraft/client/gui/font/TextRenderable.java
//!
//! What a prepared text hands its visitor: a glyph in a style (`Styled`) or an effect's rectangle.
//! Java's interface is implemented by `BakedSheetGlyph`'s two records, which are this enum's
//! variants. `ActiveArea`'s methods (the area a click finds the glyph's style in) come with
//! clickable text.

use std::sync::Arc;

use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::gui::font::glyphs::baked_sheet_glyph::{
    EffectInstance, GlyphInstance,
};
use crate::org::joml::matrix4f::Matrix4f;

// A glyph's instance holds its style, which is most of it; boxing it would allocate per glyph.
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum TextRenderable {
    Glyph(GlyphInstance),
    Effect(EffectInstance),
}

impl TextRenderable {
    pub fn render(
        &self,
        pose: &Matrix4f,
        buffer: &mut dyn VertexConsumer,
        packed_light_coords: i32,
        flat: bool,
    ) {
        match self {
            TextRenderable::Glyph(glyph) => glyph.render(pose, buffer, packed_light_coords, flat),
            TextRenderable::Effect(effect) => {
                effect.render(pose, buffer, packed_light_coords, flat)
            }
        }
    }

    /// The font texture's view, which the goldens' textures without a device haven't got.
    pub fn texture_view(&self) -> Option<&GpuTextureView> {
        match self {
            TextRenderable::Glyph(glyph) => glyph.glyph.texture_view(),
            TextRenderable::Effect(effect) => effect.glyph.texture_view(),
        }
    }

    pub fn gui_pipeline(&self) -> &Arc<RenderPipeline> {
        match self {
            TextRenderable::Glyph(glyph) => glyph.glyph.gui_pipeline(),
            TextRenderable::Effect(effect) => effect.glyph.gui_pipeline(),
        }
    }

    pub fn left(&self) -> f32 {
        match self {
            TextRenderable::Glyph(glyph) => glyph.left(),
            TextRenderable::Effect(effect) => effect.left(),
        }
    }

    pub fn top(&self) -> f32 {
        match self {
            TextRenderable::Glyph(glyph) => glyph.top(),
            TextRenderable::Effect(effect) => effect.top(),
        }
    }

    pub fn right(&self) -> f32 {
        match self {
            TextRenderable::Glyph(glyph) => glyph.right(),
            TextRenderable::Effect(effect) => effect.right(),
        }
    }

    pub fn bottom(&self) -> f32 {
        match self {
            TextRenderable::Glyph(glyph) => glyph.bottom(),
            TextRenderable::Effect(effect) => effect.bottom(),
        }
    }
}
