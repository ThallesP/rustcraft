//! com/mojang/renderpearl/backend/opengl/GlCommandEncoder.java
//!
//! Only render passes that clear, clearing textures, copying a texture into a buffer and
//! presenting so far. GL runs commands as they're issued, so `submit` only has to wait for the
//! GPU before running the copies' callbacks, which Java queues as fenced tasks for
//! `RenderSystem` to run once their fence signals.
//!
//! Java's `GlStateManager` skips calls that wouldn't change GL's state; the port calls GL
//! directly, which leaves the same state. `FrameBufferCache` is a list of framebuffers by their
//! attachments here, and `DirectStateAccess`'s binds use the classic entry points, which do the
//! same.

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::GpuBuffer;
use crate::com::mojang::renderpearl::api::commands::command_encoder::CommandEncoder;
use crate::com::mojang::renderpearl::api::commands::render_pass::RenderPass;
use crate::com::mojang::renderpearl::api::commands::render_pass_descriptor::RenderPassDescriptor;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::GpuTexture;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::com::mojang::renderpearl::backend::opengl::gl_buffer::GlBuffer;
use crate::com::mojang::renderpearl::backend::opengl::gl_const::GlConst;
use crate::com::mojang::renderpearl::backend::opengl::gl_device::GlDevice;
use crate::com::mojang::renderpearl::backend::opengl::gl_render_pass::GlRenderPass;
use crate::com::mojang::renderpearl::backend::opengl::gl_texture::GlTexture;
use crate::org::joml::vector4f::Vector4f;
use crate::org::lwjgl::sdl::sdl_video::SdlWindow;
use glow::HasContext;
use std::cell::RefCell;
use std::rc::Rc;

/// A framebuffer's attachments: each color target's texture and mip level, and the depth's.
type FrameBufferKey = (
    Vec<Option<(glow::NativeTexture, i32)>>,
    Option<(glow::NativeTexture, i32)>,
);

pub struct GlCommandEncoder {
    gl: Rc<glow::Context>,
    read_fbo: glow::NativeFramebuffer,
    draw_fbo: glow::NativeFramebuffer,
    frame_buffers: RefCell<Vec<(FrameBufferKey, glow::NativeFramebuffer)>>,
    fenced_tasks: RefCell<Vec<Box<dyn FnOnce()>>>,
}

fn gl_texture(texture: &dyn GpuTexture) -> &GlTexture {
    texture
        .as_any()
        .downcast_ref::<GlTexture>()
        .expect("an OpenGL texture")
}

impl GlCommandEncoder {
    pub(super) fn new(gl: Rc<glow::Context>) -> GlCommandEncoder {
        // SAFETY: the device's context is current.
        let (read_fbo, draw_fbo) = unsafe {
            (
                gl.create_framebuffer().expect("glGenFramebuffers"),
                gl.create_framebuffer().expect("glGenFramebuffers"),
            )
        };
        GlCommandEncoder {
            gl,
            read_fbo,
            draw_fbo,
            frame_buffers: RefCell::default(),
            fenced_tasks: RefCell::default(),
        }
    }

    /// `DirectStateAccess.bindFrameBufferTextures`: binds the framebuffer to `target` and attaches
    /// the color and depth textures at the mip level, or detaches them.
    fn bind_frame_buffer_textures(
        &self,
        fbo: glow::NativeFramebuffer,
        color: Option<glow::NativeTexture>,
        depth: Option<glow::NativeTexture>,
        mip_level: i32,
        target: u32,
    ) {
        // SAFETY: the framebuffer and textures are the context's.
        unsafe {
            self.gl.bind_framebuffer(target, Some(fbo));
            self.gl.framebuffer_texture_2d(
                target,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                color,
                mip_level,
            );
            self.gl.framebuffer_texture_2d(
                target,
                glow::DEPTH_ATTACHMENT,
                glow::TEXTURE_2D,
                depth,
                mip_level,
            );
        }
    }

