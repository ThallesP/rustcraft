//! com/mojang/renderpearl/frontend/shaders/SPIRVModule.java
//!
//! Java keeps the SPIRV-Cross context alive for its reflection's getters to ask. The port asks
//! everything the game reads when it reflects (see `SpvModule`), then destroys the context.

use crate::com::mojang::renderpearl::api::pipeline::shader_type::ShaderType;
use crate::com::mojang::renderpearl::backend::api::spv_module::{
    Descriptor, InterfaceVariable, PushConstant, Reflection, SpvModule, Type,
};
use crate::com::mojang::renderpearl::frontend::shaders::spv_util::SpvUtil;
use crate::com::mojang::renderpearl::util::shader_compile_exception::ShaderCompileException;
use crate::org::lwjgl::util::spvc::spvc::*;

/// A module of SPIR-V that shaderc made.
pub struct SPIRVModule {
    spv: Vec<u8>,
    shader_type: ShaderType,
    reflection: Option<Reflection>,
}

impl SPIRVModule {
    pub fn new(spv: Vec<u8>, shader_type: ShaderType) -> SPIRVModule {
        SPIRVModule {
            spv,
            shader_type,
            reflection: None,
        }
    }

    /// The module as words, as `spv.asIntBuffer()` reads it.
    pub fn words(&self) -> Vec<u32> {
        self.spv
            .as_chunks::<4>()
            .0
            .iter()
            .map(|word| u32::from_ne_bytes(*word))
            .collect()
    }

    fn do_reflection(&self) -> Result<Reflection, ShaderCompileException> {
        let spirv = self.words();
        let (result, context) = Spvc::spvc_context_create();
        SpvUtil::throw_if_error(result, "Couldn't create spvc context")?;
        let reflection = SPIRVModule::reflect_in(context, &spirv);
        Spvc::spvc_context_destroy(context);
        reflection
    }

    fn reflect_in(
        context: SpvcContext,
        spirv: &[u32],
    ) -> Result<Reflection, ShaderCompileException> {
        let (result, ir) = Spvc::spvc_context_parse_spirv(context, spirv);
        SpvUtil::throw_if_error(result, "Couldn't parse spirv")?;
        let (result, compiler) = Spvc::spvc_context_create_compiler(
            context,
            SPVC_BACKEND_NONE,
            ir,
            SPVC_CAPTURE_MODE_COPY,
        );
        SpvUtil::throw_if_error(result, "Couldn't create compiler")?;
        let (result, resources) = Spvc::spvc_compiler_create_shader_resources(compiler);
        SpvUtil::throw_if_error(result, "Couldn't create resource list")?;

        let inputs = SPIRVModule::generate_interface_variable_list(
            compiler,
            resources,
            SPVC_RESOURCE_TYPE_STAGE_INPUT,
        )?;
        let outputs = SPIRVModule::generate_interface_variable_list(
            compiler,
            resources,
            SPVC_RESOURCE_TYPE_STAGE_OUTPUT,
        )?;
        let mut descriptors_by_type = Vec::new();
        for descriptor_type in SpvUtil::DESCRIPTOR_TYPES {
            descriptors_by_type.push((
                descriptor_type,
                SPIRVModule::generate_descriptor_list(compiler, resources, descriptor_type)?,
            ));
        }
        let push_constants = SPIRVModule::generate_push_constant_list(compiler, resources)?;
        Ok(Reflection {
            inputs,
            outputs,
            descriptors_by_type,
            push_constants,
        })
    }

    fn generate_interface_variable_list(
        compiler: SpvcCompiler,
        resources: SpvcResources,
        resource_type: i32,
    ) -> Result<Vec<InterfaceVariable>, ShaderCompileException> {
        let (result, list) =
            Spvc::spvc_resources_get_resource_list_for_type(resources, resource_type);
        SpvUtil::throw_if_error(result, "Couldn't list input variables")?;
        let mut variables = Vec::new();
        for resource in list {
            let name = resource.name_string();
            let location_offset = Spvc::spvc_compiler_get_binary_offset_for_decoration(
                compiler,
                resource.id,
                SPV_DECORATION_LOCATION,
            )
            .ok_or_else(|| {
                ShaderCompileException(format!(
                    "Couldn't find byte offset for location decoration of {name}"
                ))
            })?;
            variables.push(InterfaceVariable {
                variable_type: SPIRVModule::type_of(compiler, resource.type_id),
                location_offset: location_offset as usize,
                flat: Spvc::spvc_compiler_get_decoration(
                    compiler,
                    resource.id,
                    SPV_DECORATION_FLAT,
                ),
                component: Spvc::spvc_compiler_get_decoration(
                    compiler,
                    resource.id,
                    SPV_DECORATION_COMPONENT,
                ),
                name,
            });
        }
        Ok(variables)
    }

