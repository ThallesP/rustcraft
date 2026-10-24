//! org/lwjgl/sdl/SDLEvents.java (LWJGL 3.4.3)
//!
//! Only polling, the event types the window loop reads, and the keyboard's, the text input's, the
//! mouse's, the mouse buttons' and the window's fields (`SDL_KeyboardEvent`, `SDL_TextInputEvent`,
//! `SDL_MouseMotionEvent`, `SDL_MouseButtonEvent`, `SDL_WindowEvent`).

use crate::org::lwjgl::system::shared_library_loader;

shared_library_loader::functions! {
    "sdl", "SDL3";
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

    /// `SDL_KeyboardEvent.key`: the keycode in the keyboard's layout.
    pub fn key_keycode(&self) -> i32 {
        i32::from_ne_bytes(self.bytes(28))
    }

    /// `SDL_KeyboardEvent.mod`: the modifiers held.
    pub fn key_modifiers(&self) -> i32 {
        i32::from(u16::from_ne_bytes(self.bytes(32)))
    }

    /// `SDL_TextInputEvent.text`: the UTF-8 text typed, which SDL keeps until the next poll.
    pub fn text_input(&self) -> String {
        let pointer = usize::from_ne_bytes(self.bytes(24)) as *const std::ffi::c_char;
        if pointer.is_null() {
            return String::new();
        }
        // SAFETY: SDL's null-terminated text, alive until the next SDL_PollEvent.
        unsafe { std::ffi::CStr::from_ptr(pointer) }
            .to_string_lossy()
            .into_owned()
    }

    /// `SDL_MouseMotionEvent.x` and `y`: the mouse in the window.
    pub fn motion_position(&self) -> (f32, f32) {
        (
            f32::from_ne_bytes(self.bytes(28)),
            f32::from_ne_bytes(self.bytes(32)),
        )
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

    /// `SDL_WindowEvent.data1` and `data2`: the new size, for a resize.
    pub fn window_data(&self) -> (i32, i32) {
        (
            i32::from_ne_bytes(self.bytes(20)),
            i32::from_ne_bytes(self.bytes(24)),
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
    pub const SDL_EVENT_WINDOW_RESIZED: u32 = 0x206;
    pub const SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED: u32 = 0x207;
    pub const SDL_EVENT_WINDOW_FOCUS_LOST: u32 = 0x20F;
    pub const SDL_EVENT_WINDOW_CLOSE_REQUESTED: u32 = 0x210;
    pub const SDL_EVENT_KEY_DOWN: u32 = 0x300;
    pub const SDL_EVENT_KEY_UP: u32 = 0x301;
    pub const SDL_EVENT_TEXT_INPUT: u32 = 0x303;
    pub const SDL_EVENT_MOUSE_MOTION: u32 = 0x400;
    pub const SDL_EVENT_MOUSE_BUTTON_DOWN: u32 = 0x401;
    pub const SDL_EVENT_MOUSE_BUTTON_UP: u32 = 0x402;

    pub fn SDL_PollEvent(event: &mut SdlEvent) -> bool {
        // SAFETY: the event is SDL_Event's size, which SDL writes into.
        unsafe { (FUNCTIONS.SDL_PollEvent)(event) }
    }
}
