//! net/minecraft/client/gui/font/GlyphStitcher.java
//!
//! A font set's pages: a glyph goes on the first page it fits on, or a new one of its kind. Java
//! registers each page with the texture manager as `<font>/<index>`, which only the dump command
//! looks them up by; the pages are the stitcher's here.

use std::rc::Rc;

use crate::com::mojang::blaze3d::font::glyph_bitmap::GlyphBitmap;
use crate::com::mojang::blaze3d::font::glyph_info::GlyphInfo;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::net::minecraft::client::gui::font::font_texture::FontTexture;
use crate::net::minecraft::client::gui::font::glyphs::baked_sheet_glyph::BakedSheetGlyph;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::resources::identifier::Identifier;

pub struct GlyphStitcher {
    device: Option<&'static dyn GpuDevice>,
    texture_prefix: Identifier,
    textures: Vec<FontTexture>,
}

impl GlyphStitcher {
    /// The pages go on `device`, or nowhere for a layout alone.
    pub fn new(
        device: Option<&'static dyn GpuDevice>,
        texture_prefix: Identifier,
    ) -> GlyphStitcher {
        GlyphStitcher {
            device,
            texture_prefix,
            textures: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.textures.clear();
    }

    pub fn stitch(
        &mut self,
        info: GlyphInfo,
        glyph_bitmap: &dyn GlyphBitmap,
    ) -> Option<Rc<BakedSheetGlyph>> {
        for texture in &mut self.textures {
            if let Some(glyph) = texture.add(self.device, info, glyph_bitmap) {
                return Some(glyph);
            }
        }
        let next_index = self.textures.len();
        let name = self.texture_name(next_index);
        let is_colored = glyph_bitmap.is_colored();
        // GlyphRenderTypes.createForColorTexture or createForGrayscaleTexture's GUI pipeline.
        let gui_pipeline = if is_colored {
            &RenderPipelines::get().GUI_TEXT
        } else {
            &RenderPipelines::get().GUI_TEXT_GRAYSCALE
        };
        self.textures.push(FontTexture::new(
            self.device,
            &name.to_string(),
            std::sync::Arc::clone(gui_pipeline),
            is_colored,
        ));
        self.textures
            .last_mut()
            .expect("just pushed")
            .add(self.device, info, glyph_bitmap)
    }

    /// The pages so far, for the goldens.
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    fn texture_name(&self, index: usize) -> Identifier {
        self.texture_prefix.with_suffix(&format!("/{index}"))
    }
}
