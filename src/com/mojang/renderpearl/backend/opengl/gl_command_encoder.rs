//! com/mojang/renderpearl/backend/opengl/GlCommandEncoder.java
//!
//! Render passes and their draws, clearing textures, writing buffers and textures, copying a
//! texture into a buffer and presenting. GL runs commands as they're issued, so `submit` only has
//! to wait for the GPU before running the copies' callbacks, which Java queues as fenced tasks
//! for `RenderSystem` to run once their fence signals.
//!
//! The encoder is also the API's `CommandEncoder` until the frontend's `FrontendCommandEncoder`
//! is ported, so it makes each render pass a `FrontendRenderPass` around its own, and checks what
//! Java's frontend checks before a pass or a write (the render area, attachments' usage, sizes).
//!
//! Java's `GlStateManager` skips calls that wouldn't change GL's state; the port calls GL
//! directly, which leaves the same state. `FrameBufferCache` is a list of framebuffers by their
//! attachments here, and `DirectStateAccess`'s binds use the classic entry points, which do the
//! same.

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::com::mojang::blaze3d::systems::scissor_state::ScissorState;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::api::commands::command_encoder::CommandEncoder;
use crate::com::mojang::renderpearl::api::commands::render_pass::RenderPass;
use crate::com::mojang::renderpearl::api::commands::render_pass_descriptor::RenderPassDescriptor;
use crate::com::mojang::renderpearl::api::device::device_features::DeviceFeatures;
use crate::com::mojang::renderpearl::api::device::device_limits::DeviceLimits;
use crate::com::mojang::renderpearl::api::gpu_format::ComponentType;
use crate::com::mojang::renderpearl::api::pipeline::index_type::IndexType;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::{self, GpuTexture};
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::com::mojang::renderpearl::backend::api::render_pass_backend::UniformValue;
use crate::com::mojang::renderpearl::backend::opengl::gl_buffer::GlBuffer;
use crate::com::mojang::renderpearl::backend::opengl::gl_const::GlConst;
use crate::com::mojang::renderpearl::backend::opengl::gl_device::GlDevice;
use crate::com::mojang::renderpearl::backend::opengl::gl_render_pass::GlRenderPass;
use crate::com::mojang::renderpearl::backend::opengl::gl_render_pipeline::GlRenderPipeline;
use crate::com::mojang::renderpearl::backend::opengl::gl_sampler::GlSampler;
use crate::com::mojang::renderpearl::backend::opengl::gl_texture::GlTexture;
use crate::com::mojang::renderpearl::backend::opengl::uniform::Uniform;
use crate::com::mojang::renderpearl::frontend::frontend_render_pass::FrontendRenderPass;
use crate::org::joml::vector4f::Vector4f;
use crate::org::lwjgl::sdl::sdl_video::SdlWindow;
use glow::HasContext;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// A framebuffer's attachments: each color target's texture and mip level, and the depth's.
type FrameBufferKey = (
    Vec<Option<(glow::NativeTexture, i32)>>,
    Option<(glow::NativeTexture, i32)>,
);

pub struct GlCommandEncoder {
    gl: Rc<glow::Context>,
    features: DeviceFeatures,
    limits: DeviceLimits,
    read_fbo: glow::NativeFramebuffer,
    draw_fbo: glow::NativeFramebuffer,
    frame_buffers: RefCell<Vec<(FrameBufferKey, glow::NativeFramebuffer)>>,
    fenced_tasks: RefCell<Vec<Box<dyn FnOnce()>>>,
    last_pipeline: RefCell<Option<Rc<GlRenderPipeline>>>,
    /// The frontend's `isInRenderPass`.
    is_in_render_pass: Cell<bool>,
}

fn gl_texture(texture: &dyn GpuTexture) -> &GlTexture {
    texture
        .as_any()
        .downcast_ref::<GlTexture>()
        .expect("an OpenGL texture")
}

