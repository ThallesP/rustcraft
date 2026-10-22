//! com/mojang/renderpearl/backend/opengl/GlDevice.java
//!
//! Makes the OpenGL context the way vanilla does: on a hidden utility window, checking it's at
//! least 3.3, then making and destroying a second window. It reads the driver's extensions like
//! LWJGL's `GLCapabilities` (an extension the driver lists), and turns on the ones vanilla uses in
//! the same order, which makes its `DeviceInfo`: those flags decide the shaders' text (see
//! `GlslCompiler` and `GlPipelineRecompiler`).
//!
//! Functions come from `SDL_GL_GetProcAddress`. Not ported yet: samplers, pipelines (compiling
//! the recompiler's GLSL), queries, fences, and the vertex array, buffer storage, direct state
//! access and debug label helpers, whose extensions are still turned on as vanilla turns them on.

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::GpuBuffer;
use crate::com::mojang::renderpearl::api::commands::command_encoder::CommandEncoder;
use crate::com::mojang::renderpearl::api::device::backend_creation_exception::{
    BackendCreationException, Reason,
};
use crate::com::mojang::renderpearl::api::device::device_info::DeviceInfo;
use crate::com::mojang::renderpearl::api::device::gpu_debug_options::GpuDebugOptions;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::device::gpu_surface::GpuSurface;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::{self, GpuTexture};
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::com::mojang::renderpearl::backend::opengl::gl_backend::GlBackend;
use crate::com::mojang::renderpearl::backend::opengl::gl_buffer::GlBuffer;
use crate::com::mojang::renderpearl::backend::opengl::gl_command_encoder::GlCommandEncoder;
use crate::com::mojang::renderpearl::backend::opengl::gl_const::GlConst;
use crate::com::mojang::renderpearl::backend::opengl::gl_debug::GlDebug;
use crate::com::mojang::renderpearl::backend::opengl::gl_heuristics::GlHeuristics;
use crate::com::mojang::renderpearl::backend::opengl::gl_pipeline_recompiler::GlPipelineRecompiler;
use crate::com::mojang::renderpearl::backend::opengl::gl_surface::GlSurface;
use crate::com::mojang::renderpearl::backend::opengl::gl_texture::GlTexture;
use crate::java::util::hash_set::HashSet;
use crate::org::lwjgl::sdl::sdl_error::SDLError;
use crate::org::lwjgl::sdl::sdl_video::{SDLVideo, SdlGlContext, SdlWindow};
use glow::HasContext;
use std::cell::Cell;
use std::ffi::{CString, c_void};
use std::rc::Rc;

pub struct GlDevice {
    gl: Rc<glow::Context>,
    initial_window_handle: SdlWindow,
    gl_context: SdlGlContext,
    heuristics: GlHeuristics,
    encoder: GlCommandEncoder,
    device_info: DeviceInfo,
    recompiler: GlPipelineRecompiler,
    current_window: Cell<SdlWindow>,
}

/// The device's flags for hidden windows: hidden, a utility window, not focusable.
const HIDDEN_WINDOW_FLAGS: u64 = 2_147_614_728;

impl GlDevice {
    pub const USE_GL_ARB_VERTEX_ATTRIB_BINDING: bool = true;
    pub const USE_GL_KHR_DEBUG: bool = true;
    pub const USE_GL_EXT_DEBUG_LABEL: bool = true;
    pub const USE_GL_ARB_DEBUG_OUTPUT: bool = true;
    pub const USE_GL_ARB_DIRECT_STATE_ACCESS: bool = true;
    pub const USE_GL_ARB_BUFFER_STORAGE: bool = true;
    pub const USE_GL_ARB_BASE_INSTANCE: bool = true;
    pub const USE_GL_ARB_DRAW_INDIRECT: bool = true;
    pub const USE_GL_ARB_MULTI_DRAW_INDIRECT: bool = true;
    pub const USE_GL_ARB_SHADER_DRAW_PARAMETERS: bool = true;

    fn sdl_error() -> String {
        SDLError::SDL_GetError().unwrap_or_else(|| "<no error>".to_owned())
    }

