//! org/lwjgl/util/shaderc/Shaderc.java (LWJGL 3.4.3)
//!
//! The functions of shaderc's C API (`shaderc/shaderc.h`) the game calls, from the
//! `libshaderc.so` in LWJGL's natives jar: the same build vanilla compiles its shaders with.
//! LWJGL passes a Java string as its UTF-8 bytes and their length, which the `&[u8]` arguments do.

use crate::org::lwjgl::system::shared_library_loader;
use std::ffi::{c_char, c_int, c_void};

/// `shaderc_compiler_t`, `shaderc_compile_options_t` and `shaderc_compilation_result_t`.
pub type Compiler = *mut c_void;
pub type CompileOptions = *mut c_void;
pub type CompilationResult = *mut c_void;

pub const SHADERC_VERTEX_SHADER: c_int = 0;
pub const SHADERC_FRAGMENT_SHADER: c_int = 1;
pub const SHADERC_TARGET_ENV_VULKAN: c_int = 0;
pub const SHADERC_ENV_VERSION_VULKAN_1_2: u32 = (1 << 22) | (2 << 12);
pub const SHADERC_OPTIMIZATION_LEVEL_ZERO: c_int = 0;
pub const SHADERC_COMPILATION_STATUS_SUCCESS: c_int = 0;

/// `shaderc_include_result`: what an include callback hands back, which shaderc reads until the
/// release callback is called with it.
#[repr(C)]
pub struct ShadercIncludeResult {
    pub source_name: *const c_char,
    pub source_name_length: usize,
    pub content: *const c_char,
    pub content_length: usize,
    pub user_data: *mut c_void,
}

/// `shaderc_include_resolve_fn`: `(user_data, requested_source, type, requesting_source,
/// include_depth)`.
pub type ShadercIncludeResolve = unsafe extern "C" fn(
    *mut c_void,
    *const c_char,
    c_int,
    *const c_char,
    usize,
) -> *mut ShadercIncludeResult;

/// `shaderc_include_result_release_fn`.
pub type ShadercIncludeResultRelease = unsafe extern "C" fn(*mut c_void, *mut ShadercIncludeResult);

shared_library_loader::functions! {
    "shaderc", "libshaderc.so";
    shaderc_compiler_initialize: fn() -> Compiler;
    shaderc_compiler_release: fn(Compiler);
    shaderc_compile_options_initialize: fn() -> CompileOptions;
    shaderc_compile_options_release: fn(CompileOptions);
    shaderc_compile_options_add_macro_definition: fn(CompileOptions, *const c_char, usize, *const c_char, usize);
    shaderc_compile_options_set_generate_debug_info: fn(CompileOptions);
    shaderc_compile_options_set_optimization_level: fn(CompileOptions, c_int);
    shaderc_compile_options_set_include_callbacks: fn(CompileOptions, ShadercIncludeResolve, ShadercIncludeResultRelease, *mut c_void);
    shaderc_compile_options_set_target_env: fn(CompileOptions, c_int, u32);
    shaderc_compile_options_set_auto_bind_uniforms: fn(CompileOptions, bool);
    shaderc_compile_options_set_preserve_bindings: fn(CompileOptions, bool);
    shaderc_compile_into_spv: fn(Compiler, *const c_char, usize, c_int, *const c_char, *const c_char, CompileOptions) -> CompilationResult;
    shaderc_result_release: fn(CompilationResult);
    shaderc_result_get_length: fn(CompilationResult) -> usize;
    shaderc_result_get_bytes: fn(CompilationResult) -> *const c_char;
    shaderc_result_get_error_message: fn(CompilationResult) -> *const c_char;
    shaderc_result_get_compilation_status: fn(CompilationResult) -> c_int;
}

/// Shaderc's functions, each a thin wrapper around the C one like LWJGL's.
pub struct Shaderc;

// SAFETY (all): the handles passed in are ones these functions made and haven't released, which
// the callers in `GlslCompiler` keep to.
impl Shaderc {
    pub(crate) fn shaderc_compiler_initialize() -> Compiler {
        unsafe { (FUNCTIONS.shaderc_compiler_initialize)() }
    }

    pub(crate) fn shaderc_compiler_release(compiler: Compiler) {
        unsafe { (FUNCTIONS.shaderc_compiler_release)(compiler) }
    }

    pub(crate) fn shaderc_compile_options_initialize() -> CompileOptions {
        unsafe { (FUNCTIONS.shaderc_compile_options_initialize)() }
    }

