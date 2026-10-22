//! com/mojang/renderpearl/frontend/shaders/GlslCompiler.java
//!
//! Compiles the game's GLSL to SPIR-V with shaderc, every option as vanilla sets it: Vulkan 1.2,
//! uniforms bound automatically, debug info, no optimisation, and the device's macros before the
//! pipeline's. `createBaseShaderOptions` asks `RenderSystem`'s device whether explicit depth
//! invariance is required, which isn't ported, so the answer is a parameter like the other two.
//!
//! Java makes an include resolver per compile, a closure over the pipeline's `ShaderSource`
//! that LWJGL turns into a C callback; here the callback is a function whose user data points at
//! the compiler and the source, which outlive the compile. A Tracy zone around the compile isn't
//! ported.

use crate::com::mojang::renderpearl::api::pipeline::shader_source::{
    CachedIncludeSource, ShaderSource,
};
use crate::com::mojang::renderpearl::api::pipeline::shader_type::ShaderType;
use crate::com::mojang::renderpearl::frontend::shaders::spirv_module::SPIRVModule;
use crate::com::mojang::renderpearl::util::shader_compile_exception::ShaderCompileException;
use crate::net::minecraft::client::renderer::shader_defines::ShaderDefines;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::org::lwjgl::util::shaderc::shaderc::*;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::sync::Mutex;

pub struct GlslCompiler {
    is_zero_to_one: bool,
    shader_draw_parameters: bool,
    is_explicit_depth_required: bool,
    missing_include_result: CachedIncludeSource,
    malformed_id_result: CachedIncludeSource,
    /// Compilers made so far and not in use, as addresses (`LongArrayList`).
    compilers: Mutex<Vec<usize>>,
}

/// What the include callback is handed: the compiler, for its error results, and the source.
struct IncludeContext<'a> {
    compiler: &'a GlslCompiler,
    shader_source: &'a dyn ShaderSource,
}

impl GlslCompiler {
    pub fn new(
        is_zero_to_one: bool,
        shader_draw_parameters: bool,
        is_explicit_depth_required: bool,
    ) -> GlslCompiler {
        GlslCompiler {
            is_zero_to_one,
            shader_draw_parameters,
            is_explicit_depth_required,
            missing_include_result: CachedIncludeSource::create_error("not found"),
            malformed_id_result: CachedIncludeSource::create_error("malformed id"),
            compilers: Mutex::default(),
        }
    }