    /// `FrameBufferCache.getFbo`: the framebuffer with these attachments, made the first time.
    fn get_fbo(
        &self,
        colors: &[Option<&GpuTextureView>],
        depth: Option<&GpuTextureView>,
    ) -> glow::NativeFramebuffer {
        let attachment =
            |view: &GpuTextureView| (gl_texture(&**view.texture()).id, view.base_mip_level());
        let key: FrameBufferKey = (
            colors.iter().map(|view| view.map(attachment)).collect(),
            depth.map(attachment),
        );
        if let Some((_, fbo)) = self
            .frame_buffers
            .borrow()
            .iter()
            .find(|(existing, _)| *existing == key)
        {
            return *fbo;
        }
        // SAFETY: the textures are the context's.
        let fbo = unsafe {
            let fbo = self.gl.create_framebuffer().expect("glGenFramebuffers");
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            for (index, color) in key.0.iter().enumerate() {
                if let Some((texture, mip_level)) = color {
                    self.gl.framebuffer_texture_2d(
                        glow::FRAMEBUFFER,
                        glow::COLOR_ATTACHMENT0 + index as u32,
                        glow::TEXTURE_2D,
                        Some(*texture),
                        *mip_level,
                    );
                }
            }
            if let Some((texture, mip_level)) = key.1 {
                self.gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    glow::DEPTH_ATTACHMENT,
                    glow::TEXTURE_2D,
                    Some(texture),
                    mip_level,
                );
            }
            fbo
        };
        self.frame_buffers.borrow_mut().push((key, fbo));
        fbo
    }

    /// Blits the texture into the window's framebuffer, at its top left.
    pub(super) fn present_texture(
        &self,
        device: &GlDevice,
        window_handle: SdlWindow,
        texture_view: &GpuTextureView,
        swapchain_width: i32,
        swapchain_height: i32,
    ) {
        device.make_current(window_handle);
        let dest_y = (swapchain_height - texture_view.get_height(0)).max(0);
        let copy_width = swapchain_width.min(texture_view.get_width(0));
        let copy_height = swapchain_height.min(texture_view.get_height(0));
        // SAFETY: the device's context is current.
        unsafe {
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl
                .viewport(0, 0, texture_view.get_width(0), texture_view.get_height(0));
            self.gl.depth_mask(true);
            self.gl.color_mask(true, true, true, true);
            self.bind_frame_buffer_textures(
                self.draw_fbo,
                Some(gl_texture(&**texture_view.texture()).id),
                None,
                0,
                glow::READ_FRAMEBUFFER,
            );
            self.gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
            self.gl.blit_framebuffer(
                0,
                0,
                copy_width,
                copy_height,
                0,
                dest_y,
                copy_width,
                copy_height + dest_y,
                glow::COLOR_BUFFER_BIT,
                glow::NEAREST,
            );
            self.gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
        }
    }

    /// Deletes the encoder's framebuffers, while the device's context is still current.
    pub(super) fn close(&self) {
        // SAFETY: the framebuffers are the context's.
        unsafe {
            for (_, fbo) in self.frame_buffers.borrow_mut().drain(..) {
                self.gl.delete_framebuffer(fbo);
            }
            self.gl.delete_framebuffer(self.read_fbo);
            self.gl.delete_framebuffer(self.draw_fbo);
        }
    }
}

impl CommandEncoder for GlCommandEncoder {
    fn submit(&self) {
        // SAFETY: the device's context is current.
        unsafe { self.gl.finish() };
        for task in self.fenced_tasks.borrow_mut().drain(..) {
            task();
        }
    }

    fn create_render_pass(&self, descriptor: RenderPassDescriptor) -> Box<dyn RenderPass + '_> {
        let colors: Vec<Option<&GpuTextureView>> = descriptor
            .color_attachments
            .iter()
            .map(|attachment| {
                attachment
                    .as_ref()
                    .map(|attachment| &attachment.texture_view)
            })
            .collect();
        let depth = descriptor
            .depth_attachment
            .as_ref()
            .map(|attachment| &attachment.texture_view);
        let fbo = self.get_fbo(&colors, depth);

        let (mut width, mut height) = (0, 0);
        if !colors.is_empty() {
            for color in colors.iter().flatten() {
                width = color.get_width(0);
                height = color.get_height(0);
            }
        } else if let Some(depth) = depth {
            width = depth.get_width(0);
            height = depth.get_height(0);
        }

