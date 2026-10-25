//! org/lwjgl/stb/STBImage.java (LWJGL 3.4.3)
//!
//! stb_image as vanilla runs it, from LWJGL's own `liblwjgl_stb.so`, so the port decodes images
//! with the very same code and gets the same pixels. Only the functions `NativeImage` calls:
//! `stbi_load_from_memory`, `stbi_image_free` and `stbi_failure_reason`, through their JNI entry
//! points (see `LibSTB`).

use crate::org::lwjgl::stb::lib_stb::{LibSTB, NO_ENV};
use crate::org::lwjgl::system::shared_library_loader;
use std::ffi::{CStr, c_char, c_int, c_void};

shared_library_loader::functions! {
    "stb", "lwjgl_stb";
    Java_org_lwjgl_stb_STBImage_nstbi_1load_1from_1memory__JIJJJI: fn(*mut c_void, *mut c_void, i64, c_int, i64, i64, i64, c_int) -> i64;
    Java_org_lwjgl_stb_STBImage_nstbi_1image_1free: fn(*mut c_void, *mut c_void, i64);
    Java_org_lwjgl_stb_STBImage_nstbi_1failure_1reason: fn(*mut c_void, *mut c_void) -> i64;
}

pub struct STBImage;

/// What `stbi_load_from_memory` decoded: stb's buffer of `width * height * channels` bytes,
/// which is freed with `stbi_image_free` when it's dropped.
pub struct StbiImage {
    pixels: *mut u8,
    pub width: i32,
    pub height: i32,
    pub channels_in_file: i32,
    pub channels: i32,
}

impl StbiImage {
    pub fn bytes(&self) -> &[u8] {
        let length = self.width as usize * self.height as usize * self.channels as usize;
        // SAFETY: stb allocated that many bytes, which live until the image is dropped.
        unsafe { std::slice::from_raw_parts(self.pixels, length) }
    }
}

impl Drop for StbiImage {
    fn drop(&mut self) {
        // SAFETY: stb allocated the pixels, and nothing else frees them.
        unsafe {
            (FUNCTIONS.Java_org_lwjgl_stb_STBImage_nstbi_1image_1free)(
                NO_ENV,
                NO_ENV,
                self.pixels as i64,
            )
        }
    }
}

impl STBImage {
    /// `stbi_load_from_memory(buffer, x, y, channels_in_file, desired_channels)`, or `None` where
    /// it returns null.
    pub fn stbi_load_from_memory(buffer: &[u8], desired_channels: i32) -> Option<StbiImage> {
        LibSTB::initialize();
        let (mut x, mut y, mut channels_in_file): (c_int, c_int, c_int) = (0, 0, 0);
        // SAFETY: stb reads `len` bytes of the buffer and writes the three ints.
        let pixels = unsafe {
            (FUNCTIONS.Java_org_lwjgl_stb_STBImage_nstbi_1load_1from_1memory__JIJJJI)(
                NO_ENV,
                NO_ENV,
                buffer.as_ptr() as i64,
                c_int::try_from(buffer.len()).expect("images under 2 GiB"),
                &raw mut x as i64,
                &raw mut y as i64,
                &raw mut channels_in_file as i64,
                desired_channels,
            )
        };
        if pixels == 0 {
            return None;
        }
        Some(StbiImage {
            pixels: pixels as *mut u8,
            width: x,
            height: y,
            channels_in_file,
            channels: if desired_channels != 0 {
                desired_channels
            } else {
                channels_in_file
            },
        })
    }

    /// The calling thread's last failure, which stb keeps in a thread local.
    pub fn stbi_failure_reason() -> Option<String> {
        LibSTB::initialize();
        // SAFETY: stb returns a static string or null.
        let reason = unsafe {
            (FUNCTIONS.Java_org_lwjgl_stb_STBImage_nstbi_1failure_1reason)(NO_ENV, NO_ENV)
        };
        if reason == 0 {
            return None;
        }
        // SAFETY: a NUL-terminated string of stb's.
        let reason = unsafe { CStr::from_ptr(reason as *const c_char) };
        Some(reason.to_string_lossy().into_owned())
    }
}
