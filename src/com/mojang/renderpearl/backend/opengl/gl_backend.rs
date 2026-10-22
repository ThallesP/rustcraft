//! com/mojang/renderpearl/backend/opengl/GlBackend.java
//!
//! LWJGL loads `libGL.so.1` as its OpenGL library on Linux, which `loadLibrary` hands SDL too.
//! Java then checks SDL and LWJGL agree on `glGetError`'s address; the port has no other loader
//! than SDL's, so there's nothing to compare.

use crate::com::mojang::renderpearl::api::device::backend_creation_exception::{
    BackendCreationException, Reason,
};
use crate::com::mojang::renderpearl::api::device::gpu_backend::GpuBackend;
use crate::com::mojang::renderpearl::api::device::gpu_debug_options::GpuDebugOptions;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::backend::opengl::gl_device::GlDevice;
use crate::org::lwjgl::sdl::sdl_error::SDLError;
use crate::org::lwjgl::sdl::sdl_video::{SDLVideo, SdlWindow};

#[derive(Default)]
pub struct GlBackend {
    library_loaded: bool,
    library_load_failure: Option<BackendCreationException>,
}

const VERSION_MAJOR: i32 = 3;
const VERSION_MINOR: i32 = 3;
const LIBRARY: &str = "libGL.so.1";

impl GlBackend {
    pub fn new() -> GlBackend {
        GlBackend::default()
    }

    /// `createWindow`: OpenGL 3.3, core and forward compatible, with an sRGB-capable framebuffer.
    pub fn create_window_with(
        &self,
        title: Option<&str>,
        width: i32,
        height: i32,
        flags: u64,
    ) -> SdlWindow {
        SDLVideo::SDL_GL_SetAttribute(SDLVideo::SDL_GL_CONTEXT_MAJOR_VERSION, VERSION_MAJOR);
        SDLVideo::SDL_GL_SetAttribute(SDLVideo::SDL_GL_CONTEXT_MINOR_VERSION, VERSION_MINOR);
        SDLVideo::SDL_GL_SetAttribute(
            SDLVideo::SDL_GL_CONTEXT_PROFILE_MASK,
            SDLVideo::SDL_GL_CONTEXT_PROFILE_CORE,
        );
        SDLVideo::SDL_GL_SetAttribute(
            SDLVideo::SDL_GL_CONTEXT_FLAGS,
            SDLVideo::SDL_GL_CONTEXT_FORWARD_COMPATIBLE_FLAG,
        );
        SDLVideo::SDL_GL_SetAttribute(SDLVideo::SDL_GL_FRAMEBUFFER_SRGB_CAPABLE, 1);
        SDLVideo::SDL_CreateWindow(title, width, height, SDLVideo::SDL_WINDOW_OPENGL | flags)
    }
}

impl GpuBackend for GlBackend {
    fn get_name(&self) -> &'static str {
        "OpenGL"
    }

    fn load_library(&mut self) -> Result<(), BackendCreationException> {
        if self.library_loaded {
            return Ok(());
        }
        if let Some(failure) = &self.library_load_failure {
            return Err(failure.clone());
        }
        if !SDLVideo::SDL_GL_LoadLibrary(Some(LIBRARY)) {
            let failure = BackendCreationException::new(
                format!(
                    "OpenGL is not supported: {}",
                    SDLError::SDL_GetError().unwrap_or_else(|| "<no error>".to_owned())
                ),
                Reason::OPENGL_MISSING,
            );
            self.library_load_failure = Some(failure.clone());
            return Err(failure);
        }
        self.library_loaded = true;
        Ok(())
    }

    fn unload_library(&mut self) {
        if self.library_loaded {
            SDLVideo::SDL_GL_UnloadLibrary();
            self.library_loaded = false;
        }
    }

    fn create_window(&self, title: Option<&str>, width: i32, height: i32, flags: u64) -> SdlWindow {
        self.create_window_with(title, width, height, flags)
    }

    fn create_device(
        &self,
        debug_options: &GpuDebugOptions,
    ) -> Result<Box<dyn GpuDevice>, BackendCreationException> {
        Ok(Box::new(GlDevice::new(self, debug_options)?))
    }
}
