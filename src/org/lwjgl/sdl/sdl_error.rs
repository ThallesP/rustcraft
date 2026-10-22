//! org/lwjgl/sdl/SDLError.java (LWJGL 3.4.3)

use crate::org::lwjgl::system::shared_library_loader;
use std::ffi::{CStr, c_char};

shared_library_loader::functions! {
    "sdl", "libSDL3.so";
    SDL_GetError: fn() -> *const c_char;
}

pub struct SDLError;

#[allow(non_snake_case)]
impl SDLError {
    /// The calling thread's last error, or `None` like LWJGL's null.
    pub fn SDL_GetError() -> Option<String> {
        // SAFETY: SDL returns a null-terminated string that lives until the next call.
        let error = unsafe { (FUNCTIONS.SDL_GetError)() };
        (!error.is_null()).then(|| {
            unsafe { CStr::from_ptr(error) }
                .to_string_lossy()
                .into_owned()
        })
    }
}
