//! com/mojang/renderpearl/frontend/shaders/PipelineBuilder.java
//!
//! `generateBackendCreateInfo` compiles a pipeline's shaders, checks them against its vertex
//! formats and uniforms, and renumbers each shader's descriptors in place: every descriptor goes
//! to set 0, and each uniform name gets the next binding the first time a stage uses it, vertex
//! stage first. That's the SPIR-V the backends translate. `uniformBindings`, Java's
//! `Object2IntOpenHashMap`, is a `Vec` of names whose index is their binding.
//!
//! `compilePipeline` runs this, hands the create info to the backend device and makes the
//! `FrontendRenderPipeline`. Java runs it on an executor and the backend's second half later
//! (`Pending`); the port runs both at once, which gives the same pipeline.

use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::gpu_format::{ComponentType, GpuFormat};
use crate::com::mojang::renderpearl::api::pipeline::bind_group_layout::{
    BindGroupLayout, UniformDescription,
};
use crate::com::mojang::renderpearl::api::pipeline::polygon_mode::PolygonMode;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::com::mojang::renderpearl::api::pipeline::shader_source::ShaderSource;
use crate::com::mojang::renderpearl::api::pipeline::shader_type::ShaderType;
use crate::com::mojang::renderpearl::api::pipeline::uniform_type::UniformType;
use crate::com::mojang::renderpearl::backend::api::backend_render_pipeline::{
    AttribBinding, CreateInfo, Shader, VertexBuffer,
};
use crate::com::mojang::renderpearl::backend::api::spv_module::{InterfaceVariable, SpvModule};
use crate::com::mojang::renderpearl::frontend::frontend_render_pipeline::FrontendRenderPipeline;
use crate::com::mojang::renderpearl::frontend::shaders::glsl_compiler::GlslCompiler;
use crate::com::mojang::renderpearl::frontend::shaders::spv_util::SpvUtil;
use crate::com::mojang::renderpearl::util::shader_compile_exception::ShaderCompileException;
use std::collections::BTreeMap;
use std::rc::Rc;

pub struct PipelineBuilder {
    compiler: GlslCompiler,
}

/// `PipelineBuilder.InterfaceVariableInfo`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InterfaceVariableInfo {
    base_type: i32,
    vector_length: i32,
    flat_interpolated: bool,
}

fn error<T>(message: String) -> Result<T, ShaderCompileException> {
    Err(ShaderCompileException(message))
}

impl PipelineBuilder {
    /// Java reads the flags from the backend device's `DeviceInfo`: `isZZeroToOne()` and
    /// `features().shaderDrawParameters()`, and `GlslCompiler` its hints' `isExplicitDepthRequired()`.
    pub fn new(
        is_z_zero_to_one: bool,
        shader_draw_parameters: bool,
        is_explicit_depth_required: bool,
    ) -> PipelineBuilder {
        PipelineBuilder {
            compiler: GlslCompiler::new(
                is_z_zero_to_one,
                shader_draw_parameters,
                is_explicit_depth_required,
            ),
        }
    }

    pub fn compiler(&self) -> &GlslCompiler {
        &self.compiler
    }

    /// The pipeline compiled on `backend_device`, or `None` where Java logs why and completes with
    /// `Pending.NULL` or a null pipeline.
    pub fn compile_pipeline(
        &self,
        pipeline: &RenderPipeline,
        shader_source: &dyn ShaderSource,
        backend_device: &dyn GpuDevice,
    ) -> Option<Rc<FrontendRenderPipeline>> {
        if pipeline.get_polygon_mode() == PolygonMode::WIREFRAME
            && !backend_device
                .get_device_info()
                .features
                .wireframe_fill_mode
        {
            eprintln!(
                "Pipeline {} uses {:?} fill mode, not supported by device",
                pipeline.get_location(),
                PolygonMode::WIREFRAME
            );
            return None;
        }
        let mut uniform_bindings = Vec::new();
        let create_info =
            match self.generate_backend_create_info(pipeline, shader_source, &mut uniform_bindings)
            {
                Ok(create_info) => create_info,
                Err(error) => {
                    eprintln!(
                        "Couldn't compile pipeline {}: {error}",
                        pipeline.get_location()
                    );
                    return None;
                }
            };
        let backend_pipeline = backend_device.compile_pipeline(&create_info)?;
        Some(Rc::new(FrontendRenderPipeline {
            name: pipeline.get_location().to_string(),
            backend_render_pipeline: backend_pipeline,
            vertex_formats: *pipeline.get_vertex_format_bindings(),
            uniform_indices: uniform_bindings,
            uniforms: BindGroupLayout::flatten_uniforms(pipeline.get_bind_group_layouts()),
            color_target_states: pipeline.get_color_target_states().to_vec(),
            wants_depth_texture: pipeline.wants_depth_texture(),
            push_constant_size: pipeline.push_constant_size(),
        }))
    }

