//! org/lwjgl/util/spvc/Spvc.java (LWJGL 3.4.3)
//!
//! The functions of SPIRV-Cross's C API (`spirv_cross_c.h`) the game calls, from the
//! `libspirv-cross.so` in LWJGL's natives jar: the same build vanilla translates its SPIR-V with.

use crate::org::lwjgl::system::shared_library_loader;
use std::ffi::{CStr, CString, c_char, c_int, c_uint, c_void};

pub type SpvcContext = *mut c_void;
pub type SpvcParsedIr = *mut c_void;
pub type SpvcCompiler = *mut c_void;
pub type SpvcCompilerOptions = *mut c_void;
pub type SpvcResources = *mut c_void;
pub type SpvcType = *const c_void;
pub type SpvcResult = c_int;

pub const SPVC_SUCCESS: SpvcResult = 0;

pub const SPVC_BACKEND_NONE: c_int = 0;
pub const SPVC_BACKEND_GLSL: c_int = 1;
pub const SPVC_CAPTURE_MODE_COPY: c_int = 0;

pub const SPVC_RESOURCE_TYPE_UNIFORM_BUFFER: c_int = 1;
pub const SPVC_RESOURCE_TYPE_STORAGE_BUFFER: c_int = 2;
pub const SPVC_RESOURCE_TYPE_STAGE_INPUT: c_int = 3;
pub const SPVC_RESOURCE_TYPE_STAGE_OUTPUT: c_int = 4;
pub const SPVC_RESOURCE_TYPE_STORAGE_IMAGE: c_int = 6;
pub const SPVC_RESOURCE_TYPE_SAMPLED_IMAGE: c_int = 7;
pub const SPVC_RESOURCE_TYPE_PUSH_CONSTANT: c_int = 9;
pub const SPVC_RESOURCE_TYPE_SEPARATE_IMAGE: c_int = 10;
pub const SPVC_RESOURCE_TYPE_SEPARATE_SAMPLERS: c_int = 11;

pub const SPVC_COMPILER_OPTION_FLATTEN_MULTIDIMENSIONAL_ARRAYS: c_int = 0x0100_0002;
pub const SPVC_COMPILER_OPTION_FORCE_ZERO_INITIALIZED_VARIABLES: c_int = 0x0100_0036;
pub const SPVC_COMPILER_OPTION_GLSL_SUPPORT_NONZERO_BASE_INSTANCE: c_int = 0x0200_0005;
pub const SPVC_COMPILER_OPTION_GLSL_SEPARATE_SHADER_OBJECTS: c_int = 0x0200_0006;
pub const SPVC_COMPILER_OPTION_GLSL_ENABLE_420PACK_EXTENSION: c_int = 0x0200_0007;
pub const SPVC_COMPILER_OPTION_GLSL_VERSION: c_int = 0x0200_0008;
pub const SPVC_COMPILER_OPTION_GLSL_EMIT_PUSH_CONSTANT_AS_UNIFORM_BUFFER: c_int = 0x0200_0021;

pub const SPV_EXECUTION_MODEL_VERTEX: c_int = 0;
pub const SPV_EXECUTION_MODEL_FRAGMENT: c_int = 4;

pub const SPV_DECORATION_FLAT: c_int = 14;
pub const SPV_DECORATION_LOCATION: c_int = 30;
pub const SPV_DECORATION_COMPONENT: c_int = 31;
pub const SPV_DECORATION_BINDING: c_int = 33;
pub const SPV_DECORATION_DESCRIPTOR_SET: c_int = 34;

/// `spvc_reflected_resource`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SpvcReflectedResource {
    pub id: c_uint,
    pub base_type_id: c_uint,
    pub type_id: c_uint,
    pub name: *const c_char,
}

impl SpvcReflectedResource {
    /// LWJGL's `nameString()`.
    pub(crate) fn name_string(&self) -> String {
        // SAFETY: SPIRV-Cross hands out a null-terminated name that lives as long as the context.
        unsafe { CStr::from_ptr(self.name) }
            .to_string_lossy()
            .into_owned()
    }
}

