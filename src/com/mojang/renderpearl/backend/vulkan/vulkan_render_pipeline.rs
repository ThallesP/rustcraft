//! com/mojang/renderpearl/backend/vulkan/VulkanRenderPipeline.java
//!
//! A pipeline compiled from the frontend's SPIR-V as it is (no SPIRV-Cross step, unlike the GL
//! backend's), its uniforms push descriptors of set 0 in order, a push constant range when it has
//! one, for dynamic rendering: once with the 32-bit float depth attachment every depth target
//! the game makes has, and once more without one when it doesn't test depth. Its viewport and
//! scissor are the render pass's (dynamic state). Its destruction is queued on the encoder when
//! it's dropped (Java's `close()`).

use crate::com::mojang::renderpearl::api::pipeline::bind_group_layout::UniformDescription;
use crate::com::mojang::renderpearl::api::pipeline::blend_function::BlendFunction;
use crate::com::mojang::renderpearl::api::pipeline::uniform_type::UniformType;
use crate::com::mojang::renderpearl::backend::api::backend_render_pipeline::{
    BackendRenderPipeline, CreateInfo,
};
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_const::VulkanConst;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_device::VulkanDevice;
use crate::com::mojang::renderpearl::backend::vulkan::vulkan_utils::VulkanUtils;
use ash::vk::{self, Handle};
use std::any::Any;
use std::cell::Cell;
use std::ffi::CString;
use std::rc::Rc;

pub struct VulkanRenderPipeline {
    device: Rc<VulkanDevice>,
    with_depth_pipeline: vk::Pipeline,
    without_depth_pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    shader_modules: Vec<vk::ShaderModule>,
    uniforms: Vec<UniformDescription>,
    closed: Cell<bool>,
}

