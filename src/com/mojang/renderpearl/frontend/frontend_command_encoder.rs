//! com/mojang/renderpearl/frontend/FrontendCommandEncoder.java
//!
//! The checks Java's frontend encoder makes before the backend's runs a call, as functions the
//! backends' encoders call until the frontend encoder itself is ported (the Vulkan backend's; the
//! GL backend's encoder still has its own copies). Each panics where Java throws.

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::commands::render_pass_descriptor::RenderPassDescriptor;
use crate::com::mojang::renderpearl::api::gpu_format::ComponentType;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::{self, GpuTexture};
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;

pub struct FrontendCommandEncoder;

impl FrontendCommandEncoder {
    pub fn check_not_in_render_pass(is_in_render_pass: bool) {
        assert!(
            !is_in_render_pass,
            "Close the existing render pass before performing additional commands"
        );
    }

    /// `createRenderPass`'s checks: no other pass open, attachments within the device's count,
    /// at least one, render attachments of one size the render area fits in.
    pub fn check_render_pass(
        is_in_render_pass: bool,
        descriptor: &RenderPassDescriptor,
        max_color_attachments: i32,
    ) {
        assert!(
            !is_in_render_pass,
            "Close the existing render pass before creating a new one!"
        );
        let color_attachment_count = descriptor.color_attachments.len();
        assert!(
            color_attachment_count <= max_color_attachments as usize,
            "Render pass created with {color_attachment_count} color attachments but device only supports {max_color_attachments}"
        );
        let total_attachments =
            color_attachment_count + usize::from(descriptor.depth_attachment.is_some());
        assert!(
            total_attachments != 0,
            "At least one attachment (depth or color) must be specified"
        );
        let first_view = match descriptor.color_attachments.first() {
            Some(first) => {
                &first
                    .as_ref()
                    .expect("a first color attachment")
                    .texture_view
            }
            None => {
                &descriptor
                    .depth_attachment
                    .as_ref()
                    .expect("a depth attachment")
                    .texture_view
            }
        };
        let (attachment_width, attachment_height) =
            (first_view.get_width(0), first_view.get_height(0));
        let render_area = descriptor.render_area;
        assert!(
            render_area.x >= 0
                && render_area.y >= 0
                && render_area.x + render_area.width <= attachment_width
                && render_area.y + render_area.height <= attachment_height,
            "RenderPass render area {render_area:?} is out of bounds for texture of {attachment_width}x{attachment_height}"
        );
        let check_attachment = |name: String, view: &GpuTextureView| {
            assert!(
                view.texture().get_usage() & gpu_texture::USAGE_RENDER_ATTACHMENT != 0,
                "{name} must have USAGE_RENDER_ATTACHMENT"
            );
            assert!(
                view.texture().get_depth_or_layers() <= 1,
                "{name}: Textures with multiple depths or layers are not yet supported as an attachment"
            );
            assert!(
                view.get_width(0) == attachment_width && view.get_height(0) == attachment_height,
                "{name}: size does not match expected attachment size. Is {}x{} expected {attachment_width}x{attachment_height}",
                view.get_width(0),
                view.get_height(0)
            );
        };
        for (i, attachment) in descriptor.color_attachments.iter().enumerate() {
            if let Some(attachment) = attachment {
                check_attachment(format!("Color texture {i}"), &attachment.texture_view);
            }
        }
        if let Some(depth) = &descriptor.depth_attachment {
            check_attachment("Depth texture".to_owned(), &depth.texture_view);
        }
    }

    /// `writeToBuffer`'s: a copy destination, the data within the slice and the slice within the
    /// buffer.
    pub fn check_write_to_buffer(slice: &GpuBufferSlice, length: i64) {
        let buffer = &slice.buffer;
        assert!(
            buffer.usage() & gpu_buffer::USAGE_COPY_DST != 0,
            "Buffer needs USAGE_COPY_DST to be a destination for a copy"
        );
        assert!(
            length <= slice.length,
            "Cannot write more data than the slice allows (attempting to write {length} bytes into a slice of length {})",
            slice.length
        );
        assert!(
            slice.length + slice.offset <= buffer.size(),
            "Cannot write more data than this buffer can hold (attempting to write {length} bytes at offset {} to {} size buffer)",
            slice.offset,
            buffer.size()
        );
    }

    /// `writeToTexture(GpuTexture, NativeImage, …)`'s: an 8-bit texture with the image's
    /// channels, a mip level and layer it has, a rectangle that fits, a copy destination.
    pub fn check_write_to_texture_image(
        destination: &dyn GpuTexture,
        source: &NativeImage,
        mip_level: i32,
        depth_or_layer: i32,
        dest_x: i32,
        dest_y: i32,
    ) {
        let format = destination.get_format();
        assert!(
            format.component_type() == ComponentType::UNORM_8,
            "Destination texture for NativeImage writes must have component type of UNORM_8"
        );
        assert!(
            format.component_count() == source.format().components(),
            "Destination({format:?}) texture for NativeImage({:?}) write must have channel count matching source",
            source.format()
        );
        assert!(
            mip_level >= 0 && mip_level < destination.get_mip_levels(),
            "Invalid mipLevel {mip_level}, must be >= 0 and < {}",
            destination.get_mip_levels()
        );
        assert!(
            dest_x + source.get_width() <= destination.get_width(mip_level)
                && dest_y + source.get_height() <= destination.get_height(mip_level),
            "Dest texture ({}x{}) is not large enough to write a rectangle of {}x{} at {dest_x}x{dest_y} (at mip level {mip_level})",
            source.get_width(),
            source.get_height(),
            source.get_width(),
            source.get_height()
        );
        assert!(
            destination.get_usage() & gpu_texture::USAGE_COPY_DST != 0,
            "Color texture must have USAGE_COPY_DST to be a destination for a write"
        );
        assert!(
            depth_or_layer < destination.get_depth_or_layers(),
            "Depth or layer is out of range, must be >= 0 and < {}",
            destination.get_depth_or_layers()
        );
    }

    /// `writeToTexture(GpuTexture, ByteBuffer, …)`'s: a mip level and layer the texture has,
    /// enough bytes, a rectangle that fits, a copy destination.
    #[allow(clippy::too_many_arguments)]
    pub fn check_write_to_texture(
        destination: &dyn GpuTexture,
        source_length: usize,
        mip_level: i32,
        depth_or_layer: i32,
        dest_x: i32,
        dest_y: i32,
        width: i32,
        height: i32,
    ) {
        let format = destination.get_format();
        assert!(
            mip_level >= 0 && mip_level < destination.get_mip_levels(),
            "Invalid mipLevel {mip_level}, must be >= 0 and < {}",
            destination.get_mip_levels()
        );
        assert!(
            (width * height * format.block_size()) as usize <= source_length,
            "Copy would overrun the source buffer (remaining length of {source_length}, but copy is {width}x{height} of format {format:?})"
        );
        assert!(
            dest_x + width <= destination.get_width(mip_level)
                && dest_y + height <= destination.get_height(mip_level),
            "Dest texture ({}x{}) is not large enough to write a rectangle of {width}x{height} at {dest_x}x{dest_y}",
            destination.get_width(mip_level),
            destination.get_height(mip_level)
        );
        assert!(
            destination.get_usage() & gpu_texture::USAGE_COPY_DST != 0,
            "Color texture must have USAGE_COPY_DST to be a destination for a write"
        );
        assert!(
            depth_or_layer < destination.get_depth_or_layers(),
            "Depth or layer is out of range, must be >= 0 and < {}",
            destination.get_depth_or_layers()
        );
    }
}
