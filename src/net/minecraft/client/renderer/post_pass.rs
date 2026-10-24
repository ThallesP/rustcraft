//! net/minecraft/client/renderer/PostPass.java
//!
//! One pass of a post effect: a full-screen triangle drawn into its output target with its
//! pipeline, sampling its inputs (with `SamplerInfo`, the output's and each input's size) and its
//! uniform blocks, the default uniforms bound. Java writes `SamplerInfo` into a ring buffer it
//! rotates each frame; here each pass makes a buffer of the same bytes. The orthographic
//! projection Java sets around the pass isn't read by the post shaders' vertex shader
//! (`core/screenquad`, which places the triangle itself), so the port leaves the projection as it
//! is. Texture inputs (`TextureInput`, a texture by location) aren't ported: no vanilla effect the
//! client draws yet has one.

use std::rc::Rc;
use std::sync::Arc;

use crate::com::mojang::blaze3d::buffers::std140_builder::Std140Builder;
use crate::com::mojang::blaze3d::buffers::std140_size_calculator::Std140SizeCalculator;
use crate::com::mojang::blaze3d::systems::render_system::RenderSystem;
use crate::com::mojang::blaze3d::systems::sampler_cache::SamplerCache;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::com::mojang::renderpearl::api::textures::filter_mode::FilterMode;
use crate::net::minecraft::client::renderer::game_renderer::Pipelines;
use crate::net::minecraft::client::renderer::post_chain::PostTargets;
use crate::net::minecraft::client::renderer::post_chain_config::Input;
use crate::net::minecraft::client::renderer::uniform_value::UniformValue;
use crate::net::minecraft::resources::identifier::Identifier;

pub struct PostPass {
    name: String,
    pipeline: Arc<RenderPipeline>,
    output_target_id: Identifier,
    custom_uniforms: Vec<(String, Rc<dyn GpuBuffer>)>,
    inputs: Vec<Input>,
}

impl PostPass {
    pub fn new(
        device: &dyn GpuDevice,
        pipeline: Arc<RenderPipeline>,
        output_target_id: Identifier,
        uniform_groups: &std::collections::BTreeMap<String, Vec<UniformValue>>,
        inputs: Vec<Input>,
    ) -> PostPass {
        let name = pipeline.get_location().to_string();
        let mut custom_uniforms = Vec::new();
        for (group, uniforms) in uniform_groups {
            if uniforms.is_empty() {
                continue;
            }
            let size = uniforms
                .iter()
                .fold(Std140SizeCalculator::new(), |calculator, uniform| {
                    uniform.add_size(calculator)
                })
                .get();
            let mut builder = Std140Builder::on_stack(size);
            for uniform in uniforms {
                uniform.write_to(&mut builder);
            }
            let buffer = device.create_buffer_with_data(
                Some(&format!("{name} / {group}")),
                gpu_buffer::USAGE_UNIFORM,
                &builder.get(),
            );
            custom_uniforms.push((group.clone(), buffer));
        }
        PostPass {
            name,
            pipeline,
            output_target_id,
            custom_uniforms,
            inputs,
        }
    }

    pub fn output_target_id(&self) -> &Identifier {
        &self.output_target_id
    }

    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    /// The pass's `executes` task: the output target drawn into, its inputs found among `targets`.
    pub fn execute(
        &self,
        device: &dyn GpuDevice,
        pipelines: &mut Pipelines,
        sampler_cache: &SamplerCache,
        targets: &PostTargets,
    ) {
        let output_target = targets
            .get(&self.output_target_id)
            .unwrap_or_else(|| panic!("Missing handle for target {}", self.output_target_id));
        let input_textures: Vec<_> = self
            .inputs
            .iter()
            .map(|input| match input {
                Input::Target {
                    sampler_name,
                    target_id,
                    use_depth_buffer,
                    bilinear,
                } => {
                    let target = targets
                        .get(target_id)
                        .unwrap_or_else(|| panic!("Missing handle for target {target_id}"));
                    let view = if *use_depth_buffer {
                        target.get_depth_texture_view()
                    } else {
                        target.get_color_texture_view()
                    }
                    .unwrap_or_else(|| panic!("Missing texture for target {target_id}"))
                    .clone();
                    let filter = if *bilinear {
                        FilterMode::LINEAR
                    } else {
                        FilterMode::NEAREST
                    };
                    (
                        sampler_name.clone(),
                        view,
                        Rc::clone(sampler_cache.get_clamp_to_edge(filter)),
                    )
                }
                Input::Texture { location, .. } => {
                    panic!("a post pass's texture input ({location}) isn't ported")
                }
            })
            .collect();

        let mut info = Std140Builder::on_stack(((self.inputs.len() + 1) * 8) as i32);
        info.put_vec2(output_target.width as f32, output_target.height as f32);
        for (_, view, _) in &input_textures {
            info.put_vec2(view.get_width(0) as f32, view.get_height(0) as f32);
        }
        let info_ubo = device.create_buffer_with_data(
            Some(&format!("{} SamplerInfo", self.name)),
            gpu_buffer::USAGE_UNIFORM,
            &info.get(),
        );

        let compiled = pipelines.get(device, &self.pipeline);
        let encoder = device.create_command_encoder();
        let mut render_pass = encoder.create_render_pass_with(
            &format!("Post pass {}", self.name),
            output_target
                .get_color_texture_view()
                .expect("a post target has color"),
            None,
            output_target.get_depth_texture_view(),
            None,
        );
        render_pass.set_pipeline(&compiled);
        RenderSystem::bind_default_uniforms(&mut *render_pass);
        render_pass.set_uniform_buffer("SamplerInfo", &info_ubo);
        for (name, buffer) in &self.custom_uniforms {
            render_pass.set_uniform_buffer(name, buffer);
        }
        for (sampler_name, view, sampler) in &input_textures {
            render_pass.set_uniform_sampler(
                &format!("{sampler_name}Sampler"),
                Some(view),
                Some(sampler),
            );
        }
        render_pass.draw(3, 1, 0, 0);
    }
}