shared_library_loader::functions! {
    "spvc", "libspirv-cross.so";
    spvc_context_create: fn(*mut SpvcContext) -> SpvcResult;
    spvc_context_destroy: fn(SpvcContext);
    spvc_context_parse_spirv: fn(SpvcContext, *const u32, usize, *mut SpvcParsedIr) -> SpvcResult;
    spvc_context_create_compiler: fn(SpvcContext, c_int, SpvcParsedIr, c_int, *mut SpvcCompiler) -> SpvcResult;
    spvc_compiler_create_compiler_options: fn(SpvcCompiler, *mut SpvcCompilerOptions) -> SpvcResult;
    spvc_compiler_options_set_bool: fn(SpvcCompilerOptions, c_int, u8) -> SpvcResult;
    spvc_compiler_options_set_uint: fn(SpvcCompilerOptions, c_int, c_uint) -> SpvcResult;
    spvc_compiler_install_compiler_options: fn(SpvcCompiler, SpvcCompilerOptions) -> SpvcResult;
    spvc_compiler_compile: fn(SpvcCompiler, *mut *const c_char) -> SpvcResult;
    spvc_compiler_create_shader_resources: fn(SpvcCompiler, *mut SpvcResources) -> SpvcResult;
    spvc_resources_get_resource_list_for_type: fn(SpvcResources, c_int, *mut *const SpvcReflectedResource, *mut usize) -> SpvcResult;
    spvc_compiler_get_decoration: fn(SpvcCompiler, c_uint, c_int) -> c_uint;
    spvc_compiler_set_name: fn(SpvcCompiler, c_uint, *const c_char);
    spvc_compiler_set_entry_point: fn(SpvcCompiler, *const c_char, c_int) -> SpvcResult;
    spvc_compiler_get_binary_offset_for_decoration: fn(SpvcCompiler, c_uint, c_int, *mut c_uint) -> u8;
    spvc_compiler_get_type_handle: fn(SpvcCompiler, c_uint) -> SpvcType;
    spvc_compiler_get_declared_struct_size: fn(SpvcCompiler, SpvcType, *mut usize) -> SpvcResult;
    spvc_type_get_basetype: fn(SpvcType) -> c_int;
    spvc_type_get_vector_size: fn(SpvcType) -> c_uint;
    spvc_type_get_num_array_dimensions: fn(SpvcType) -> c_uint;
    spvc_type_get_array_dimension: fn(SpvcType, c_uint) -> c_uint;
    spvc_type_get_image_dimension: fn(SpvcType) -> c_int;
}

/// SPIRV-Cross's functions, each a thin wrapper around the C one like LWJGL's. Where LWJGL
/// returns through a `PointerBuffer`, these return the value with the result code.
pub struct Spvc;

// SAFETY (all): the handles passed in are ones these functions made from a context that hasn't
// been destroyed, which the callers in `SPIRVModule` and `GlPipelineRecompiler` keep to.
impl Spvc {
    pub(crate) fn spvc_context_create() -> (SpvcResult, SpvcContext) {
        let mut context = std::ptr::null_mut();
        let result = unsafe { (FUNCTIONS.spvc_context_create)(&mut context) };
        (result, context)
    }

    pub(crate) fn spvc_context_destroy(context: SpvcContext) {
        unsafe { (FUNCTIONS.spvc_context_destroy)(context) }
    }

    pub(crate) fn spvc_context_parse_spirv(
        context: SpvcContext,
        spirv: &[u32],
    ) -> (SpvcResult, SpvcParsedIr) {
        let mut ir = std::ptr::null_mut();
        let result = unsafe {
            (FUNCTIONS.spvc_context_parse_spirv)(context, spirv.as_ptr(), spirv.len(), &mut ir)
        };
        (result, ir)
    }

    pub(crate) fn spvc_context_create_compiler(
        context: SpvcContext,
        backend: c_int,
        ir: SpvcParsedIr,
        mode: c_int,
    ) -> (SpvcResult, SpvcCompiler) {
        let mut compiler = std::ptr::null_mut();
        let result = unsafe {
            (FUNCTIONS.spvc_context_create_compiler)(context, backend, ir, mode, &mut compiler)
        };
        (result, compiler)
    }

    pub(crate) fn spvc_compiler_create_compiler_options(
        compiler: SpvcCompiler,
    ) -> (SpvcResult, SpvcCompilerOptions) {
        let mut options = std::ptr::null_mut();
        let result =
            unsafe { (FUNCTIONS.spvc_compiler_create_compiler_options)(compiler, &mut options) };
        (result, options)
    }

    pub(crate) fn spvc_compiler_options_set_bool(
        options: SpvcCompilerOptions,
        option: c_int,
        value: bool,
    ) -> SpvcResult {
        unsafe { (FUNCTIONS.spvc_compiler_options_set_bool)(options, option, value as u8) }
    }

    pub(crate) fn spvc_compiler_options_set_uint(
        options: SpvcCompilerOptions,
        option: c_int,
        value: c_uint,
    ) -> SpvcResult {
        unsafe { (FUNCTIONS.spvc_compiler_options_set_uint)(options, option, value) }
    }

    pub(crate) fn spvc_compiler_install_compiler_options(
        compiler: SpvcCompiler,
        options: SpvcCompilerOptions,
    ) -> SpvcResult {
        unsafe { (FUNCTIONS.spvc_compiler_install_compiler_options)(compiler, options) }
    }