    /// Java logs the error and returns null, closing the modules it made.
    pub fn generate_backend_create_info(
        &self,
        pipeline: &RenderPipeline,
        shader_source: &dyn ShaderSource,
        uniform_bindings: &mut Vec<String>,
    ) -> Result<CreateInfo, ShaderCompileException> {
        let mut shader_create_infos: Vec<Shader> = Vec::new();
        for (&shader_type, id) in pipeline.get_shaders() {
            let Some(source) = shader_source.get_shader(id, shader_type) else {
                return error(format!(
                    "Couldn't find source for {shader_type:?} shader ({id})"
                ));
            };
            let module = self.compiler.compile_to_spv(
                &id.to_string(),
                source,
                shader_type,
                pipeline.get_shader_defines(),
                shader_source,
            )?;
            shader_create_infos.push(Shader {
                name: id.to_string(),
                entry_point: "main".to_owned(),
                module: Box::new(module),
            });
        }
        let stage = |shader_type: ShaderType| {
            pipeline
                .get_shaders()
                .keys()
                .position(|&stage| stage == shader_type)
        };

        let mut vertex_buffers = Vec::new();
        let mut attrib_bindings = Vec::new();
        if let Some(vertex) = stage(ShaderType::VERTEX) {
            let vertex_stage_name = pipeline.get_shaders()[&ShaderType::VERTEX].to_string();
            let vertex_shader = &mut shader_create_infos[vertex].module;
            let vertex_shader_inputs = vertex_shader.reflect()?.inputs.clone();
            let vertex_shader = &**vertex_shader;
            let mut attrib_formats: BTreeMap<i32, GpuFormat> = BTreeMap::new();
            let mut previous_element_name: Option<&str> = None;
            let mut previous_element_location = 0;
            for (slot, vertex_binding) in pipeline.get_vertex_format_bindings().iter().enumerate() {
                let Some(vertex_binding) = vertex_binding else {
                    continue;
                };
                vertex_buffers.push(VertexBuffer {
                    buffer_slot: slot as i32,
                    stride: vertex_binding.get_vertex_size(),
                    step_rate: vertex_binding.get_step_rate(),
                });
                for element in vertex_binding.get_elements() {
                    let Some(input) = vertex_shader_inputs
                        .iter()
                        .find(|input| input.name == element.name)
                    else {
                        continue;
                    };
                    let attrib_location = if previous_element_name == Some(element.name.as_str()) {
                        previous_element_location + 1
                    } else {
                        input.location(vertex_shader)
                    };
                    attrib_bindings.push(AttribBinding {
                        buffer_slot: slot as i32,
                        location: attrib_location,
                        offset: element.offset,
                        format: element.format,
                    });
                    attrib_formats.insert(attrib_location, element.format);
                    previous_element_name = Some(&element.name);
                    previous_element_location = attrib_location;
                }
            }

            for input in &vertex_shader_inputs {
                if input.component != 0 {
                    return error(format!(
                        "vertex shader ({vertex_stage_name}) attrib ({}) has component decoration, this is not permitted",
                        input.name
                    ));
                }
                let Some(format) = attrib_formats.get(&input.location(vertex_shader)) else {
                    return error(format!(
                        "vertex shader ({vertex_stage_name}) attrib ({}) does not have a matching vertex buffer element",
                        input.name
                    ));
                };
                let expected_base_type = match format.component_type() {
                    ComponentType::UNORM_8
                    | ComponentType::SNORM_8
                    | ComponentType::UNORM_16
                    | ComponentType::SNORM_16
                    | ComponentType::FLOAT_16
                    | ComponentType::FLOAT_32 => 13,
                    ComponentType::UINT_8 | ComponentType::UINT_16 | ComponentType::UINT_32 => 8,
                    ComponentType::SINT_8 | ComponentType::SINT_16 | ComponentType::SINT_32 => 7,
                    other => return error(format!("Unexpected value: {other:?}")),
                };
                if expected_base_type != input.variable_type.base_type {
                    return error(format!(
                        "Unexpected base type for input attribute {} in vertex shader {vertex_stage_name}, expected {} got {}",
                        input.name,
                        SpvUtil::base_type_string(expected_base_type),
                        SpvUtil::base_type_string(input.variable_type.base_type)
                    ));
                }
                if input.variable_type.vector_size > format.component_count() {
                    return error(format!(
                        "Not enough components for input attribute {} in vertex shader {vertex_stage_name}, expected at least {} got {}",
                        input.name,
                        input.variable_type.vector_size,
                        format.component_count()
                    ));
                }
            }
        }

        let (vertex_stage_name, output_slot_map) = match stage(ShaderType::VERTEX) {
            Some(vertex) => {
                let name = pipeline.get_shaders()[&ShaderType::VERTEX].to_string();
                let module = &mut shader_create_infos[vertex].module;
                let outputs = module.reflect()?.outputs.clone();
                let slots = PipelineBuilder::generate_slot_map(
                    &outputs,
                    &**module,
                    &name,
                    ShaderType::VERTEX,
                )?;
                (name, slots)
            }
            None => (String::new(), BTreeMap::new()),
        };
        let fragment_stage_name = pipeline.get_shaders()[&ShaderType::FRAGMENT].to_string();
        let fragment = stage(ShaderType::FRAGMENT).expect("pipelines have a fragment shader");
        let fragment_module = &mut shader_create_infos[fragment].module;
        let fragment_inputs = fragment_module.reflect()?.inputs.clone();
        let input_slot_map = PipelineBuilder::generate_slot_map(
            &fragment_inputs,
            &**fragment_module,
            &vertex_stage_name,
            ShaderType::FRAGMENT,
        )?;

        for (location, input) in &input_slot_map {
            let Some(output) = output_slot_map.get(location) else {
                return error(format!(
                    "Vertex shader ({vertex_stage_name}) missing output at location {location} consumed by Fragment shader ({fragment_stage_name})"
                ));
            };
            if output != input {
                if output.base_type != input.base_type {
                    return error(format!(
                        "Vertex shader ({vertex_stage_name}) and Fragment shader ({fragment_stage_name}) have base type mismatch at location {location} of {} and {}.",
                        SpvUtil::base_type_string(output.base_type),
                        SpvUtil::base_type_string(input.base_type)
                    ));
                }
                if output.vector_length != input.vector_length {
                    return error(format!(
                        "Vertex shader ({vertex_stage_name}) and Fragment shader ({fragment_stage_name}) have vector length mismatch at location {location} of {} and {}.",
                        output.vector_length, input.vector_length
                    ));
                }
                if output.flat_interpolated != input.flat_interpolated {
                    return error(format!(
                        "Vertex shader ({vertex_stage_name}) and Fragment shader ({fragment_stage_name}) have mismatched interpolation at location {location}, flat in {}",
                        if output.flat_interpolated {
                            "vertex"
                        } else {
                            "fragment"
                        }
                    ));
                }
            }
        }

        let pipeline_uniforms =
            BindGroupLayout::flatten_uniforms(pipeline.get_bind_group_layouts());
        let pipeline_uniform_by_name = |name: &str| {
            pipeline_uniforms
                .iter()
                .rev()
                .find(|uniform| uniform.name == name)
        };
        let mut uniform_resource_types: Vec<i32> = Vec::new();
        let mut uniform_dimensions: Vec<i32> = Vec::new();
        let mut uniforms: Vec<UniformDescription> = Vec::new();
        for shader in &mut shader_create_infos {
            let descriptors = shader.module.reflect()?.descriptors();
            for descriptor in descriptors {
                let Some(pipeline_uniform) = pipeline_uniform_by_name(&descriptor.name) else {
                    return error(format!(
                        "Unable to find shader defined uniform ({})",
                        descriptor.name
                    ));
                };
                let new_binding = match uniform_bindings
                    .iter()
                    .position(|name| *name == descriptor.name)
                {
                    Some(binding) => binding,
                    None => {
                        uniform_bindings.push(descriptor.name.clone());
                        uniform_bindings.len() - 1
                    }
                };
                if new_binding >= uniform_resource_types.len() {
                    uniform_resource_types.push(descriptor.resource_type);
                    uniform_dimensions.push(descriptor.descriptor_type.dimensions);
                    uniforms.push(pipeline_uniform.clone());
                } else {
                    if descriptor.resource_type != uniform_resource_types[new_binding] {
                        return error(format!(
                            "Uniform type for {} does not match across all stages",
                            descriptor.name
                        ));
                    }
                    if descriptor.descriptor_type.dimensions != uniform_dimensions[new_binding] {
                        return error(format!(
                            "Uniform dimensions for {} does not match across all stages",
                            descriptor.name
                        ));
                    }
                }
                descriptor.set_binding(&mut *shader.module, new_binding as i32);
                descriptor.set_descriptor_set_index(&mut *shader.module, 0);
            }
        }

        for (uniform_binding, uniform_name) in uniform_bindings.iter().enumerate() {
            let pipeline_uniform = pipeline_uniform_by_name(uniform_name).expect("checked above");
            let defined_resource_type = uniform_resource_types[uniform_binding];
            let expected_resource_type = SpvUtil::resource_type(pipeline_uniform.uniform_type);
            let defined_dimensions = uniform_dimensions[uniform_binding];
            if expected_resource_type != defined_resource_type {
                return error(format!(
                    "Uniform type in shader does not match expected type from RenderPipeline for ({uniform_name})"
                ));
            }
            if defined_dimensions != i32::MAX {
                let expected_uniform_type = match defined_dimensions {
                    0 => return error(format!("1D textures not supported ({uniform_name})")),
                    1 => UniformType::COMBINED_IMAGE_SAMPLER,
                    2 => return error(format!("3D textures not supported ({uniform_name})")),
                    3 => UniformType::COMBINED_IMAGE_SAMPLER,
                    4 => UniformType::COMBINED_IMAGE_SAMPLER,
                    5 => UniformType::TEXEL_BUFFER,
                    _ => {
                        return error(format!(
                            "Unexpected SpvDim ({defined_dimensions}) for uniform ({uniform_name})"
                        ));
                    }
                };
                if pipeline_uniform.uniform_type != expected_uniform_type {
                    return error(format!(
                        "Unexpected dimensions for uniform ({uniform_name}), does not match RenderPipeline definition"
                    ));
                }
            }
        }

        for shader in &mut shader_create_infos {
            let push_constants = &shader.module.reflect()?.push_constants;
            if let Some(push_constant) = push_constants.first() {
                if push_constants.len() > 1 {
                    return error("Shader may define at most one push_constant block".to_owned());
                }
                if push_constant.size > pipeline.push_constant_size() {
                    return error(
                        "Shader push constant size exceeds pipeline declared size".to_owned(),
                    );
                }
            }
        }

        Ok(CreateInfo {
            name: pipeline.get_location().to_string(),
            shaders: shader_create_infos,
            vertex_buffers,
            attrib_bindings,
            uniforms,
            push_constants_size: pipeline.push_constant_size(),
            depth_stencil_state: pipeline.get_depth_stencil_state(),
            polygon_mode: pipeline.get_polygon_mode(),
            cull: pipeline.is_cull(),
            color_target_states: pipeline.get_color_target_states().to_vec(),
            primitive_topology: pipeline.get_primitive_topology(),
        })
    }

