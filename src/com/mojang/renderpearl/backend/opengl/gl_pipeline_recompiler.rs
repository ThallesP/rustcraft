//! com/mojang/renderpearl/backend/opengl/GlPipelineRecompiler.java
//!
//! Translates a pipeline's SPIR-V to GLSL 330 with SPIRV-Cross, every option as vanilla sets it
//! outside RenderDoc (`SHADER_DEBUG_MODE`, which is `SharedConstants.IS_RENDERDOC_ATTACHED`):
//! no 420pack, push constants as a uniform buffer, zero-initialised variables, flattened arrays,
//! and the base instance in `gl_InstanceID` when the device has draw parameters. Every interface
//! variable and descriptor is renamed after its location or binding, so the text doesn't depend on
//! the GLSL's names. That text is what the backend hands `glShaderSource`.
//!
//! Not ported yet: `compileShader` and `compileProgram`, which come with the GL device; and the
//! debug mode's names, since vanilla never runs it.

use crate::com::mojang::renderpearl::api::pipeline::shader_type::ShaderType;
use crate::com::mojang::renderpearl::backend::api::backend_render_pipeline::{CreateInfo, Shader};
use crate::com::mojang::renderpearl::frontend::shaders::spv_util::SpvUtil;
use crate::com::mojang::renderpearl::util::shader_compile_exception::ShaderCompileException;
use crate::org::lwjgl::util::spvc::spvc::*;
use std::ffi::CStr;

pub struct GlPipelineRecompiler {
    draw_parameters_supported: bool,
}

impl GlPipelineRecompiler {
    pub const MAX_LOG_LENGTH: i32 = 32768;
    pub const UNIFORM_FORMAT_STRING: &str = "_uniform_%02d_%02d";
    pub const PUSH_CONSTANT_BLOCK_NAME: &str = "_push_constants";

    /// Java also takes the device's `GlDebugLabel`, which only `compileShader` uses.
    pub fn new(draw_parameters_supported: bool) -> GlPipelineRecompiler {
        GlPipelineRecompiler {
            draw_parameters_supported,
        }
    }

