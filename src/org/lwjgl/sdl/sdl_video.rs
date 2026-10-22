//! org/lwjgl/sdl/SDLVideo.java (LWJGL 3.4.3)
//!
//! The window and OpenGL context functions the GL backend calls. Windows and contexts are
//! pointers in C, `long`s in LWJGL, and raw pointers here.

use crate::org::lwjgl::system::shared_library_loader;
use std::ffi::{CStr, CString, c_char, c_int, c_void};

pub type SdlWindow = *mut c_void;
pub type SdlGlContext = *mut c_void;

shared_library_loader::functions! {
    "sdl", "libSDL3.so";
    SDL_GL_LoadLibrary: fn(*const c_char) -> bool;
    SDL_GL_UnloadLibrary: fn();
    SDL_GL_GetProcAddress: fn(*const c_char) -> *const c_void;
    SDL_GL_SetAttribute: fn(c_int, c_int) -> bool;
    SDL_GL_GetAttribute: fn(c_int, *mut c_int) -> bool;
    SDL_GL_CreateContext: fn(SdlWindow) -> SdlGlContext;
    SDL_GL_DestroyContext: fn(SdlGlContext) -> bool;
    SDL_GL_MakeCurrent: fn(SdlWindow, SdlGlContext) -> bool;
    SDL_GL_SwapWindow: fn(SdlWindow) -> bool;
    SDL_CreateWindow: fn(*const c_char, c_int, c_int, u64) -> SdlWindow;
    SDL_DestroyWindow: fn(SdlWindow);
    SDL_SetWindowMinimumSize: fn(SdlWindow, c_int, c_int) -> bool;
    SDL_GetWindowSizeInPixels: fn(SdlWindow, *mut c_int, *mut c_int) -> bool;
}

// SAFETY (all): windows and contexts passed in are ones SDL made and hasn't destroyed, which
// the GL backend keeps to; strings are null-terminated.
pub struct SDLVideo;

#[allow(non_snake_case)]
impl SDLVideo {
    pub const SDL_WINDOW_OPENGL: u64 = 0x2;
    pub const SDL_WINDOW_HIDDEN: u64 = 0x8;
    pub const SDL_WINDOW_RESIZABLE: u64 = 0x20;
    pub const SDL_WINDOW_HIGH_PIXEL_DENSITY: u64 = 0x2000;
    pub const SDL_WINDOW_UTILITY: u64 = 0x20000;
    pub const SDL_WINDOW_NOT_FOCUSABLE: u64 = 0x8000_0000;

    pub const SDL_GL_CONTEXT_MAJOR_VERSION: c_int = 17;
    pub const SDL_GL_CONTEXT_MINOR_VERSION: c_int = 18;
    pub const SDL_GL_CONTEXT_FLAGS: c_int = 19;
    pub const SDL_GL_CONTEXT_PROFILE_MASK: c_int = 20;
    pub const SDL_GL_FRAMEBUFFER_SRGB_CAPABLE: c_int = 22;
    pub const SDL_GL_CONTEXT_PROFILE_CORE: c_int = 1;
    pub const SDL_GL_CONTEXT_FORWARD_COMPATIBLE_FLAG: c_int = 2;

    pub(crate) fn SDL_GL_LoadLibrary(path: Option<&str>) -> bool {
        let path = path.map(|path| CString::new(path).expect("no NUL"));
        unsafe {
            (FUNCTIONS.SDL_GL_LoadLibrary)(
                path.as_ref().map_or(std::ptr::null(), |path| path.as_ptr()),
            )
        }
    }

    pub(crate) fn SDL_GL_UnloadLibrary() {
        unsafe { (FUNCTIONS.SDL_GL_UnloadLibrary)() }
    }

    pub(crate) fn SDL_GL_GetProcAddress(name: &CStr) -> *const c_void {
        unsafe { (FUNCTIONS.SDL_GL_GetProcAddress)(name.as_ptr()) }
    }

    pub(crate) fn SDL_GL_SetAttribute(attribute: c_int, value: c_int) -> bool {
        unsafe { (FUNCTIONS.SDL_GL_SetAttribute)(attribute, value) }
    }

    pub(crate) fn SDL_GL_GetAttribute(attribute: c_int) -> Option<c_int> {
        let mut value = 0;
        unsafe { (FUNCTIONS.SDL_GL_GetAttribute)(attribute, &mut value) }.then_some(value)
    }

    pub(crate) fn SDL_GL_CreateContext(window: SdlWindow) -> SdlGlContext {
        unsafe { (FUNCTIONS.SDL_GL_CreateContext)(window) }
    }

    pub(crate) fn SDL_GL_DestroyContext(context: SdlGlContext) -> bool {
        unsafe { (FUNCTIONS.SDL_GL_DestroyContext)(context) }
    }

    pub(crate) fn SDL_GL_MakeCurrent(window: SdlWindow, context: SdlGlContext) -> bool {
        unsafe { (FUNCTIONS.SDL_GL_MakeCurrent)(window, context) }
    }

    pub(crate) fn SDL_GL_SwapWindow(window: SdlWindow) -> bool {
        unsafe { (FUNCTIONS.SDL_GL_SwapWindow)(window) }
    }

    pub(crate) fn SDL_CreateWindow(
        title: Option<&str>,
        width: c_int,
        height: c_int,
        flags: u64,
    ) -> SdlWindow {
        let title = title.map(|title| CString::new(title).expect("no NUL"));
        unsafe {
            (FUNCTIONS.SDL_CreateWindow)(
                title
                    .as_ref()
                    .map_or(std::ptr::null(), |title| title.as_ptr()),
                width,
                height,
                flags,
            )
        }
    }

    pub(crate) fn SDL_DestroyWindow(window: SdlWindow) {
        unsafe { (FUNCTIONS.SDL_DestroyWindow)(window) }
    }

    pub(crate) fn SDL_SetWindowMinimumSize(
        window: SdlWindow,
        min_width: c_int,
        min_height: c_int,
    ) -> bool {
        unsafe { (FUNCTIONS.SDL_SetWindowMinimumSize)(window, min_width, min_height) }
    }

    pub(crate) fn SDL_GetWindowSizeInPixels(window: SdlWindow) -> Option<(c_int, c_int)> {
        let (mut width, mut height) = (0, 0);
        unsafe { (FUNCTIONS.SDL_GetWindowSizeInPixels)(window, &mut width, &mut height) }
            .then_some((width, height))
    }
}