    /// Each location an interface variable takes, an array taking one per element.
    fn generate_slot_map(
        interface_variables: &[InterfaceVariable],
        module: &dyn SpvModule,
        stage_name: &str,
        shader_type: ShaderType,
    ) -> Result<BTreeMap<i32, InterfaceVariableInfo>, ShaderCompileException> {
        let mut slot_map = BTreeMap::new();
        for interface_variable in interface_variables {
            let variable_type = &interface_variable.variable_type;
            let base_location = interface_variable.location(module);
            if interface_variable.component != 0 {
                return error(format!(
                    "{} shader ({stage_name}) interface ({}) has component decoration, this is not permitted",
                    shader_type.get_name(),
                    interface_variable.name
                ));
            }
            if matches!(variable_type.base_type, 9 | 10 | 14 | 15) {
                return error(format!(
                    "Unsupported interface variable type {} in {} shader ({stage_name}) location {base_location}",
                    SpvUtil::base_type_string(variable_type.base_type),
                    shader_type.get_name()
                ));
            }

            let info = InterfaceVariableInfo {
                base_type: variable_type.base_type,
                vector_length: variable_type.vector_size,
                flat_interpolated: interface_variable.has_flat_decoration(),
            };
            let total_array_size: i32 = variable_type.array_lengths.iter().product();
            for i in 0..total_array_size {
                slot_map.insert(base_location + i, info);
            }
        }
        Ok(slot_map)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::tests::{
        encode_color_target_state, encode_depth_stencil_state, encode_uniforms,
    };
    use crate::com::mojang::renderpearl::frontend::shaders::glsl_compiler::tests::hex;
    use crate::golden::Golden;
    use crate::net::minecraft::client::renderer::render_pipelines::pipelines_by_location::pipeline;
    use crate::net::minecraft::client::renderer::shader_manager::tests::vanilla_configs;
    use serde_json::{Value, json};

    /// The create info for the pipeline at `location` on a device with `flags`, and its
    /// uniforms' bindings.
    pub fn create_info(flags: &[bool], location: &str) -> (CreateInfo, Vec<String>) {
        let mut uniform_bindings = Vec::new();
        let create_info = PipelineBuilder::new(flags[0], flags[1], flags[2])
            .generate_backend_create_info(
                &pipeline(location),
                vanilla_configs(),
                &mut uniform_bindings,
            )
            .unwrap_or_else(|error| panic!("{error}"));
        (create_info, uniform_bindings)
    }

    fn encode_create_info(info: &CreateInfo, uniform_bindings: &[String]) -> Value {
        let shaders: Vec<Value> = info
            .shaders
            .iter()
            .map(|shader| {
                json!([
                    shader.name,
                    shader.entry_point,
                    format!("{:?}", shader.module.shader_type()),
                    hex(shader.module.spv())
                ])
            })
            .collect();
        let vertex_buffers: Vec<Value> = info
            .vertex_buffers
            .iter()
            .map(|buffer| json!([buffer.buffer_slot, buffer.stride, buffer.step_rate]))
            .collect();
        let attrib_bindings: Vec<Value> = info
            .attrib_bindings
            .iter()
            .map(|binding| {
                json!([
                    binding.buffer_slot,
                    binding.location,
                    binding.offset,
                    format!("{:?}", binding.format)
                ])
            })
            .collect();
        let color_target_states: Vec<Value> = info
            .color_target_states
            .iter()
            .map(|state| {
                state
                    .as_ref()
                    .map_or(Value::Null, encode_color_target_state)
            })
            .collect();
        let bindings: Vec<Value> = uniform_bindings
            .iter()
            .enumerate()
            .map(|(binding, name)| json!([name, binding]))
            .collect();
        json!([
            info.name,
            shaders,
            vertex_buffers,
            attrib_bindings,
            encode_uniforms(&info.uniforms),
            info.push_constants_size,
            info.depth_stencil_state
                .as_ref()
                .map(encode_depth_stencil_state),
            format!("{:?}", info.polygon_mode),
            info.cull,
            color_target_states,
            format!("{:?}", info.primitive_topology),
            bindings
        ])
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.renderpearl.frontend.shaders.PipelineBuilder");
        golden.check(
            "generateBackendCreateInfo",
            |flags: Vec<bool>, location: String| {
                let (info, uniform_bindings) = create_info(&flags, &location);
                encode_create_info(&info, &uniform_bindings)
            },
        );
        golden.finish();
    }
}
