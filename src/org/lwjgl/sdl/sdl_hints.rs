//! org/lwjgl/sdl/SDLHints.java (LWJGL 3.4.3)

use crate::org::lwjgl::system::shared_library_loader;
use std::ffi::{CString, c_char};

shared_library_loader::functions! {
    "sdl", "SDL3";
    SDL_SetHint: fn(*const c_char, *const c_char) -> bool;
}

pub struct SDLHints;

#[allow(non_snake_case)]
impl SDLHints {
    pub fn SDL_SetHint(name: &str, value: &str) -> bool {
        let name = CString::new(name).expect("no NUL");
        let value = CString::new(value).expect("no NUL");
        // SAFETY: both strings are null-terminated and SDL copies them.
        unsafe { (FUNCTIONS.SDL_SetHint)(name.as_ptr(), value.as_ptr()) }
    }
}