    /// The source, or null when compiling failed (which vanilla doesn't check).
    pub(crate) fn spvc_compiler_compile(compiler: SpvcCompiler) -> (SpvcResult, *const c_char) {
        let mut source = std::ptr::null();
        let result = unsafe { (FUNCTIONS.spvc_compiler_compile)(compiler, &mut source) };
        (result, source)
    }

    pub(crate) fn spvc_compiler_create_shader_resources(
        compiler: SpvcCompiler,
    ) -> (SpvcResult, SpvcResources) {
        let mut resources = std::ptr::null_mut();
        let result =
            unsafe { (FUNCTIONS.spvc_compiler_create_shader_resources)(compiler, &mut resources) };
        (result, resources)
    }

    /// The resources of a type, copied out of the context's list.
    pub(crate) fn spvc_resources_get_resource_list_for_type(
        resources: SpvcResources,
        resource_type: c_int,
    ) -> (SpvcResult, Vec<SpvcReflectedResource>) {
        let mut list = std::ptr::null();
        let mut count = 0;
        unsafe {
            let result = (FUNCTIONS.spvc_resources_get_resource_list_for_type)(
                resources,
                resource_type,
                &mut list,
                &mut count,
            );
            let copied = if list.is_null() || count == 0 {
                Vec::new()
            } else {
                std::slice::from_raw_parts(list, count).to_vec()
            };
            (result, copied)
        }
    }

    pub(crate) fn spvc_compiler_get_decoration(
        compiler: SpvcCompiler,
        id: c_uint,
        decoration: c_int,
    ) -> c_uint {
        unsafe { (FUNCTIONS.spvc_compiler_get_decoration)(compiler, id, decoration) }
    }

    /// LWJGL passes the name null-terminated.
    pub(crate) fn spvc_compiler_set_name(compiler: SpvcCompiler, id: c_uint, name: &str) {
        let name = CString::new(name).expect("names have no NUL");
        unsafe { (FUNCTIONS.spvc_compiler_set_name)(compiler, id, name.as_ptr()) }
    }

    pub(crate) fn spvc_compiler_set_entry_point(
        compiler: SpvcCompiler,
        name: &str,
        model: c_int,
    ) -> SpvcResult {
        let name = CString::new(name).expect("names have no NUL");
        unsafe { (FUNCTIONS.spvc_compiler_set_entry_point)(compiler, name.as_ptr(), model) }
    }

    /// The word offset of the decoration's operand, if the id has the decoration.
    pub(crate) fn spvc_compiler_get_binary_offset_for_decoration(
        compiler: SpvcCompiler,
        id: c_uint,
        decoration: c_int,
    ) -> Option<c_uint> {
        let mut offset = 0;
        let found = unsafe {
            (FUNCTIONS.spvc_compiler_get_binary_offset_for_decoration)(
                compiler,
                id,
                decoration,
                &mut offset,
            )
        };
        (found != 0).then_some(offset)
    }

    pub(crate) fn spvc_compiler_get_type_handle(compiler: SpvcCompiler, id: c_uint) -> SpvcType {
        unsafe { (FUNCTIONS.spvc_compiler_get_type_handle)(compiler, id) }
    }

    pub(crate) fn spvc_compiler_get_declared_struct_size(
        compiler: SpvcCompiler,
        struct_type: SpvcType,
    ) -> (SpvcResult, usize) {
        let mut size = 0;
        let result = unsafe {
            (FUNCTIONS.spvc_compiler_get_declared_struct_size)(compiler, struct_type, &mut size)
        };
        (result, size)
    }

    pub(crate) fn spvc_type_get_basetype(handle: SpvcType) -> c_int {
        unsafe { (FUNCTIONS.spvc_type_get_basetype)(handle) }
    }

    pub(crate) fn spvc_type_get_vector_size(handle: SpvcType) -> c_uint {
        unsafe { (FUNCTIONS.spvc_type_get_vector_size)(handle) }
    }

    pub(crate) fn spvc_type_get_num_array_dimensions(handle: SpvcType) -> c_uint {
        unsafe { (FUNCTIONS.spvc_type_get_num_array_dimensions)(handle) }
    }

    pub(crate) fn spvc_type_get_array_dimension(handle: SpvcType, dimension: c_uint) -> c_uint {
        unsafe { (FUNCTIONS.spvc_type_get_array_dimension)(handle, dimension) }
    }

    pub(crate) fn spvc_type_get_image_dimension(handle: SpvcType) -> c_int {
        unsafe { (FUNCTIONS.spvc_type_get_image_dimension)(handle) }
    }
}
