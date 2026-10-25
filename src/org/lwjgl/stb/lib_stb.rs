//! org/lwjgl/stb/LibSTB.java (LWJGL 3.4.3)
//!
//! LWJGL builds stb into `liblwjgl_stb.so`, its JNI library, with stb's allocations going through
//! function pointers that `LibSTB`'s static initialiser hands it (`setupMalloc`, LWJGL's
//! `MemoryUtil` allocator, jemalloc in the game). The port hands it the C library's: where stb's
//! memory comes from doesn't change what it decodes, and the port frees what stb hands back
//! through stb (`stbi_image_free`), so the two allocators never mix.
//!
//! Each function is the JNI entry point Java calls (`Java_org_lwjgl_stb_LibSTB_setupMalloc`),
//! which takes the JNI environment and class first and its pointers as `jlong`s. Neither is read
//! by LWJGL's generated code (`UNUSED_PARAMS(__env, clazz)`), so they're null here.

use crate::org::lwjgl::system::shared_library_loader;
use std::ffi::c_void;
use std::sync::Once;

/// `JNIEnv*` and `jclass`, which LWJGL's natives don't read.
pub(crate) const NO_ENV: *mut c_void = std::ptr::null_mut();

shared_library_loader::functions! {
    "stb", "lwjgl_stb";
    Java_org_lwjgl_stb_LibSTB_setupMalloc: fn(*mut c_void, *mut c_void, i64, i64, i64, i64, i64, i64);
}

pub struct LibSTB;

static INITIALIZED: Once = Once::new();

impl LibSTB {
    /// `LibSTB.initialize()`, which every stb class's static initialiser runs first.
    pub fn initialize() {
        INITIALIZED.call_once(|| {
            // SAFETY: the C library's allocator functions, which stb calls with their own
            // signatures; LWJGL's aligned free is the C library's free too.
            unsafe {
                (FUNCTIONS.Java_org_lwjgl_stb_LibSTB_setupMalloc)(
                    NO_ENV,
                    NO_ENV,
                    libc::malloc as *const () as i64,
                    libc::calloc as *const () as i64,
                    libc::realloc as *const () as i64,
                    libc::free as *const () as i64,
                    libc::aligned_alloc as *const () as i64,
                    libc::free as *const () as i64,
                );
            }
        });
    }
}
