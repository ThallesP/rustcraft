//! org/lwjgl/sdl/SDLEvents.java (LWJGL 3.4.3)
//!
//! Only polling, and the event types the window loop reads.

use crate::org::lwjgl::system::shared_library_loader;

shared_library_loader::functions! {
    "sdl", "libSDL3.so";
    SDL_PollEvent: fn(*mut SdlEvent) -> bool;
}

/// `SDL_Event`, a 128-byte union whose first field is its type.
#[repr(C, align(8))]
pub struct SdlEvent {
    pub event_type: u32,
    _rest: [u8; 124],
}

impl Default for SdlEvent {
    fn default() -> SdlEvent {
        SdlEvent {
            event_type: 0,
            _rest: [0; 124],
        }
    }
}

pub struct SDLEvents;

#[allow(non_snake_case)]
impl SDLEvents {
    pub const SDL_EVENT_QUIT: u32 = 0x100;
    pub const SDL_EVENT_WINDOW_CLOSE_REQUESTED: u32 = 0x210;

    pub fn SDL_PollEvent(event: &mut SdlEvent) -> bool {
        // SAFETY: the event is SDL_Event's size, which SDL writes into.
        unsafe { (FUNCTIONS.SDL_PollEvent)(event) }
    }
}