impl VulkanRenderPipeline {
    /// `compile(device, pipelineCreateInfo)`. Panics where Java's `crashIfFailure` throws.
    pub fn compile(
        device: &Rc<VulkanDevice>,
        pipeline_create_info: &CreateInfo,
    ) -> VulkanRenderPipeline {
        let vk_device = device.vk_device();
        let debug = device.instance().debug();
        let name = &pipeline_create_info.name;

        let bindings: Vec<vk::DescriptorSetLayoutBinding<'_>> = pipeline_create_info
            .uniforms
            .iter()
            .enumerate()
            .map(|(i, uniform)| {
                vk::DescriptorSetLayoutBinding::default()
                    .descriptor_type(match uniform.uniform_type {
                        UniformType::UNIFORM_BUFFER => vk::DescriptorType::UNIFORM_BUFFER,
                        UniformType::COMBINED_IMAGE_SAMPLER => {
                            vk::DescriptorType::COMBINED_IMAGE_SAMPLER
                        }
                        UniformType::TEXEL_BUFFER => vk::DescriptorType::UNIFORM_TEXEL_BUFFER,
                    })
                    .descriptor_count(1)
                    .binding(i as u32)
                    .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            })
            .collect();
        let set_create_info = vk::DescriptorSetLayoutCreateInfo::default()
            .flags(vk::DescriptorSetLayoutCreateFlags::PUSH_DESCRIPTOR_KHR)
            .bindings(&bindings);
        // SAFETY: the create info lives for the call.
        let descriptor_set_layout = VulkanUtils::crash_if_err(
            unsafe { vk_device.create_descriptor_set_layout(&set_create_info, None) },
            &format!("Can't create descriptor set layout for {name}"),
        );

        let push_constant_ranges = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::from_raw(i32::MAX as u32))
            .offset(0)
            .size(pipeline_create_info.push_constants_size as u32)];
        let set_layouts = [descriptor_set_layout];
        let mut layout_create_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);
        if pipeline_create_info.push_constants_size != 0 {
            layout_create_info = layout_create_info.push_constant_ranges(&push_constant_ranges);
        }
        // SAFETY: as above.
        let pipeline_layout = VulkanUtils::crash_if_err(
            unsafe { vk_device.create_pipeline_layout(&layout_create_info, None) },
            &format!("Can't create pipeline for {name}"),
        );
        debug.set_object_name(
            vk::ObjectType::PIPELINE_LAYOUT,
            pipeline_layout.as_raw(),
            &format!("Pipeline layout for {name}"),
        );

        let mut compiled_shader_modules = Vec::new();
        let entry_points: Vec<CString> = pipeline_create_info
            .shaders
            .iter()
            .map(|shader| CString::new(shader.entry_point.as_str()).expect("no NUL"))
            .collect();
        let mut shader_stages = Vec::new();
        for (shader, entry_point) in pipeline_create_info.shaders.iter().zip(&entry_points) {
            let words: Vec<u32> = shader
                .module
                .spv()
                .as_chunks::<4>()
                .0
                .iter()
                .map(|word| u32::from_le_bytes(*word))
                .collect();
            let info = vk::ShaderModuleCreateInfo::default().code(&words);
            // SAFETY: the SPIR-V is shaderc's, and lives for the call.
            let module = VulkanUtils::crash_if_err(
                unsafe { vk_device.create_shader_module(&info, None) },
                &format!(
                    "Can't compile {} ({:?}) for pipeline {name}",
                    shader.name,
                    shader.module.shader_type()
                ),
            );
            debug.set_object_name(vk::ObjectType::SHADER_MODULE, module.as_raw(), name);
            compiled_shader_modules.push(module);
            shader_stages.push(
                vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::from_raw(
                        VulkanConst::to_vk_shader_type(shader.module.shader_type()) as u32,
                    ))
                    .module(module)
                    .name(entry_point),
            );
        }

        let mut vertex_binding_descriptions = Vec::new();
        let mut vertex_binding_divisor_descriptions = Vec::new();
        for vertex_binding in &pipeline_create_info.vertex_buffers {
            vertex_binding_descriptions.push(
                vk::VertexInputBindingDescription::default()
                    .binding(vertex_binding.buffer_slot as u32)
                    .stride(vertex_binding.stride as u32)
                    .input_rate(if vertex_binding.step_rate > 0 {
                        vk::VertexInputRate::INSTANCE
                    } else {
                        vk::VertexInputRate::VERTEX
                    }),
            );
            if vertex_binding.step_rate > 0 {
                vertex_binding_divisor_descriptions.push(
                    vk::VertexInputBindingDivisorDescriptionEXT::default()
                        .binding(vertex_binding.buffer_slot as u32)
                        .divisor(vertex_binding.step_rate as u32),
                );
            }
        }
        let vertex_attribute_descriptions: Vec<_> = pipeline_create_info
            .attrib_bindings
            .iter()
            .map(|attrib_binding| {
                vk::VertexInputAttributeDescription::default()
                    .location(attrib_binding.location as u32)
                    .binding(attrib_binding.buffer_slot as u32)
                    .offset(attrib_binding.offset as u32)
                    .format(vk::Format::from_raw(VulkanConst::to_vk_format(
                        attrib_binding.format,
                    )))
            })
            .collect();
        let mut vertex_input_divisor_state =
            vk::PipelineVertexInputDivisorStateCreateInfoEXT::default()
                .vertex_binding_divisors(&vertex_binding_divisor_descriptions);
        let mut vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&vertex_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_binding_descriptions);
        if !vertex_binding_divisor_descriptions.is_empty() {
            vertex_input_state = vertex_input_state.push_next(&mut vertex_input_divisor_state);
        }

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default().topology(
            vk::PrimitiveTopology::from_raw(VulkanConst::to_vk_primitive_topology(
                pipeline_create_info.primitive_topology,
            )),
        );
        let mut rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::from_raw(VulkanConst::to_vk_polygon_mode(
                pipeline_create_info.polygon_mode,
            )))
            .cull_mode(if pipeline_create_info.cull {
                vk::CullModeFlags::BACK
            } else {
                vk::CullModeFlags::NONE
            })
            .front_face(vk::FrontFace::CLOCKWISE)
            .line_width(1.0);
        let mut vk_depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::default();
        let depth_stencil_state = pipeline_create_info.depth_stencil_state;
        if let Some(depth_stencil_state) = &depth_stencil_state {
            rasterization_state = rasterization_state
                .depth_bias_enable(
                    depth_stencil_state.depth_bias_constant != 0.0
                        || depth_stencil_state.depth_bias_scale_factor != 0.0,
                )
                .depth_bias_constant_factor(depth_stencil_state.depth_bias_constant)
                .depth_bias_slope_factor(depth_stencil_state.depth_bias_scale_factor);
            vk_depth_stencil_state = vk_depth_stencil_state
                .depth_test_enable(true)
                .depth_write_enable(depth_stencil_state.write_depth)
                .depth_compare_op(vk::CompareOp::from_raw(VulkanConst::to_vk_compare_op(
                    depth_stencil_state.depth_test,
                )));
        }

        let color_target_states = &pipeline_create_info.color_target_states;
        let blend_attachments: Vec<vk::PipelineColorBlendAttachmentState> = color_target_states
            .iter()
            .map(|color_target_state| {
                let mut attachment = vk::PipelineColorBlendAttachmentState::default()
                    .color_write_mask(vk::ColorComponentFlags::from_raw(
                        color_target_state
                            .as_ref()
                            .map_or(0, VulkanConst::to_vk_color_target_state)
                            as u32,
                    ));
                if let Some(blend_function) = color_target_state
                    .as_ref()
                    .and_then(|state| state.blend_function)
                {
                    attachment =
                        VulkanRenderPipeline::apply_blend_information(attachment, &blend_function);
                }
                attachment
            })
            .collect();
        let color_blend_state =
            vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachments);
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .scissor_count(1)
            .viewport_count(1);
        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);
        let dynamic_states = [vk::DynamicState::SCISSOR, vk::DynamicState::VIEWPORT];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let color_attachment_formats: Vec<vk::Format> = color_target_states
            .iter()
            .map(|color_target_state| {
                vk::Format::from_raw(
                    color_target_state
                        .as_ref()
                        .map_or(0, |state| VulkanConst::to_vk_format(state.format)),
                )
            })
            .collect();

        let compile = |depth_attachment_format: vk::Format| -> vk::Pipeline {
            let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
                .color_attachment_formats(&color_attachment_formats)
                .depth_attachment_format(depth_attachment_format);
            let create_info = vk::GraphicsPipelineCreateInfo::default()
                .flags(vk::PipelineCreateFlags::empty())
                .stages(&shader_stages)
                .vertex_input_state(&vertex_input_state)
                .input_assembly_state(&input_assembly_state)
                .rasterization_state(&rasterization_state)
                .depth_stencil_state(&vk_depth_stencil_state)
                .color_blend_state(&color_blend_state)
                .viewport_state(&viewport_state)
                .multisample_state(&multisample_state)
                .dynamic_state(&dynamic_state_info)
                .layout(pipeline_layout)
                .push_next(&mut rendering_info);
            // SAFETY: every state the create info points at lives for the call.
            let pipelines = unsafe {
                vk_device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
            }
            .map_err(|(_, error)| error);
            let pipeline =
                VulkanUtils::crash_if_err(pipelines, &format!("Can't compile pipeline {name}"))[0];
            debug.set_object_name(
                vk::ObjectType::PIPELINE,
                pipeline.as_raw(),
                &format!("Pipeline {name}"),
            );
            pipeline
        };
        let with_depth_pipeline = compile(vk::Format::D32_SFLOAT);
        let without_depth_pipeline = if depth_stencil_state.is_none() {
            compile(vk::Format::UNDEFINED)
        } else {
            vk::Pipeline::null()
        };

        VulkanRenderPipeline {
            device: device.clone(),
            with_depth_pipeline,
            without_depth_pipeline,
            pipeline_layout,
            descriptor_set_layout,
            shader_modules: compiled_shader_modules,
            uniforms: pipeline_create_info.uniforms.clone(),
            closed: Cell::new(false),
        }
    }

    fn apply_blend_information(
        blend_attachment: vk::PipelineColorBlendAttachmentState,
        blend_function: &BlendFunction,
    ) -> vk::PipelineColorBlendAttachmentState {
        let factor = |factor| vk::BlendFactor::from_raw(VulkanConst::to_vk_blend_factor(factor));
        blend_attachment
            .blend_enable(true)
            .color_blend_op(vk::BlendOp::from_raw(VulkanConst::to_vk_blend_op(
                blend_function.color.op,
            )))
            .alpha_blend_op(vk::BlendOp::from_raw(VulkanConst::to_vk_blend_op(
                blend_function.alpha.op,
            )))
            .dst_alpha_blend_factor(factor(blend_function.alpha.dest_factor))
            .dst_color_blend_factor(factor(blend_function.color.dest_factor))
            .src_alpha_blend_factor(factor(blend_function.alpha.source_factor))
            .src_color_blend_factor(factor(blend_function.color.source_factor))
    }

    pub fn with_depth_pipeline(&self) -> vk::Pipeline {
        self.with_depth_pipeline
    }

    pub fn without_depth_pipeline(&self) -> vk::Pipeline {
        self.without_depth_pipeline
    }

    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout
    }

    pub fn uniforms(&self) -> &[UniformDescription] {
        &self.uniforms
    }
}

impl BackendRenderPipeline for VulkanRenderPipeline {
    fn is_closed(&self) -> bool {
        self.closed.get()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for VulkanRenderPipeline {
    /// `close()`, then `destroy()` once the GPU is done with it.
    fn drop(&mut self) {
        self.closed.set(true);
        let device = self.device.clone();
        let pipelines = [self.without_depth_pipeline, self.with_depth_pipeline];
        let pipeline_layout = self.pipeline_layout;
        let descriptor_set_layout = self.descriptor_set_layout;
        let shader_modules = std::mem::take(&mut self.shader_modules);
        self.device
            .create_command_encoder()
            .queue_for_destroy(Box::new(move || {
                let vk_device = device.vk_device();
                // SAFETY: the objects are the device's, and the GPU is done with them.
                unsafe {
                    for pipeline in pipelines {
                        vk_device.destroy_pipeline(pipeline, None);
                    }
                    vk_device.destroy_pipeline_layout(pipeline_layout, None);
                    vk_device.destroy_descriptor_set_layout(descriptor_set_layout, None);
                    for module in shader_modules {
                        vk_device.destroy_shader_module(module, None);
                    }
                }
            }));
    }
}
