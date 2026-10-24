//! net/minecraft/client/renderer/PostChain.java
//!
//! A post effect: its passes, each a pipeline built from the post processing snippet and the
//! pass's shaders, run over the main target and the effect's own targets (`process`). Java adds the
//! passes to a frame graph (`FrameGraphBuilder`), which runs them in the order their targets need,
//! with each internal target acquired from the game renderer's pool and cleared
//! (`RenderTargetDescriptor.prepare`); for the effects ported (the menu's blur), whose passes each
//! read what the one before wrote, that's the order they were added in, so the port runs them in
//! that order, its internal targets kept from frame to frame like the pool's and cleared as each
//! process starts. Persistent targets and external targets other than the main one aren't ported.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::com::mojang::blaze3d::pipeline::render_target::RenderTarget;
use crate::com::mojang::blaze3d::systems::sampler_cache::SamplerCache;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::pipeline::bind_group_layout::BindGroupLayout;
use crate::com::mojang::renderpearl::api::pipeline::color_target_state::ColorTargetState;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::com::mojang::renderpearl::api::pipeline::uniform_type::UniformType;
use crate::net::minecraft::client::renderer::game_renderer::Pipelines;
use crate::net::minecraft::client::renderer::post_chain_config::{InternalTarget, PostChainConfig};
use crate::net::minecraft::client::renderer::post_pass::PostPass;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::argb::ARGB;

pub struct PostChain {
    id: Identifier,
    passes: Vec<PostPass>,
    internal_targets: BTreeMap<Identifier, InternalTarget>,
    /// The internal targets as the pool keeps them, by the size they were made at.
    allocated: BTreeMap<Identifier, RenderTarget>,
}

impl PostChain {
    pub fn main_target_id() -> Identifier {
        Identifier::with_default_namespace("main")
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    /// `load`: the passes' pipelines and uniforms, which panics where Java throws for targets the
    /// context hasn't got.
    pub fn load(device: &dyn GpuDevice, config: &PostChainConfig, id: &Identifier) -> PostChain {
        for pass in &config.passes {
            for target in pass.referenced_targets() {
                if !config.internal_targets.contains_key(&target)
                    && target != PostChain::main_target_id()
                {
                    panic!(
                        "Referenced external targets are not available in this context: [{target}]"
                    );
                }
            }
        }
        let passes = config
            .passes
            .iter()
            .enumerate()
            .map(|(i, pass)| {
                let location = id.with_suffix(&format!("/{i}"));
                let mut layout = BindGroupLayout::builder();
                for input in &pass.inputs {
                    layout = layout.with_uniform(
                        &format!("{}Sampler", input.sampler_name()),
                        UniformType::COMBINED_IMAGE_SAMPLER,
                    );
                }
                layout = layout.with_uniform("SamplerInfo", UniformType::UNIFORM_BUFFER);
                for group in pass.uniforms.keys() {
                    layout = layout.with_uniform(group, UniformType::UNIFORM_BUFFER);
                }
                let pipeline =
                    RenderPipeline::builder(&[&**RenderPipelines::POST_PROCESSING_SNIPPET])
                        .with_fragment_shader_id(pass.fragment_shader_id.clone())
                        .with_vertex_shader_id(pass.vertex_shader_id.clone())
                        .with_location_id(location)
                        .with_bind_group_layout(&layout.build())
                        .with_color_target_state(ColorTargetState::DEFAULT)
                        .build();
                PostPass::new(
                    device,
                    Arc::new(pipeline),
                    pass.output_target.clone(),
                    &pass.uniforms,
                    pass.inputs.clone(),
                )
            })
            .collect();
        PostChain {
            id: id.clone(),
            passes,
            internal_targets: config.internal_targets.clone(),
            allocated: BTreeMap::new(),
        }
    }

    /// `process(RenderTarget, GraphicsResourceAllocator)`: every pass over `main_target`.
    pub fn process(
        &mut self,
        device: &dyn GpuDevice,
        main_target: &RenderTarget,
        pipelines: &mut Pipelines,
        sampler_cache: &SamplerCache,
    ) {
        let (screen_width, screen_height) = (main_target.width, main_target.height);
        for (id, target) in &self.internal_targets {
            let width = target.width.unwrap_or(screen_width);
            let height = target.height.unwrap_or(screen_height);
            let reusable = self
                .allocated
                .get(id)
                .is_some_and(|allocated| allocated.width == width && allocated.height == height);
            if !reusable {
                // RenderTargetDescriptor.allocate: a TextureTarget with the default depth.
                let mut allocated = RenderTarget::new(
                    None,
                    Some(GpuFormat::RGBA8_UNORM),
                    Some(GpuFormat::D32_FLOAT),
                );
                allocated.create_buffers(device, width, height);
                self.allocated.insert(id.clone(), allocated);
            }
            // RenderTargetDescriptor.prepare: the color cleared to the target's clear color and
            // the depth to 0.
            let allocated = &self.allocated[id];
            device
                .create_command_encoder()
                .clear_color_and_depth_textures(
                    &**allocated.get_color_texture().expect("color"),
                    &ARGB::vector4f_from_argb32(target.clear_color),
                    &**allocated.get_depth_texture().expect("depth"),
                    0.0,
                );
        }
        let targets = PostTargets {
            main: main_target,
            internal: &self.allocated,
        };
        for pass in &self.passes {
            pass.execute(device, pipelines, sampler_cache, &targets);
        }
    }
}

/// The targets a process's passes draw into and sample: the main one and the chain's own.
pub struct PostTargets<'a> {
    main: &'a RenderTarget,
    internal: &'a BTreeMap<Identifier, RenderTarget>,
}

impl<'a> PostTargets<'a> {
    pub fn get(&self, id: &Identifier) -> Option<&'a RenderTarget> {
        if *id == PostChain::main_target_id() {
            Some(self.main)
        } else {
            self.internal.get(id)
        }
    }
}