    fn process_include<'a>(
        &'a self,
        shader_source: &'a dyn ShaderSource,
        requested_shader: &str,
    ) -> &'a CachedIncludeSource {
        let Some(id) = Identifier::try_parse(requested_shader) else {
            return &self.malformed_id_result;
        };
        shader_source
            .get_include(&id)
            .unwrap_or(&self.missing_include_result)
    }

    /// `ShadercIncludeResolve`: LWJGL reads the requested name with `memASCII`, a byte per char.
    unsafe extern "C" fn resolve_include(
        user_data: *mut c_void,
        requested_source: *const c_char,
        _include_type: c_int,
        _requesting_source: *const c_char,
        _include_depth: usize,
    ) -> *mut ShadercIncludeResult {
        // SAFETY: `compile_to_spv` passes a context that lives until shaderc returns, and shaderc
        // passes a null-terminated name.
        let (context, requested) = unsafe {
            (
                &*(user_data as *const IncludeContext),
                CStr::from_ptr(requested_source),
            )
        };
        let requested_shader: String = requested
            .to_bytes()
            .iter()
            .map(|&byte| byte as char)
            .collect();
        context
            .compiler
            .process_include(context.shader_source, &requested_shader)
            .include_result_ptr()
    }

    /// `releaseIncludeResult`: the results belong to their `ShaderSource`, so there's nothing to do.
    unsafe extern "C" fn release_include_result(
        _user_data: *mut c_void,
        _include_result: *mut ShadercIncludeResult,
    ) {
    }

    fn acquire_compiler(&self) -> Compiler {
        let pooled = self
            .compilers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pop();
        match pooled {
            Some(compiler) => compiler as Compiler,
            None => Shaderc::shaderc_compiler_initialize(),
        }
    }

    fn release_compiler(&self, compiler: Compiler) {
        self.compilers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(compiler as usize);
    }

    fn create_base_shader_options(&self) -> CompileOptions {
        let shader_options = Shaderc::shaderc_compile_options_initialize();
        Shaderc::shaderc_compile_options_set_target_env(
            shader_options,
            SHADERC_TARGET_ENV_VULKAN,
            SHADERC_ENV_VERSION_VULKAN_1_2,
        );
        Shaderc::shaderc_compile_options_set_auto_bind_uniforms(shader_options, true);
        Shaderc::shaderc_compile_options_set_preserve_bindings(shader_options, false);
        Shaderc::shaderc_compile_options_set_generate_debug_info(shader_options);
        Shaderc::shaderc_compile_options_set_optimization_level(
            shader_options,
            SHADERC_OPTIMIZATION_LEVEL_ZERO,
        );
        if self.is_zero_to_one {
            Shaderc::shaderc_compile_options_add_macro_definition(
                shader_options,
                "RENDERPEARL_DEPTH_IS_ZERO_TO_ONE",
                "",
            );
        }
        if self.is_explicit_depth_required {
            Shaderc::shaderc_compile_options_add_macro_definition(
                shader_options,
                "RENDERPEARL_EXPLICIT_DEPTH_INVARIANCE",
                "",
            );
        }
        if self.shader_draw_parameters {
            Shaderc::shaderc_compile_options_add_macro_definition(
                shader_options,
                "RENDERPEARL_INSTANCE_INDEX_INCLUDES_BASE_INSTANCE",
                "",
            );
        }
        shader_options
    }

    pub fn compile_to_spv(
        &self,
        name: &str,
        source: &str,
        shader_type: ShaderType,
        shader_defines: &ShaderDefines,
        shader_source: &dyn ShaderSource,
    ) -> Result<SPIRVModule, ShaderCompileException> {
        let kind = if shader_type == ShaderType::FRAGMENT {
            SHADERC_FRAGMENT_SHADER
        } else {
            SHADERC_VERTEX_SHADER
        };
        let file_name = CString::new(name).expect("shader names have no NUL");
        let entry_point = c"main";
        let shader_options = self.create_base_shader_options();
        for (key, value) in shader_defines.values() {
            Shaderc::shaderc_compile_options_add_macro_definition(shader_options, key, value);
        }
        for flag in shader_defines.flags() {
            Shaderc::shaderc_compile_options_add_macro_definition(shader_options, flag, "");
        }

        let context = IncludeContext {
            compiler: self,
            shader_source,
        };
        // SAFETY: the context outlives the options' last compile, just below.
        unsafe {
            Shaderc::shaderc_compile_options_set_include_callbacks(
                shader_options,
                GlslCompiler::resolve_include,
                GlslCompiler::release_include_result,
                (&context as *const IncludeContext).cast_mut().cast(),
            );
        }
        let compiler = self.acquire_compiler();
        let result = Shaderc::shaderc_compile_into_spv(
            compiler,
            source.as_bytes(),
            kind,
            &file_name,
            entry_point,
            shader_options,
        );
        self.release_compiler(compiler);

        let module = if Shaderc::shaderc_result_get_compilation_status(result)
            != SHADERC_COMPILATION_STATUS_SUCCESS
        {
            Err(ShaderCompileException(format!(
                "Couldn't parse GLSL: {}",
                Shaderc::shaderc_result_get_error_message(result)
            )))
        } else {
            Ok(SPIRVModule::new(
                Shaderc::shaderc_result_get_bytes(result),
                shader_type,
            ))
        };
        Shaderc::shaderc_result_release(result);
        Shaderc::shaderc_compile_options_release(shader_options);
        module
    }
}

impl Drop for GlslCompiler {
    fn drop(&mut self) {
        for compiler in self
            .compilers
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .drain(..)
        {
            Shaderc::shaderc_compiler_release(compiler as Compiler);
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::com::mojang::renderpearl::backend::api::spv_module::SpvModule;
    use crate::golden::Golden;
    use crate::net::minecraft::client::renderer::shader_manager::tests::vanilla_configs;
    use serde_json::Value;

    pub fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    pub fn shader_type(name: &str) -> ShaderType {
        match name {
            "VERTEX" => ShaderType::VERTEX,
            "FRAGMENT" => ShaderType::FRAGMENT,
            _ => panic!("no shader type {name}"),
        }
    }

    /// `[[[key, value], …], [flag, …]]`.
    pub fn defines(value: &Value) -> ShaderDefines {
        let (values, flags): (Vec<(String, String)>, Vec<String>) =
            crate::golden::Decode::decode(value);
        ShaderDefines::new(values, flags)
    }

    /// Compiles the vanilla pack's shader `name` of `shader_type`, with a device's flags
    /// `[isZZeroToOne, shaderDrawParameters, isExplicitDepthRequired]`.
    pub fn compile(
        flags: &[bool],
        name: &str,
        shader_type: ShaderType,
        defines: &ShaderDefines,
    ) -> SPIRVModule {
        let configs = vanilla_configs();
        let source = configs
            .get_shader(&Identifier::parse(name), shader_type)
            .unwrap_or_else(|| panic!("no {shader_type:?} shader {name}"));
        GlslCompiler::new(flags[0], flags[1], flags[2])
            .compile_to_spv(name, source, shader_type, defines, configs)
            .unwrap_or_else(|error| panic!("{error}"))
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.renderpearl.frontend.shaders.GlslCompiler");
        golden.check(
            "compileToSpv",
            |flags: Vec<bool>, name: String, shader: String, shader_defines: Value| {
                hex(compile(
                    &flags,
                    &name,
                    shader_type(&shader),
                    &defines(&shader_defines),
                )
                .spv())
            },
        );
        golden.finish();
    }
}