        let render_area = descriptor.render_area;
        // SAFETY: the device's context is current.
        unsafe {
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            self.gl.enable(glow::SCISSOR_TEST);
            self.gl.scissor(
                render_area.x,
                render_area.y,
                render_area.width,
                render_area.height,
            );
            for (index, attachment) in descriptor.color_attachments.iter().enumerate() {
                if let Some(Some(clear_value)) =
                    attachment.as_ref().map(|attachment| attachment.clear_value)
                {
                    self.gl
                        .color_mask_draw_buffer(index as u32, true, true, true, true);
                    self.gl.clear_buffer_f32_slice(
                        glow::COLOR,
                        index as u32,
                        &[clear_value.x, clear_value.y, clear_value.z, clear_value.w],
                    );
                }
            }
            if let Some(Some(clear_depth)) = descriptor
                .depth_attachment
                .as_ref()
                .map(|attachment| attachment.clear_value)
            {
                self.gl.depth_mask(true);
                self.gl
                    .clear_buffer_f32_slice(glow::DEPTH, 0, &[clear_depth as f32]);
            }
            self.gl.viewport(0, 0, width, height);
            let draw_buffers: Vec<u32> = colors
                .iter()
                .enumerate()
                .map(|(index, color)| {
                    if color.is_some() {
                        glow::COLOR_ATTACHMENT0 + index as u32
                    } else {
                        glow::NONE
                    }
                })
                .collect();
            self.gl.draw_buffers(&draw_buffers);
        }
        Box::new(GlRenderPass {
            color_attachment_count: colors.len(),
        })
    }

    fn clear_color_texture(&self, color_texture: &dyn GpuTexture, clear_color: &Vector4f) {
        // SAFETY: the device's context is current.
        unsafe {
            self.gl
                .clear_color(clear_color.x, clear_color.y, clear_color.z, clear_color.w);
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl.color_mask(true, true, true, true);
            for mip_level in 0..color_texture.get_mip_levels() {
                self.bind_frame_buffer_textures(
                    self.draw_fbo,
                    Some(gl_texture(color_texture).id),
                    None,
                    mip_level,
                    glow::FRAMEBUFFER,
                );
                self.gl.clear(glow::COLOR_BUFFER_BIT);
            }
            self.gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                None,
                0,
            );
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }

    fn clear_color_and_depth_textures(
        &self,
        color_texture: &dyn GpuTexture,
        clear_color: &Vector4f,
        depth_texture: &dyn GpuTexture,
        clear_depth: f64,
    ) {
        // SAFETY: the device's context is current.
        unsafe {
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl.clear_depth_f64(clear_depth);
            self.gl
                .clear_color(clear_color.x, clear_color.y, clear_color.z, clear_color.w);
            self.gl.depth_mask(true);
            self.gl.color_mask(true, true, true, true);
            self.bind_frame_buffer_textures(
                self.draw_fbo,
                Some(gl_texture(color_texture).id),
                Some(gl_texture(depth_texture).id),
                0,
                glow::FRAMEBUFFER,
            );
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            self.gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                None,
                0,
            );
            self.gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::DEPTH_ATTACHMENT,
                glow::TEXTURE_2D,
                None,
                0,
            );
            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }

    /// Panics where Java throws `IllegalStateException` on a GL error.
    fn copy_texture_to_buffer(
        &self,
        source: &dyn GpuTexture,
        destination: &Rc<dyn GpuBuffer>,
        offset: i64,
        callback: Box<dyn FnOnce()>,
        mip_level: i32,
    ) {
        let width = source.get_width(mip_level);
        let height = source.get_height(mip_level);
        let is_depth = source.get_format().has_depth_aspect();
        let texture_id = gl_texture(source).id;
        let buffer = destination
            .as_any()
            .downcast_ref::<GlBuffer>()
            .expect("an OpenGL buffer");
        // SAFETY: the device's context is current, and the texture and buffer are its.
        unsafe {
            while self.gl.get_error() != glow::NO_ERROR {}
            self.bind_frame_buffer_textures(
                self.read_fbo,
                (!is_depth).then_some(texture_id),
                is_depth.then_some(texture_id),
                mip_level,
                glow::READ_FRAMEBUFFER,
            );
            self.gl
                .bind_buffer(glow::PIXEL_PACK_BUFFER, Some(buffer.handle));
            self.gl
                .pixel_store_i32(glow::PACK_ALIGNMENT, source.get_format().byte_alignment());
            self.gl.pixel_store_i32(glow::PACK_ROW_LENGTH, width);
            if is_depth {
                self.gl.read_buffer(glow::NONE);
            }
            self.gl.read_pixels(
                0,
                0,
                width,
                height,
                GlConst::to_gl_external_id(source.get_format()),
                GlConst::to_gl_type(source.get_format()),
                glow::PixelPackData::BufferOffset(offset as u32),
            );
            self.fenced_tasks.borrow_mut().push(callback);
            self.gl.framebuffer_texture_2d(
                glow::READ_FRAMEBUFFER,
                if is_depth {
                    glow::DEPTH_ATTACHMENT
                } else {
                    glow::COLOR_ATTACHMENT0
                },
                glow::TEXTURE_2D,
                None,
                mip_level,
            );
            self.gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
            self.gl.bind_buffer(glow::PIXEL_PACK_BUFFER, None);
            let error = self.gl.get_error();
            assert!(
                error == glow::NO_ERROR,
                "Couldn't perform copyTobuffer for texture {}: GL error {error}",
                source.get_label()
            );
        }
    }
}
