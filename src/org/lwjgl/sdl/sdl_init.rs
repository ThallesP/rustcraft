//! org/lwjgl/sdl/SDLInit.java (LWJGL 3.4.3)

use crate::org::lwjgl::system::shared_library_loader;
use std::ffi::{CString, c_char};

shared_library_loader::functions! {
    "sdl", "SDL3";
    SDL_Init: fn(u32) -> bool;
    SDL_Quit: fn();
    SDL_SetAppMetadataProperty: fn(*const c_char, *const c_char) -> bool;
}

pub struct SDLInit;

#[allow(non_snake_case)]
impl SDLInit {
    pub const SDL_INIT_VIDEO: u32 = 0x20;

    pub fn SDL_Init(flags: u32) -> bool {
        // SAFETY: SDL_Init takes any flags.
        unsafe { (FUNCTIONS.SDL_Init)(flags) }
    }

    pub fn SDL_Quit() {
        // SAFETY: SDL_Quit can be called whether or not SDL is initialised.
        unsafe { (FUNCTIONS.SDL_Quit)() }
    }

    pub fn SDL_SetAppMetadataProperty(name: &str, value: &str) -> bool {
        let name = CString::new(name).expect("no NUL");
        let value = CString::new(value).expect("no NUL");
        // SAFETY: both strings are null-terminated and SDL copies them.
        unsafe { (FUNCTIONS.SDL_SetAppMetadataProperty)(name.as_ptr(), value.as_ptr()) }
    }
}
