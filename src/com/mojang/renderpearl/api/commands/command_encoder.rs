//! com/mojang/renderpearl/api/commands/CommandEncoder.java
//!
//! Render passes, clears, writing buffers and textures, and copying a texture into a buffer. A
//! copy's callback runs once the GPU is done with it, which `submit` waits for (Java queues it
//! as a fenced task that `RenderSystem` runs later). The other copies, fences and timestamps come
//! with their callers.

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::GpuBuffer;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::commands::render_pass::{RenderArea, RenderPass};
use crate::com::mojang::renderpearl::api::commands::render_pass_descriptor::RenderPassDescriptor;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::GpuTexture;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::org::joml::vector4f::Vector4f;
use std::rc::Rc;

pub trait CommandEncoder {
    fn submit(&self);

    /// `createRenderPass(label, colorTexture, clearColor, depthTexture, clearDepth)`, over the
    /// whole color texture.
    fn create_render_pass_with(
        &self,
        label: &str,
        color_texture: &GpuTextureView,
        clear_color: Option<Vector4f>,
        depth_texture: Option<&GpuTextureView>,
        clear_depth: Option<f64>,
    ) -> Box<dyn RenderPass + '_> {
        let mut descriptor = RenderPassDescriptor::builder(label)
            .with_color_attachment(color_texture.clone(), clear_color);
        if let Some(depth_texture) = depth_texture {
            descriptor = descriptor.with_depth_attachment(depth_texture.clone(), clear_depth);
        }
        let render_area = RenderArea {
            x: 0,
            y: 0,
            width: color_texture.get_width(0),
            height: color_texture.get_height(0),
        };
        self.create_render_pass(descriptor.with_render_area(render_area).build())
    }

    fn create_render_pass(&self, descriptor: RenderPassDescriptor) -> Box<dyn RenderPass + '_>;

    fn clear_color_texture(&self, color_texture: &dyn GpuTexture, clear_color: &Vector4f);

    fn clear_color_and_depth_textures(
        &self,
        color_texture: &dyn GpuTexture,
        clear_color: &Vector4f,
        depth_texture: &dyn GpuTexture,
        clear_depth: f64,
    );

    /// `writeToBuffer(GpuBufferSlice, ByteBuffer)`: the data at the slice's start.
    fn write_to_buffer(&self, slice: &GpuBufferSlice, data: &[u8]);

    /// `writeToTexture(GpuTexture, NativeImage)`: the whole image over the whole texture. Panics
    /// where Java throws `IllegalArgumentException` for an image of another size.
    fn write_to_texture_whole(&self, destination: &dyn GpuTexture, source: &NativeImage) {
        let width = destination.get_width(0);
        let height = destination.get_height(0);
        assert!(
            source.get_width() == width && source.get_height() == height,
            "Cannot replace texture of size {width}x{height} with image of size {}x{}",
            source.get_width(),
            source.get_height()
        );
        self.write_to_texture_image(destination, source, 0, 0, 0, 0);
    }

    /// `writeToTexture(GpuTexture, NativeImage, int, int, int, int)`.
    fn write_to_texture_image(
        &self,
        destination: &dyn GpuTexture,
        source: &NativeImage,
        mip_level: i32,
        depth_or_layer: i32,
        dest_x: i32,
        dest_y: i32,
    );

    /// `writeToTexture(GpuTexture, ByteBuffer, int, int, int, int, int, int)`.
    #[allow(clippy::too_many_arguments)]
    fn write_to_texture(
        &self,
        destination: &dyn GpuTexture,
        source: &[u8],
        mip_level: i32,
        depth_or_layer: i32,
        dest_x: i32,
        dest_y: i32,
        width: i32,
        height: i32,
    );

    fn copy_texture_to_buffer(
        &self,
        source: &dyn GpuTexture,
        destination: &Rc<dyn GpuBuffer>,
        offset: i64,
        callback: Box<dyn FnOnce()>,
        mip_level: i32,
    );
}
