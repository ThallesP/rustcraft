//! com/mojang/blaze3d/platform/Window.java
//!
//! Making the window: resizable, high pixel density, at least 320 by 240, at the size asked (Java
//! first takes it from the options and its display, at least 320 by 240); its sizes as it's
//! resized (`onResize`, `onFramebufferResize`), the framebuffer's being what the game renders at
//! (`getWidth`, `getHeight`); and the GUI's scale for a framebuffer's size (`calculateScale`,
//! `setGuiScale`), as functions of the size, which an offscreen renderer has without a window.
//! Fullscreen, icons, video modes and the mouse's grab on a resize aren't ported yet.

use crate::com::mojang::renderpearl::api::device::gpu_backend::GpuBackend;
use crate::org::lwjgl::sdl::sdl_error::SDLError;
use crate::org::lwjgl::sdl::sdl_video::{SDLVideo, SdlWindow};

pub struct Window {
    handle: SdlWindow,
    /// The window's size, in screen coordinates.
    windowed_width: i32,
    windowed_height: i32,
    /// The framebuffer's, in pixels.
    framebuffer_width: i32,
    framebuffer_height: i32,
}

impl Window {
    /// Panics where Java throws `WindowInitFailed`'s `IllegalStateException`.
    pub fn new(backend: &dyn GpuBackend, width: i32, height: i32, title: &str) -> Window {
        let width = Window::allowed_window_min_size(width, 320);
        let height = Window::allowed_window_min_size(height, 240);
        let mut window = Window {
            handle: Window::create_window(backend, width, height, title),
            windowed_width: width,
            windowed_height: height,
            framebuffer_width: width,
            framebuffer_height: height,
        };
        window.refresh_framebuffer_size();
        window
    }

    /// `onResize`: the window's new size, at least 320 by 240 while windowed.
    pub fn on_resize(&mut self, new_width: i32, new_height: i32) {
        self.windowed_width = Window::allowed_window_min_size(new_width, 320);
        self.windowed_height = Window::allowed_window_min_size(new_height, 240);
    }

    /// `onFramebufferResize`: the framebuffer's new size, unless it's empty (minimized); true when
    /// it took it, after which Java's `framebufferSizeChanged` runs.
    pub fn on_framebuffer_resize(&mut self, new_width: i32, new_height: i32) -> bool {
        if new_width > 0 && new_height > 0 {
            self.framebuffer_width = new_width;
            self.framebuffer_height = new_height;
            true
        } else {
            false
        }
    }

    /// `refreshFramebufferSize`: the framebuffer's size as SDL has it now.
    fn refresh_framebuffer_size(&mut self) {
        if let Some((width, height)) = SDLVideo::SDL_GetWindowSizeInPixels(self.handle) {
            self.framebuffer_width = width;
            self.framebuffer_height = height;
        }
    }

    fn allowed_window_min_size(size: i32, min_size: i32) -> i32 {
        size.max(min_size)
    }

    fn create_window(backend: &dyn GpuBackend, width: i32, height: i32, title: &str) -> SdlWindow {
        let flags = SDLVideo::SDL_WINDOW_RESIZABLE | SDLVideo::SDL_WINDOW_HIGH_PIXEL_DENSITY;
        let window_handle = backend.create_window(Some(title), width, height, flags);
        assert!(
            !window_handle.is_null(),
            "Failed to create window: {}",
            SDLError::SDL_GetError().unwrap_or_else(|| "<no error>".to_owned())
        );
        SDLVideo::SDL_SetWindowMinimumSize(window_handle, 320, 240);
        window_handle
    }

    /// `calculateScale`: the largest scale up to `max_scale` (0 for as large as fits) that keeps
    /// the GUI at least 320 by 240, made even for the Unicode font.
    pub fn calculate_scale(
        framebuffer_width: i32,
        framebuffer_height: i32,
        max_scale: i32,
        enforce_unicode: bool,
    ) -> i32 {
        let mut scale = 1;
        while scale != max_scale
            && scale < framebuffer_width
            && scale < framebuffer_height
            && framebuffer_width / (scale + 1) >= 320
            && framebuffer_height / (scale + 1) >= 240
        {
            scale += 1;
        }
        if enforce_unicode && scale % 2 != 0 {
            scale += 1;
        }
        scale
    }

    /// `setGuiScale`'s `guiScaledWidth` and `guiScaledHeight`.
    pub fn gui_scaled_size(
        framebuffer_width: i32,
        framebuffer_height: i32,
        gui_scale: i32,
    ) -> (i32, i32) {
        (
            (f64::from(framebuffer_width) / f64::from(gui_scale)).ceil() as i32,
            (f64::from(framebuffer_height) / f64::from(gui_scale)).ceil() as i32,
        )
    }

    pub fn handle(&self) -> SdlWindow {
        self.handle
    }

    /// The framebuffer's width, like Java's.
    pub fn get_width(&self) -> i32 {
        self.framebuffer_width
    }

    pub fn get_height(&self) -> i32 {
        self.framebuffer_height
    }

    pub fn get_screen_width(&self) -> i32 {
        self.windowed_width
    }

    pub fn get_screen_height(&self) -> i32 {
        self.windowed_height
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        SDLVideo::SDL_DestroyWindow(self.handle);
    }
}