    fn decompile_shader(
        &self,
        shader_create_info: &Shader,
    ) -> Result<String, ShaderCompileException> {
        let module = &shader_create_info.module;
        let spirv: Vec<u32> = module
            .spv()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|word| u32::from_ne_bytes(*word))
            .collect();
        let (result, context) = Spvc::spvc_context_create();
        SpvUtil::throw_if_error(result, "Couldn't create spvc context")?;
        let source = self.decompile_in(
            context,
            &spirv,
            module.shader_type(),
            &shader_create_info.entry_point,
        );
        Spvc::spvc_context_destroy(context);
        source
    }

    fn decompile_in(
        &self,
        context: SpvcContext,
        spirv: &[u32],
        shader_type: ShaderType,
        entry_point: &str,
    ) -> Result<String, ShaderCompileException> {
        let (result, ir) = Spvc::spvc_context_parse_spirv(context, spirv);
        SpvUtil::throw_if_error(result, "Couldn't parse spirv")?;
        let (result, compiler) = Spvc::spvc_context_create_compiler(
            context,
            SPVC_BACKEND_GLSL,
            ir,
            SPVC_CAPTURE_MODE_COPY,
        );
        SpvUtil::throw_if_error(result, "Couldn't create compiler")?;
        let (_, options) = Spvc::spvc_compiler_create_compiler_options(compiler);
        Spvc::spvc_compiler_options_set_uint(options, SPVC_COMPILER_OPTION_GLSL_VERSION, 330);
        Spvc::spvc_compiler_options_set_bool(
            options,
            SPVC_COMPILER_OPTION_GLSL_ENABLE_420PACK_EXTENSION,
            false,
        );
        Spvc::spvc_compiler_options_set_bool(
            options,
            SPVC_COMPILER_OPTION_GLSL_EMIT_PUSH_CONSTANT_AS_UNIFORM_BUFFER,
            true,
        );
        Spvc::spvc_compiler_options_set_bool(
            options,
            SPVC_COMPILER_OPTION_GLSL_SUPPORT_NONZERO_BASE_INSTANCE,
            self.draw_parameters_supported,
        );
        Spvc::spvc_compiler_options_set_bool(
            options,
            SPVC_COMPILER_OPTION_FORCE_ZERO_INITIALIZED_VARIABLES,
            true,
        );
        Spvc::spvc_compiler_options_set_bool(
            options,
            SPVC_COMPILER_OPTION_FLATTEN_MULTIDIMENSIONAL_ARRAYS,
            true,
        );

        let (result, resources) = Spvc::spvc_compiler_create_shader_resources(compiler);
        SpvUtil::throw_if_error(result, "Couldn't create resource list")?;
        if shader_type == ShaderType::VERTEX {
            GlPipelineRecompiler::rename_interface_variables(
                compiler,
                resources,
                SPVC_RESOURCE_TYPE_STAGE_INPUT,
                "_vert_input_",
            )?;
        }
        if shader_type == ShaderType::FRAGMENT {
            GlPipelineRecompiler::rename_interface_variables(
                compiler,
                resources,
                SPVC_RESOURCE_TYPE_STAGE_OUTPUT,
                "_frag_output_",
            )?;
        }
        GlPipelineRecompiler::rename_interface_variables(
            compiler,
            resources,
            if shader_type == ShaderType::FRAGMENT {
                SPVC_RESOURCE_TYPE_STAGE_INPUT
            } else {
                SPVC_RESOURCE_TYPE_STAGE_OUTPUT
            },
            "_interface_variable_",
        )?;
        for descriptor_type in SpvUtil::DESCRIPTOR_TYPES {
            GlPipelineRecompiler::rename_descriptors(compiler, resources, descriptor_type)?;
        }
        GlPipelineRecompiler::rename_descriptors(
            compiler,
            resources,
            SPVC_RESOURCE_TYPE_PUSH_CONSTANT,
        )?;

        Spvc::spvc_compiler_set_entry_point(
            compiler,
            entry_point,
            match shader_type {
                ShaderType::VERTEX => SPV_EXECUTION_MODEL_VERTEX,
                ShaderType::FRAGMENT => SPV_EXECUTION_MODEL_FRAGMENT,
            },
        );
        Spvc::spvc_compiler_install_compiler_options(compiler, options);
        let (_, source) = Spvc::spvc_compiler_compile(compiler);
        assert!(!source.is_null(), "SPIRV-Cross made no source");
        // SAFETY: SPIRV-Cross hands out a null-terminated string that lives as long as the context.
        // LWJGL reads it with `memASCII`, a byte per char.
        let bytes = unsafe { CStr::from_ptr(source) }.to_bytes();
        Ok(bytes.iter().map(|&byte| byte as char).collect())
    }

    /// Names each variable `{prefix}{location:02}`, Java's `String.format(…, "%02d")`.
    fn rename_interface_variables(
        compiler: SpvcCompiler,
        resources: SpvcResources,
        resource_type: i32,
        prefix: &str,
    ) -> Result<(), ShaderCompileException> {
        let (result, variables) =
            Spvc::spvc_resources_get_resource_list_for_type(resources, resource_type);
        SpvUtil::throw_if_error(result, "Couldn't get resource list")?;
        for variable in variables {
            let location =
                Spvc::spvc_compiler_get_decoration(compiler, variable.id, SPV_DECORATION_LOCATION);
            Spvc::spvc_compiler_set_name(
                compiler,
                variable.id,
                &format!("{prefix}{:02}", location as i32),
            );
        }
        Ok(())
    }

    fn rename_descriptors(
        compiler: SpvcCompiler,
        resources: SpvcResources,
        resource_type: i32,
    ) -> Result<(), ShaderCompileException> {
        let (result, descriptors) =
            Spvc::spvc_resources_get_resource_list_for_type(resources, resource_type);
        SpvUtil::throw_if_error(result, "Couldn't list input variables")?;
        for resource in descriptors {
            let descriptor_set = Spvc::spvc_compiler_get_decoration(
                compiler,
                resource.id,
                SPV_DECORATION_DESCRIPTOR_SET,
            ) as i32;
            let binding =
                Spvc::spvc_compiler_get_decoration(compiler, resource.id, SPV_DECORATION_BINDING)
                    as i32;
            match resource_type {
                SPVC_RESOURCE_TYPE_UNIFORM_BUFFER | SPVC_RESOURCE_TYPE_STORAGE_BUFFER => {
                    Spvc::spvc_compiler_set_name(
                        compiler,
                        resource.id,
                        &format!("_uniform_instance_{descriptor_set:02}_{binding:02}"),
                    );
                    Spvc::spvc_compiler_set_name(
                        compiler,
                        resource.base_type_id,
                        &format!("_uniform_{descriptor_set:02}_{binding:02}"),
                    );
                }
                SPVC_RESOURCE_TYPE_STORAGE_IMAGE
                | SPVC_RESOURCE_TYPE_SAMPLED_IMAGE
                | SPVC_RESOURCE_TYPE_SEPARATE_IMAGE
                | SPVC_RESOURCE_TYPE_SEPARATE_SAMPLERS => {
                    Spvc::spvc_compiler_set_name(
                        compiler,
                        resource.id,
                        &format!("_uniform_{descriptor_set:02}_{binding:02}"),
                    );
                }
                SPVC_RESOURCE_TYPE_PUSH_CONSTANT => {
                    Spvc::spvc_compiler_set_name(compiler, resource.id, "_push_constants_instance");
                    Spvc::spvc_compiler_set_name(
                        compiler,
                        resource.base_type_id,
                        "_push_constants",
                    );
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Each shader's GLSL, in the create info's order. Java logs the error and returns null.
    pub fn decompile_shaders(
        &self,
        create_info: &CreateInfo,
    ) -> Result<Vec<String>, ShaderCompileException> {
        create_info
            .shaders
            .iter()
            .map(|shader| self.decompile_shader(shader))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::com::mojang::renderpearl::frontend::shaders::pipeline_builder::tests::create_info;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.renderpearl.backend.opengl.GlPipelineRecompiler");
        golden.check("decompileShaders", |flags: Vec<bool>, location: String| {
            let (info, _) = create_info(&flags, &location);
            let sources = GlPipelineRecompiler::new(flags[1])
                .decompile_shaders(&info)
                .unwrap_or_else(|error| panic!("{error}"));
            info.shaders
                .iter()
                .zip(sources)
                .map(|(shader, source)| (shader.name.clone(), source))
                .collect::<Vec<_>>()
        });
        golden.finish();
    }
}