    pub fn new(
        backend: &GlBackend,
        debug_options: &GpuDebugOptions,
    ) -> Result<GlDevice, BackendCreationException> {
        let initial_window_handle = backend.create_window_with(
            Some("Minecraft - RenderPearl OpenGL Hidden Utility Window"),
            320,
            480,
            HIDDEN_WINDOW_FLAGS,
        );
        if initial_window_handle.is_null() {
            return Err(BackendCreationException::new(
                format!(
                    "Failed to create window for OpenGL context: {}",
                    GlDevice::sdl_error()
                ),
                Reason::OPENGL_MISSING,
            ));
        }
        let gl_context = SDLVideo::SDL_GL_CreateContext(initial_window_handle);
        if gl_context.is_null() {
            SDLVideo::SDL_DestroyWindow(initial_window_handle);
            return Err(BackendCreationException::new(
                format!("Failed to create OpenGL context: {}", GlDevice::sdl_error()),
                Reason::OPENGL_MISSING,
            ));
        }
        SDLVideo::SDL_GL_MakeCurrent(initial_window_handle, gl_context);

        let device =
            GlDevice::with_context(backend, debug_options, initial_window_handle, gl_context);
        if device.is_err() {
            SDLVideo::SDL_GL_DestroyContext(gl_context);
            SDLVideo::SDL_DestroyWindow(initial_window_handle);
        }
        device
    }

    fn with_context(
        backend: &GlBackend,
        debug_options: &GpuDebugOptions,
        initial_window_handle: SdlWindow,
        gl_context: SdlGlContext,
    ) -> Result<GlDevice, BackendCreationException> {
        let major_version =
            SDLVideo::SDL_GL_GetAttribute(SDLVideo::SDL_GL_CONTEXT_MAJOR_VERSION).unwrap_or(0);
        let minor_version =
            SDLVideo::SDL_GL_GetAttribute(SDLVideo::SDL_GL_CONTEXT_MINOR_VERSION).unwrap_or(0);
        if major_version < 3 || major_version == 3 && minor_version < 3 {
            return Err(BackendCreationException::new(
                format!(
                    "Failed to create OpenGL 3.3 context, got OpenGL {major_version}.{minor_version}"
                ),
                Reason::OPENGL_MISSING,
            ));
        }

        let second_window = backend.create_window_with(
            Some("Minecraft - RenderPearl OpenGL Hidden Test Window"),
            320,
            480,
            HIDDEN_WINDOW_FLAGS,
        );
        if second_window.is_null() {
            return Err(BackendCreationException::new(
                format!(
                    "Failed to create window for OpenGL after creating context: {}",
                    GlDevice::sdl_error()
                ),
                Reason::OPENGL_MISSING,
            ));
        }
        SDLVideo::SDL_DestroyWindow(second_window);

        // SAFETY: SDL hands out the current context's functions.
        let mut gl =
            unsafe { glow::Context::from_loader_function_cstr(SDLVideo::SDL_GL_GetProcAddress) };
        let capabilities = gl.supported_extensions().clone();
        let has = |extension: &str| capabilities.contains(extension);
        // A Java `HashSet`, whose order `DeviceInfo` lists them in.
        let mut enabled_extensions: HashSet<String> = HashSet::new();
        let mut enable = |extension: &str| {
            enabled_extensions.add(extension.to_owned());
        };

        let max_supported_anisotropy = if has("GL_EXT_texture_filter_anisotropic") {
            enable("GL_EXT_texture_filter_anisotropic");
            // SAFETY: the context is current.
            unsafe { gl.get_parameter_f32(glow::MAX_TEXTURE_MAX_ANISOTROPY_EXT) }.floor() as i32
        } else {
            1
        };

        // SAFETY: the context is current.
        let (renderer, vendor) = unsafe {
            (
                gl.get_parameter_string(glow::RENDERER),
                gl.get_parameter_string(glow::VENDOR),
            )
        };
        let heuristics = GlHeuristics::new(&renderer, &vendor);
        if let Some(extension) = GlDebug::enable_debug_callback(
            &mut gl,
            &capabilities,
            debug_options.log_level,
            debug_options.synchronous_logs,
        ) {
            enable(extension);
        }
        // GlDebugLabel.create.
        if debug_options.use_labels {
            if has("GL_KHR_debug") && GlDevice::USE_GL_KHR_DEBUG {
                enable("GL_KHR_debug");
            } else if has("GL_EXT_debug_label") && GlDevice::USE_GL_EXT_DEBUG_LABEL {
                enable("GL_EXT_debug_label");
            }
        }
        // BufferStorage.create.
        let force_mutable = heuristics.could_be_intel_gen7() || heuristics.is_nvidia();
        if !force_mutable && has("GL_ARB_buffer_storage") && GlDevice::USE_GL_ARB_BUFFER_STORAGE {
            enable("GL_ARB_buffer_storage");
        }
        // DirectStateAccess.create.
        if has("GL_ARB_direct_state_access")
            && GlDevice::USE_GL_ARB_DIRECT_STATE_ACCESS
            && !heuristics.is_gl_on_dx12()
        {
            enable("GL_ARB_direct_state_access");
        }
        // VertexArray.createSource.
        if has("GL_ARB_vertex_attrib_binding") && GlDevice::USE_GL_ARB_VERTEX_ATTRIB_BINDING {
            enable("GL_ARB_vertex_attrib_binding");
        }

        // SAFETY: the context is current.
        unsafe {
            gl.enable(glow::TEXTURE_CUBE_MAP_SEAMLESS);
            gl.enable(glow::PROGRAM_POINT_SIZE);
        }
        let has_clip_control = has("GL_ARB_clip_control");
        if has_clip_control {
            GlDevice::clip_control(glow::LOWER_LEFT, glow::ZERO_TO_ONE);
            enable("GL_ARB_clip_control");
        }
        if has("GL_ARB_shader_draw_parameters") && GlDevice::USE_GL_ARB_SHADER_DRAW_PARAMETERS {
            enable("GL_ARB_shader_draw_parameters");
        }
        if has("GL_ARB_draw_indirect") && GlDevice::USE_GL_ARB_DRAW_INDIRECT {
            enable("GL_ARB_draw_indirect");
            if has("GL_ARB_multi_draw_indirect") && GlDevice::USE_GL_ARB_MULTI_DRAW_INDIRECT {
                enable("GL_ARB_multi_draw_indirect");
            }
        }
        if has("GL_ARB_base_instance") && GlDevice::USE_GL_ARB_BASE_INSTANCE {
            enable("GL_ARB_base_instance");
        }

        let enabled_extensions: Vec<String> = enabled_extensions.iter().cloned().collect();
        let device_info = heuristics.create_device_info(
            &gl,
            has_clip_control,
            max_supported_anisotropy,
            &enabled_extensions,
        );
        let gl = Rc::new(gl);
        Ok(GlDevice {
            encoder: GlCommandEncoder::new(gl.clone()),
            recompiler: GlPipelineRecompiler::new(device_info.features.shader_draw_parameters),
            gl,
            initial_window_handle,
            gl_context,
            heuristics,
            device_info,
            current_window: Cell::new(initial_window_handle),
        })
    }

