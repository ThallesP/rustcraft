//! net/minecraft/client/renderer/feature/FeatureFrameContext.java
//!
//! What the feature renderers read of the game while they prepare and draw a frame: the
//! textures, the overlay and the lightmap, the pipelines and the staged vertices. Java's also
//! holds the options, the font, the block models and colors, and the atlases, which come with the
//! features that read them; the shared quad indices and the pipelines are `RenderSystem`'s in
//! Java, the game renderer's here.

use std::rc::Rc;

use crate::com::mojang::blaze3d::projection_type::ProjectionType;
use crate::com::mojang::blaze3d::systems::render_system::AutoStorageIndexBuffer;
use crate::com::mojang::blaze3d::systems::sampler_cache::SamplerCache;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::renderer::game_renderer::Pipelines;
use crate::net::minecraft::client::renderer::staged_vertex_buffer::StagedVertexBuffer;
use crate::net::minecraft::client::renderer::texture::texture_manager::TextureManager;

pub struct FeatureFrameContext<'a> {
    pub device: &'a dyn GpuDevice,
    pub texture_manager: &'a mut TextureManager,
    pub sampler_cache: &'a SamplerCache,
    pub overlay_texture: &'a GpuTextureView,
    pub lightmap_texture: &'a GpuTextureView,
    pub pipelines: &'a mut Pipelines,
    pub staged_vertex_buffer: &'a mut StagedVertexBuffer,
    pub sequential_buffers: SequentialBuffers<'a>,
    /// `RenderSystem.getProjectionType()`: how translucent quads sort, and how render types
    /// layer.
    pub projection_type: ProjectionType,
    /// `gameRenderer.useImprovedTransparency()`.
    pub use_improved_transparency: bool,
    /// The last transforms written and their buffer: `DynamicGpuDataStorage.writeData` hands the
    /// last slice back for data equal to the last, so render types prepared one after another
    /// with the same transforms compare equal.
    pub last_transform: Option<(Vec<u8>, Rc<dyn GpuBuffer>)>,
}

/// `RenderSystem.getSequentialBuffer`'s shared indices, of the topologies features draw: quads,
/// and lines (two triangles each, like quads).
pub struct SequentialBuffers<'a> {
    pub quads: &'a mut AutoStorageIndexBuffer,
    pub lines: &'a mut AutoStorageIndexBuffer,
}

impl FeatureFrameContext<'_> {
    /// `RenderSystem.getDynamicUniforms().writeTransform`'s block of `data`.
    pub fn write_transform(&mut self, data: Vec<u8>) -> Rc<dyn GpuBuffer> {
        if let Some((last, buffer)) = &self.last_transform
            && *last == data
        {
            return Rc::clone(buffer);
        }
        let buffer = self.device.create_buffer_with_data(
            Some("Dynamic Transforms UBO"),
            gpu_buffer::USAGE_UNIFORM,
            &data,
        );
        self.last_transform = Some((data, Rc::clone(&buffer)));
        buffer
    }
}