    pub(crate) fn shaderc_compile_options_release(options: CompileOptions) {
        unsafe { (FUNCTIONS.shaderc_compile_options_release)(options) }
    }

    /// LWJGL encodes both strings as UTF-8 and passes their lengths; shaderc copies them.
    pub(crate) fn shaderc_compile_options_add_macro_definition(
        options: CompileOptions,
        name: &str,
        value: &str,
    ) {
        unsafe {
            (FUNCTIONS.shaderc_compile_options_add_macro_definition)(
                options,
                name.as_ptr().cast(),
                name.len(),
                value.as_ptr().cast(),
                value.len(),
            )
        }
    }

    pub(crate) fn shaderc_compile_options_set_generate_debug_info(options: CompileOptions) {
        unsafe { (FUNCTIONS.shaderc_compile_options_set_generate_debug_info)(options) }
    }

    pub(crate) fn shaderc_compile_options_set_optimization_level(
        options: CompileOptions,
        level: c_int,
    ) {
        unsafe { (FUNCTIONS.shaderc_compile_options_set_optimization_level)(options, level) }
    }

    /// # Safety
    ///
    /// `user_data` must be valid for `resolver` and `releaser` for as long as `options` compiles.
    pub(crate) unsafe fn shaderc_compile_options_set_include_callbacks(
        options: CompileOptions,
        resolver: ShadercIncludeResolve,
        releaser: ShadercIncludeResultRelease,
        user_data: *mut c_void,
    ) {
        unsafe {
            (FUNCTIONS.shaderc_compile_options_set_include_callbacks)(
                options, resolver, releaser, user_data,
            )
        }
    }

    pub(crate) fn shaderc_compile_options_set_target_env(
        options: CompileOptions,
        target: c_int,
        version: u32,
    ) {
        unsafe { (FUNCTIONS.shaderc_compile_options_set_target_env)(options, target, version) }
    }

    pub(crate) fn shaderc_compile_options_set_auto_bind_uniforms(
        options: CompileOptions,
        auto_bind: bool,
    ) {
        unsafe { (FUNCTIONS.shaderc_compile_options_set_auto_bind_uniforms)(options, auto_bind) }
    }

    pub(crate) fn shaderc_compile_options_set_preserve_bindings(
        options: CompileOptions,
        preserve: bool,
    ) {
        unsafe { (FUNCTIONS.shaderc_compile_options_set_preserve_bindings)(options, preserve) }
    }

    /// `source_text` is passed with its length and without a terminator (`memUTF8(source,
    /// false)`), the file and entry point names null-terminated.
    pub(crate) fn shaderc_compile_into_spv(
        compiler: Compiler,
        source_text: &[u8],
        shader_kind: c_int,
        input_file_name: &std::ffi::CStr,
        entry_point_name: &std::ffi::CStr,
        options: CompileOptions,
    ) -> CompilationResult {
        unsafe {
            (FUNCTIONS.shaderc_compile_into_spv)(
                compiler,
                source_text.as_ptr().cast(),
                source_text.len(),
                shader_kind,
                input_file_name.as_ptr(),
                entry_point_name.as_ptr(),
                options,
            )
        }
    }

    pub(crate) fn shaderc_result_release(result: CompilationResult) {
        unsafe { (FUNCTIONS.shaderc_result_release)(result) }
    }

    pub(crate) fn shaderc_result_get_compilation_status(result: CompilationResult) -> c_int {
        unsafe { (FUNCTIONS.shaderc_result_get_compilation_status)(result) }
    }

    pub(crate) fn shaderc_result_get_error_message(result: CompilationResult) -> String {
        unsafe {
            let message = (FUNCTIONS.shaderc_result_get_error_message)(result);
            if message.is_null() {
                return String::new();
            }
            std::ffi::CStr::from_ptr(message)
                .to_string_lossy()
                .into_owned()
        }
    }

    /// The SPIR-V's bytes, which live as long as the result.
    pub(crate) fn shaderc_result_get_bytes(result: CompilationResult) -> Vec<u8> {
        unsafe {
            let length = (FUNCTIONS.shaderc_result_get_length)(result);
            let bytes = (FUNCTIONS.shaderc_result_get_bytes)(result);
            if length == 0 {
                return Vec::new();
            }
            std::slice::from_raw_parts(bytes.cast::<u8>(), length).to_vec()
        }
    }
}