    fn generate_descriptor_list(
        compiler: SpvcCompiler,
        resources: SpvcResources,
        resource_type: i32,
    ) -> Result<Vec<Descriptor>, ShaderCompileException> {
        let (result, list) =
            Spvc::spvc_resources_get_resource_list_for_type(resources, resource_type);
        SpvUtil::throw_if_error(result, "Couldn't list input variables")?;
        let mut descriptors = Vec::new();
        for resource in list {
            let name = resource.name_string();
            let set_offset = Spvc::spvc_compiler_get_binary_offset_for_decoration(
                compiler,
                resource.id,
                SPV_DECORATION_DESCRIPTOR_SET,
            )
            .ok_or_else(|| {
                ShaderCompileException(format!(
                    "Couldn't find byte offset for set decoration of {name}"
                ))
            })?;
            let binding_offset = Spvc::spvc_compiler_get_binary_offset_for_decoration(
                compiler,
                resource.id,
                SPV_DECORATION_BINDING,
            )
            .ok_or_else(|| {
                ShaderCompileException(format!(
                    "Couldn't find byte offset for binding decoration of {name}"
                ))
            })?;
            descriptors.push(Descriptor {
                descriptor_type: SPIRVModule::type_of(compiler, resource.type_id),
                resource_type,
                set_offset: set_offset as usize,
                binding_offset: binding_offset as usize,
                name,
            });
        }
        Ok(descriptors)
    }

    /// Panics where Java's `Math.toIntExact` throws.
    fn generate_push_constant_list(
        compiler: SpvcCompiler,
        resources: SpvcResources,
    ) -> Result<Vec<PushConstant>, ShaderCompileException> {
        let (result, list) = Spvc::spvc_resources_get_resource_list_for_type(
            resources,
            SPVC_RESOURCE_TYPE_PUSH_CONSTANT,
        );
        SpvUtil::throw_if_error(result, "Couldn't list input variables")?;
        Ok(list
            .iter()
            .map(|resource| {
                let handle = Spvc::spvc_compiler_get_type_handle(compiler, resource.type_id);
                let (_, size) = Spvc::spvc_compiler_get_declared_struct_size(compiler, handle);
                PushConstant {
                    size: i32::try_from(size).expect("integer overflow"),
                }
            })
            .collect())
    }

    /// `SPIRVModule.TypeHandle`'s getters.
    fn type_of(compiler: SpvcCompiler, type_id: u32) -> Type {
        let handle = Spvc::spvc_compiler_get_type_handle(compiler, type_id);
        let base_type = Spvc::spvc_type_get_basetype(handle);
        let dimensions = if base_type != 16 && base_type != 17 {
            i32::MAX
        } else {
            Spvc::spvc_type_get_image_dimension(handle)
        };
        Type {
            base_type,
            dimensions,
            vector_size: Spvc::spvc_type_get_vector_size(handle) as i32,
            array_lengths: (0..Spvc::spvc_type_get_num_array_dimensions(handle))
                .map(|dimension| Spvc::spvc_type_get_array_dimension(handle, dimension) as i32)
                .collect(),
        }
    }
}

impl SpvModule for SPIRVModule {
    fn spv(&self) -> &[u8] {
        &self.spv
    }

    fn shader_type(&self) -> ShaderType {
        self.shader_type
    }

    fn reflect(&mut self) -> Result<&Reflection, ShaderCompileException> {
        if self.reflection.is_none() {
            self.reflection = Some(self.do_reflection()?);
        }
        Ok(self.reflection.as_ref().expect("just reflected"))
    }

    fn get_reflection_info_if_available(&self) -> Option<&Reflection> {
        self.reflection.as_ref()
    }

    fn word(&self, offset: usize) -> u32 {
        let start = offset * 4;
        u32::from_ne_bytes(self.spv[start..start + 4].try_into().expect("four bytes"))
    }

    fn set_word(&mut self, offset: usize, value: u32) {
        let start = offset * 4;
        self.spv[start..start + 4].copy_from_slice(&value.to_ne_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::com::mojang::renderpearl::frontend::shaders::glsl_compiler::tests::{
        compile, defines, shader_type,
    };
    use crate::golden::Golden;
    use serde_json::{Value, json};

    fn encode_type(variable_type: &Type) -> Value {
        json!([
            variable_type.base_type,
            variable_type.dimensions,
            variable_type.vector_size,
            variable_type.array_lengths
        ])
    }

    fn encode_interface_variable(variable: &InterfaceVariable, module: &dyn SpvModule) -> Value {
        json!([
            variable.name,
            variable.location(module),
            variable.has_flat_decoration(),
            variable.component,
            encode_type(&variable.variable_type)
        ])
    }

    /// `[inputs, outputs, descriptors, pushConstants]`.
    fn encode_reflection(module: &mut SPIRVModule) -> Value {
        let reflection = module
            .reflect()
            .unwrap_or_else(|error| panic!("{error}"))
            .clone();
        let inputs: Vec<Value> = reflection
            .inputs
            .iter()
            .map(|variable| encode_interface_variable(variable, module))
            .collect();
        let outputs: Vec<Value> = reflection
            .outputs
            .iter()
            .map(|variable| encode_interface_variable(variable, module))
            .collect();
        let descriptors: Vec<Value> = reflection
            .descriptors()
            .iter()
            .map(|descriptor| {
                json!([
                    descriptor.name,
                    descriptor.resource_type,
                    descriptor.descriptor_set_index(module),
                    descriptor.binding(module),
                    encode_type(&descriptor.descriptor_type)
                ])
            })
            .collect();
        let push_constants: Vec<i32> = reflection
            .push_constants
            .iter()
            .map(|constant| constant.size)
            .collect();
        json!([inputs, outputs, descriptors, push_constants])
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.renderpearl.frontend.shaders.SPIRVModule");
        golden.check(
            "reflect",
            |flags: Vec<bool>, name: String, shader: String, shader_defines: Value| {
                encode_reflection(&mut compile(
                    &flags,
                    &name,
                    shader_type(&shader),
                    &defines(&shader_defines),
                ))
            },
        );
        golden.finish();
    }
}
