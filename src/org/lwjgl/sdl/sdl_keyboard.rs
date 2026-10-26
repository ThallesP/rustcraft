//! org/lwjgl/sdl/SDLKeyboard.java (LWJGL 3.4.3)
//!
//! Only starting and stopping text input on a window, which sends its typed text as
//! `SDL_EVENT_TEXT_INPUT` events while it's on.

use crate::org::lwjgl::sdl::sdl_video::SdlWindow;
use crate::org::lwjgl::system::shared_library_loader;

shared_library_loader::functions! {
    "sdl", "SDL3";
    SDL_StartTextInput: fn(SdlWindow) -> bool;
    SDL_StopTextInput: fn(SdlWindow) -> bool;
}

pub struct SDLKeyboard;

#[allow(non_snake_case)]
impl SDLKeyboard {
    /// The window must be one SDL made and hasn't destroyed, which the crate's callers keep to.
    pub(crate) fn SDL_StartTextInput(window: SdlWindow) -> bool {
        // SAFETY: a window SDL made, which the caller keeps alive.
        unsafe { (FUNCTIONS.SDL_StartTextInput)(window) }
    }

    pub(crate) fn SDL_StopTextInput(window: SdlWindow) -> bool {
        // SAFETY: a window SDL made, which the caller keeps alive.
        unsafe { (FUNCTIONS.SDL_StopTextInput)(window) }
    }
}
