//! com/mojang/renderpearl/backend/opengl/GlTexture.java
//!
//! The texture's name is deleted when it's dropped (Java's `close()`), after the framebuffers it's
//! attached to (`FrameBufferCache`).

use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::GpuTexture;
use crate::com::mojang::renderpearl::backend::opengl::frame_buffer_cache::FrameBufferCache;
use glow::HasContext;
use std::any::Any;
use std::rc::Rc;

pub struct GlTexture {
    gl: Rc<glow::Context>,
    frame_buffer_cache: Rc<FrameBufferCache>,
    pub(super) id: glow::NativeTexture,
    usage: i32,
    label: String,
    format: GpuFormat,
    width: i32,
    height: i32,
    depth_or_layers: i32,
    mip_levels: i32,
}

impl GlTexture {
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        gl: Rc<glow::Context>,
        frame_buffer_cache: Rc<FrameBufferCache>,
        usage: i32,
        label: String,
        format: GpuFormat,
        width: i32,
        height: i32,
        depth_or_layers: i32,
        mip_levels: i32,
        id: glow::NativeTexture,
    ) -> GlTexture {
        GlTexture {
            gl,
            frame_buffer_cache,
            id,
            usage,
            label,
            format,
            width,
            height,
            depth_or_layers,
            mip_levels,
        }
    }

    pub fn gl_id(&self) -> glow::NativeTexture {
        self.id
    }
}

impl GpuTexture for GlTexture {
    fn get_usage(&self) -> i32 {
        self.usage
    }

    fn get_label(&self) -> &str {
        &self.label
    }

    fn get_format(&self) -> GpuFormat {
        self.format
    }

    fn get_width(&self, mip_level: i32) -> i32 {
        self.width >> mip_level
    }

    fn get_height(&self, mip_level: i32) -> i32 {
        self.height >> mip_level
    }

    fn get_depth_or_layers(&self) -> i32 {
        self.depth_or_layers
    }

    fn get_mip_levels(&self) -> i32 {
        self.mip_levels
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for GlTexture {
    /// `destroyImmediately`.
    fn drop(&mut self) {
        self.frame_buffer_cache.destroy_fbos_of(&self.gl, self.id);
        // SAFETY: the texture is the context's, which outlives it.
        unsafe { self.gl.delete_texture(self.id) }
    }
}
