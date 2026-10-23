//! org/lwjgl/sdl/SDLEvents.java (LWJGL 3.4.3)
//!
//! Only polling, the event types the window loop reads, and the keyboard's, the mouse's and the
//! mouse buttons' fields (`SDL_KeyboardEvent`, `SDL_MouseMotionEvent`, `SDL_MouseButtonEvent`).

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

impl SdlEvent {
    fn bytes<const N: usize>(&self, offset: usize) -> [u8; N] {
        // The fields after the type, at their offsets in the event.
        self._rest[offset - 4..offset - 4 + N]
            .try_into()
            .expect("in the event")
    }

    /// `SDL_KeyboardEvent.scancode`.
    pub fn key_scancode(&self) -> i32 {
        i32::from_ne_bytes(self.bytes(24))
    }

    /// `SDL_KeyboardEvent.repeat`.
    pub fn key_repeat(&self) -> bool {
        self.bytes::<1>(37)[0] != 0
    }

    /// `SDL_MouseMotionEvent.xrel` and `yrel`.
    pub fn motion_relative(&self) -> (f32, f32) {
        (
            f32::from_ne_bytes(self.bytes(36)),
            f32::from_ne_bytes(self.bytes(40)),
        )
    }

    /// `SDL_MouseButtonEvent.button`.
    pub fn mouse_button(&self) -> u8 {
        self.bytes::<1>(24)[0]
    }
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
    pub const SDL_EVENT_WINDOW_FOCUS_LOST: u32 = 0x20F;
    pub const SDL_EVENT_WINDOW_CLOSE_REQUESTED: u32 = 0x210;
    pub const SDL_EVENT_KEY_DOWN: u32 = 0x300;
    pub const SDL_EVENT_KEY_UP: u32 = 0x301;
    pub const SDL_EVENT_MOUSE_MOTION: u32 = 0x400;
    pub const SDL_EVENT_MOUSE_BUTTON_DOWN: u32 = 0x401;

    pub fn SDL_PollEvent(event: &mut SdlEvent) -> bool {
        // SAFETY: the event is SDL_Event's size, which SDL writes into.
        unsafe { (FUNCTIONS.SDL_PollEvent)(event) }
    }
}