    /// `ARBClipControl.glClipControl`, which glow doesn't bind.
    fn clip_control(origin: u32, depth: u32) {
        let name = CString::new("glClipControl").expect("no NUL");
        let function = SDLVideo::SDL_GL_GetProcAddress(&name);
        assert!(
            !function.is_null(),
            "the driver lists GL_ARB_clip_control without glClipControl"
        );
        // SAFETY: glClipControl's signature, from a context that has it.
        unsafe {
            let function: unsafe extern "C" fn(u32, u32) =
                std::mem::transmute::<*const c_void, _>(function);
            function(origin, depth);
        }
    }

    pub fn heuristics(&self) -> &GlHeuristics {
        &self.heuristics
    }

    pub fn recompiler(&self) -> &GlPipelineRecompiler {
        &self.recompiler
    }

    pub fn encoder(&self) -> &GlCommandEncoder {
        &self.encoder
    }

    pub fn gl(&self) -> &glow::Context {
        &self.gl
    }

    pub(super) fn make_current(&self, window_handle: SdlWindow) {
        if window_handle != self.current_window.get() {
            self.current_window.set(window_handle);
            SDLVideo::SDL_GL_MakeCurrent(window_handle, self.gl_context);
        }
    }
}

impl GpuDevice for GlDevice {
    fn create_command_encoder(&self) -> &dyn CommandEncoder {
        &self.encoder
    }

