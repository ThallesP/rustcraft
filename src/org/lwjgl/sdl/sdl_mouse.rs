//! org/lwjgl/sdl/SDLMouse.java (LWJGL 3.4.3)
//!
//! Only the relative mouse mode, which grabs the mouse for turning.

use crate::org::lwjgl::sdl::sdl_video::SdlWindow;
use crate::org::lwjgl::system::shared_library_loader;

shared_library_loader::functions! {
    "sdl", "SDL3";
    SDL_SetWindowRelativeMouseMode: fn(SdlWindow, bool) -> bool;
}

pub struct SDLMouse;

#[allow(non_snake_case)]
impl SDLMouse {
    /// The window must be one SDL made and hasn't destroyed, which the crate's callers keep to.
    pub(crate) fn SDL_SetWindowRelativeMouseMode(window: SdlWindow, enabled: bool) -> bool {
        // SAFETY: a window SDL made, which the caller keeps alive.
        unsafe { (FUNCTIONS.SDL_SetWindowRelativeMouseMode)(window, enabled) }
    }
}
