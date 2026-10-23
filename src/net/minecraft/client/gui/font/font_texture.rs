//! net/minecraft/client/gui/font/FontTexture.java
//!
//! A 256 by 256 page of glyphs, colored (RGBA) or grayscale (R8), filled by its
//! `DynamicAtlasTree` a pixel apart, each glyph uploaded where it goes and baked with UVs nudged a
//! hundredth of a pixel inside its place. Without a device (the goldens') a page only lays its
//! glyphs out. Its sampler (`getRepeat(NEAREST)`) is for the level's text; the GUI's glyphs bind
//! their own. `dumpContents` comes with the dump command.

use std::rc::Rc;
use std::sync::Arc;

use crate::com::mojang::blaze3d::font::glyph_bitmap::GlyphBitmap;
use crate::com::mojang::blaze3d::font::glyph_info::GlyphInfo;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::{self, GpuTexture};
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::gui::font::glyphs::baked_sheet_glyph::BakedSheetGlyph;
use crate::net::minecraft::client::renderer::texture::dynamic_atlas_tree::DynamicAtlasTree;

pub struct FontTexture {
    texture: Option<(Rc<dyn GpuTexture>, GpuTextureView)>,
    gui_pipeline: Arc<RenderPipeline>,
    colored: bool,
    tree: DynamicAtlasTree,
}

impl FontTexture {
    pub const SIZE: i32 = 256;
    const SPACING: i32 = 1;

    /// A page on the device, if there's one; `gui_pipeline` is `GlyphRenderTypes.guiPipeline()`.
    pub fn new(
        device: Option<&dyn GpuDevice>,
        label: &str,
        gui_pipeline: Arc<RenderPipeline>,
        colored: bool,
    ) -> FontTexture {
        let texture = device.map(|device| {
            let texture = device.create_texture(
                Some(label),
                gpu_texture::USAGE_COPY_DST
                    | gpu_texture::USAGE_COPY_SRC
                    | gpu_texture::USAGE_TEXTURE_BINDING,
                if colored {
                    GpuFormat::RGBA8_UNORM
                } else {
                    GpuFormat::R8_UNORM
                },
                FontTexture::SIZE,
                FontTexture::SIZE,
                1,
                1,
            );
            let view = device.create_texture_view(&texture);
            (texture, view)
        });
        FontTexture {
            texture,
            gui_pipeline,
            colored,
            tree: DynamicAtlasTree::new(0, 0, FontTexture::SIZE, FontTexture::SIZE),
        }
    }

    pub fn add(
        &mut self,
        device: Option<&dyn GpuDevice>,
        info: GlyphInfo,
        glyph: &dyn GlyphBitmap,
    ) -> Option<Rc<BakedSheetGlyph>> {
        if glyph.is_colored() != self.colored {
            return None;
        }
        let slot = self.tree.insert(
            glyph.get_pixel_width(),
            glyph.get_pixel_height(),
            FontTexture::SPACING,
        )?;
        if let (Some(device), Some((texture, _))) = (device, &self.texture) {
            glyph.upload(device, slot.x, slot.y, &**texture);
        }
        let size = FontTexture::SIZE as f32;
        let nudge = 0.01;
        Some(Rc::new(BakedSheetGlyph::new(
            info,
            Arc::clone(&self.gui_pipeline),
            self.texture.as_ref().map(|(_, view)| view.clone()),
            (slot.x as f32 + nudge) / size,
            (slot.x as f32 - nudge + glyph.get_pixel_width() as f32) / size,
            (slot.y as f32 + nudge) / size,
            (slot.y as f32 - nudge + glyph.get_pixel_height() as f32) / size,
            glyph.get_left(),
            glyph.get_right(),
            glyph.get_top(),
            glyph.get_bottom(),
        )))
    }
}