fn gl_buffer(buffer: &dyn GpuBuffer) -> &GlBuffer {
    buffer
        .as_any()
        .downcast_ref::<GlBuffer>()
        .expect("an OpenGL buffer")
}

impl GlCommandEncoder {
    pub(super) fn new(
        gl: Rc<glow::Context>,
        features: DeviceFeatures,
        limits: DeviceLimits,
    ) -> GlCommandEncoder {
        // SAFETY: the device's context is current.
        let (read_fbo, draw_fbo) = unsafe {
            (
                gl.create_framebuffer().expect("glGenFramebuffers"),
                gl.create_framebuffer().expect("glGenFramebuffers"),
            )
        };
        GlCommandEncoder {
            gl,
            features,
            limits,
            read_fbo,
            draw_fbo,
            frame_buffers: RefCell::default(),
            fenced_tasks: RefCell::default(),
            last_pipeline: RefCell::default(),
            is_in_render_pass: Cell::new(false),
        }
    }

    /// Panics where Java's frontend throws `IllegalStateException` for a command inside a pass.
    fn check_not_in_render_pass(&self) {
        assert!(
            !self.is_in_render_pass.get(),
            "Close the existing render pass before performing additional commands"
        );
    }

    /// The frontend's `submitRenderPass`, then the backend's: the pass is over, and the default
    /// framebuffer bound again.
    fn submit_render_pass(&self) {
        assert!(
            self.is_in_render_pass.get(),
            "Can't submit a render pass if one isn't open"
        );
        self.is_in_render_pass.set(false);
        // SAFETY: the device's context is current.
        unsafe { self.gl.bind_framebuffer(glow::FRAMEBUFFER, None) };
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn execute_draw(
        &self,
        render_pass: &mut GlRenderPass<'_>,
        base_vertex: i32,
        first_index: i32,
        draw_count: i32,
        index_type: Option<IndexType>,
        instance_count: i32,
        first_instance: i32,
    ) {
        self.setup_draw(render_pass);
        let topology = render_pass
            .pipeline
            .as_ref()
            .expect("a pipeline")
            .primitive_topology();
        // SAFETY: the device's context is current, and setupDraw bound the pass's buffers.
        unsafe {
            match index_type {
                Some(index_type) => {
                    let offset = first_index * index_type.bytes();
                    if first_instance > 0 {
                        self.gl.draw_elements_instanced_base_vertex_base_instance(
                            topology,
                            draw_count,
                            GlConst::to_gl_index_type(index_type),
                            offset,
                            instance_count,
                            base_vertex,
                            first_instance as u32,
                        );
                    } else {
                        self.gl.draw_elements_instanced_base_vertex(
                            topology,
                            draw_count,
                            GlConst::to_gl_index_type(index_type),
                            offset,
                            instance_count,
                            base_vertex,
                        );
                    }
                }
                None if first_instance > 0 => self.gl.draw_arrays_instanced_base_instance(
                    topology,
                    base_vertex,
                    draw_count,
                    instance_count,
                    first_instance as u32,
                ),
                None => {
                    self.gl
                        .draw_arrays_instanced(topology, base_vertex, draw_count, instance_count)
                }
            }
        }
    }

    /// Binds what changed since the pass's last draw: vertex and index buffers, the pipeline's
    /// state (unless the last pass drew with it), uniforms and the scissor.
    fn setup_draw(&self, render_pass: &mut GlRenderPass<'_>) {
        let pipeline = render_pass.pipeline.clone().expect("a pipeline");
        let gl = &self.gl;
        // SAFETY: the device's context is current, and the buffers and textures are its.
        unsafe {
            if render_pass.vertex_buffer_dirty {
                pipeline.vertex_array().bind(&render_pass.vertex_buffers);
                render_pass.vertex_buffer_dirty = false;
            }

            if render_pass.index_buffer_dirty {
                render_pass.index_buffer_dirty = false;
                let index_buffer = render_pass.index_buffer.as_ref().expect("an index buffer");
                gl.bind_buffer(
                    glow::ELEMENT_ARRAY_BUFFER,
                    Some(gl_buffer(&**index_buffer).handle),
                );
            }

            let same_pipeline = self
                .last_pipeline
                .borrow()
                .as_ref()
                .is_some_and(|last| Rc::ptr_eq(last, &pipeline));
            if !same_pipeline {
                pipeline.bind();
                *self.last_pipeline.borrow_mut() = Some(pipeline.clone());
            }

            let gl_program = pipeline.program();
            if render_pass.any_uniform_dirty {
                render_pass.any_uniform_dirty = false;
                for i in 0..render_pass.dirty_uniforms.len() {
                    if !render_pass.dirty_uniforms[i] {
                        continue;
                    }
                    render_pass.dirty_uniforms[i] = false;
                    let Some(dirty_uniform) = gl_program.get_uniform(i) else {
                        continue;
                    };
                    let value = render_pass.uniforms[i].as_ref();
                    match dirty_uniform {
                        Uniform::Ubo { block_binding } => {
                            let Some(UniformValue::Buffer(slice)) = value else {
                                panic!("a uniform buffer's value is a GpuBufferSlice");
                            };
                            gl.bind_buffer_range(
                                glow::UNIFORM_BUFFER,
                                block_binding as u32,
                                Some(gl_buffer(&*slice.buffer).handle),
                                slice.offset as i32,
                                slice.length as i32,
                            );
                        }
                        Uniform::Utb {
                            sampler_index,
                            format,
                            texture,
                        } => {
                            let Some(UniformValue::Buffer(slice)) = value else {
                                panic!("a texel buffer's value is a GpuBufferSlice");
                            };
                            gl.active_texture(glow::TEXTURE0 + sampler_index as u32);
                            gl.bind_texture(glow::TEXTURE_BUFFER, Some(texture));
                            gl.tex_buffer(
                                glow::TEXTURE_BUFFER,
                                GlConst::to_gl_internal_id(format),
                                Some(gl_buffer(&*slice.buffer).handle),
                            );
                        }
                        Uniform::Sampler { sampler_index } => {
                            let Some(UniformValue::Sampler(view_and_sampler)) = value else {
                                continue;
                            };
                            let texture_view = &view_and_sampler.view;
                            gl.active_texture(glow::TEXTURE0 + sampler_index as u32);
                            let texture = texture_view.texture();
                            let target =
                                if texture.get_usage() & gpu_texture::USAGE_CUBEMAP_COMPATIBLE != 0
                                {
                                    glow::TEXTURE_CUBE_MAP
                                } else {
                                    glow::TEXTURE_2D
                                };
                            gl.bind_texture(target, Some(gl_texture(&**texture).id));
                            let sampler = view_and_sampler
                                .sampler
                                .as_any()
                                .downcast_ref::<GlSampler>()
                                .expect("an OpenGL sampler");
                            gl.bind_sampler(sampler_index as u32, Some(sampler.get_id()));
                            gl.tex_parameter_i32(
                                target,
                                glow::TEXTURE_BASE_LEVEL,
                                texture_view.base_mip_level(),
                            );
                            gl.tex_parameter_i32(
                                target,
                                glow::TEXTURE_MAX_LEVEL,
                                texture_view.base_mip_level() + texture_view.mip_levels() - 1,
                            );
                        }
                    }
                }
            }

            if render_pass.scissor_state_dirty {
                render_pass.scissor_state_dirty = false;
                let scissor = render_pass.scissor_state;
                if scissor.enabled() {
                    gl.enable(glow::SCISSOR_TEST);
                    gl.scissor(scissor.x(), scissor.y(), scissor.width(), scissor.height());
                } else {
                    gl.disable(glow::SCISSOR_TEST);
                }
            }
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

    /// Panics where Java's frontend throws: inside another pass, with no attachment, a render area
    /// outside the attachments, or an attachment that isn't a render attachment of their size.
    fn create_render_pass(&self, descriptor: RenderPassDescriptor) -> Box<dyn RenderPass + '_> {
        assert!(
            !self.is_in_render_pass.get(),
            "Close the existing render pass before creating a new one!"
        );
        let color_attachment_count = descriptor.color_attachments.len();
        assert!(
            color_attachment_count <= self.limits.max_color_attachments as usize,
            "Render pass created with {color_attachment_count} color attachments but device only supports {}",
            self.limits.max_color_attachments
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
        self.is_in_render_pass.set(true);

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
        *self.last_pipeline.borrow_mut() = None;
        let mut scissor_state = ScissorState::new();
        scissor_state.enable(
            render_area.x,
            render_area.y,
            render_area.width,
            render_area.height,
        );
        let backend = GlRenderPass::new(self, colors.len(), scissor_state);
        let formats = colors
            .iter()
            .map(|color| color.map(|view| view.texture().get_format()))
            .collect();
        Box::new(FrontendRenderPass::new(
            Box::new(backend),
            self.features,
            self.limits,
            formats,
            Box::new(|| self.submit_render_pass()),
            render_area,
        ))
    }

    /// Panics where Java's frontend throws: a buffer that can't be copied to, or data past the
    /// slice or the buffer.
    fn write_to_buffer(&self, slice: &GpuBufferSlice, data: &[u8]) {
        self.check_not_in_render_pass();
        let buffer = &slice.buffer;
        assert!(
            buffer.usage() & gpu_buffer::USAGE_COPY_DST != 0,
            "Buffer needs USAGE_COPY_DST to be a destination for a copy"
        );
        let length = data.len() as i64;
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
        gl_buffer(&**buffer).buffer_sub_data(slice.offset, data);
    }

    /// Panics where Java's frontend throws: a texture that isn't 8-bit with the image's channels,
    /// a mip level or layer it doesn't have, or a rectangle that doesn't fit.
    fn write_to_texture_image(
        &self,
        destination: &dyn GpuTexture,
        source: &NativeImage,
        mip_level: i32,
        depth_or_layer: i32,
        dest_x: i32,
        dest_y: i32,
    ) {
        self.check_not_in_render_pass();
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
        self.write_to_texture(
            destination,
            &source.get_pixel_bytes(),
            mip_level,
            depth_or_layer,
            dest_x,
            dest_y,
            source.get_width(),
            source.get_height(),
        );
    }

    /// Panics where Java's frontend throws: past the source's bytes or the texture's size.
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
    ) {
        self.check_not_in_render_pass();
        let format = destination.get_format();
        assert!(
            mip_level >= 0 && mip_level < destination.get_mip_levels(),
            "Invalid mipLevel {mip_level}, must be >= 0 and < {}",
            destination.get_mip_levels()
        );
        assert!(
            (width * height * format.block_size()) as usize <= source.len(),
            "Copy would overrun the source buffer (remaining length of {}, but copy is {width}x{height} of format {format:?})",
            source.len()
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
        assert!(
            destination.get_usage() & gpu_texture::USAGE_CUBEMAP_COMPATIBLE == 0,
            "cubemaps aren't ported yet"
        );
        // SAFETY: the device's context is current, and the texture is its.
        unsafe {
            self.gl
                .bind_texture(glow::TEXTURE_2D, Some(gl_texture(destination).id));
            self.gl.pixel_store_i32(glow::UNPACK_ROW_LENGTH, width);
            self.gl.pixel_store_i32(glow::UNPACK_SKIP_PIXELS, 0);
            self.gl.pixel_store_i32(glow::UNPACK_SKIP_ROWS, 0);
            self.gl
                .pixel_store_i32(glow::UNPACK_ALIGNMENT, format.byte_alignment());
            self.gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                mip_level,
                dest_x,
                dest_y,
                width,
                height,
                GlConst::to_gl_external_id(format),
                GlConst::to_gl_type(format),
                glow::PixelUnpackData::Slice(Some(source)),
            );
        }
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
