//! com/mojang/renderpearl/backend/opengl/GlSurface.java
//!
//! Only blitting a texture into the window and swapping so far; `configure` (the swap interval)
//! and `acquireNextTexture` come with the window loop.

use crate::com::mojang::renderpearl::api::device::gpu_surface::GpuSurface;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::com::mojang::renderpearl::backend::opengl::gl_device::GlDevice;
use crate::org::lwjgl::sdl::sdl_video::{SDLVideo, SdlWindow};

pub struct GlSurface<'a> {
    device: &'a GlDevice,
    window_handle: SdlWindow,
}

impl<'a> GlSurface<'a> {
    pub fn new(device: &'a GlDevice, window_handle: SdlWindow) -> GlSurface<'a> {
        GlSurface {
            device,
            window_handle,
        }
    }
}

impl GpuSurface for GlSurface<'_> {
    fn blit_from_texture(&self, texture_view: &GpuTextureView) {
        let (width, height) =
            SDLVideo::SDL_GetWindowSizeInPixels(self.window_handle).unwrap_or((0, 0));
        self.device.encoder().present_texture(
            self.device,
            self.window_handle,
            texture_view,
            width,
            height,
        );
    }

    fn present(&self) {
        SDLVideo::SDL_GL_SwapWindow(self.window_handle);
    }
}