    fn create_surface(&self, window: SdlWindow) -> Box<dyn GpuSurface + '_> {
        Box::new(GlSurface::new(self, window))
    }

    /// Panics where Java throws on a GL error or a format GL textures can't have.
    fn create_texture(
        &self,
        label: Option<&str>,
        usage: i32,
        format: GpuFormat,
        width: i32,
        height: i32,
        depth_or_layers: i32,
        mip_levels: i32,
    ) -> Rc<dyn GpuTexture> {
        let gl = &self.gl;
        // SAFETY: the device's context is current.
        unsafe {
            while gl.get_error() != glow::NO_ERROR {}
            let id = gl.create_texture().expect("glGenTextures");
            let label = label.map_or_else(|| id.0.get().to_string(), str::to_owned);
            let is_cubemap = usage & gpu_texture::USAGE_CUBEMAP_COMPATIBLE != 0;
            let target = if is_cubemap {
                glow::TEXTURE_CUBE_MAP
            } else {
                glow::TEXTURE_2D
            };
            gl.bind_texture(target, Some(id));
            gl.tex_parameter_i32(target, glow::TEXTURE_MAX_LEVEL, mip_levels - 1);
            gl.tex_parameter_i32(target, glow::TEXTURE_MIN_LOD, 0);
            gl.tex_parameter_i32(target, glow::TEXTURE_MAX_LOD, mip_levels - 1);
            if format.has_depth_aspect() {
                gl.tex_parameter_i32(target, glow::TEXTURE_COMPARE_MODE, 0);
            }

            let internal = GlConst::to_gl_internal_id(format);
            let external = GlConst::to_gl_external_id(format);
            let gl_type = GlConst::to_gl_type(format);
            assert!(
                internal != 0 && external != 0 && gl_type != 0,
                "{format:?} format cannot be used to create textures"
            );
            let targets: &[u32] = if is_cubemap {
                &[
                    glow::TEXTURE_CUBE_MAP_POSITIVE_X,
                    glow::TEXTURE_CUBE_MAP_NEGATIVE_X,
                    glow::TEXTURE_CUBE_MAP_POSITIVE_Y,
                    glow::TEXTURE_CUBE_MAP_NEGATIVE_Y,
                    glow::TEXTURE_CUBE_MAP_POSITIVE_Z,
                    glow::TEXTURE_CUBE_MAP_NEGATIVE_Z,
                ]
            } else {
                &[glow::TEXTURE_2D]
            };
            for &image_target in targets {
                for level in 0..mip_levels {
                    gl.tex_image_2d(
                        image_target,
                        level,
                        internal as i32,
                        width >> level,
                        height >> level,
                        0,
                        external,
                        gl_type,
                        glow::PixelUnpackData::Slice(None),
                    );
                }
            }
            let error = gl.get_error();
            assert!(
                error != glow::OUT_OF_MEMORY,
                "Could not allocate texture of {width}x{height} for {label}"
            );
            assert!(error == glow::NO_ERROR, "OpenGL error {error}");
            Rc::new(GlTexture::new(
                self.gl.clone(),
                usage,
                label,
                format,
                width,
                height,
                depth_or_layers,
                mip_levels,
                id,
            ))
        }
    }

    fn create_texture_view_mips(
        &self,
        texture: &Rc<dyn GpuTexture>,
        base_mip_level: i32,
        mip_levels: i32,
    ) -> GpuTextureView {
        GpuTextureView::new(texture.clone(), base_mip_level, mip_levels)
    }

    fn create_buffer(&self, _label: Option<&str>, usage: i32, size: i64) -> Rc<dyn GpuBuffer> {
        Rc::new(GlBuffer::new(self.gl.clone(), usage, size))
    }

    fn get_device_info(&self) -> &DeviceInfo {
        &self.device_info
    }
}

impl Drop for GlDevice {
    fn drop(&mut self) {
        self.make_current(self.initial_window_handle);
        self.encoder.close();
        SDLVideo::SDL_GL_DestroyContext(self.gl_context);
        SDLVideo::SDL_DestroyWindow(self.initial_window_handle);
    }
}
