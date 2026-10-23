//! net/minecraft/client/renderer/rendertype/PreparedRenderType.java
//!
//! A render type ready to draw this frame: its compiled pipeline (Java compiles it when it binds
//! it, `RenderSystem.getCompiledPipeline`), its transforms' uniform block and its textures. The
//! OIT pipelines and the scissor come with improved transparency and the GUI.

use std::rc::Rc;

use crate::com::mojang::blaze3d::systems::render_system::RenderSystem;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::commands::render_pass::RenderPass;
use crate::com::mojang::renderpearl::api::pipeline::compiled_render_pipeline::CompiledRenderPipeline;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::renderer::staged_vertex_buffer::ExecuteInfo;

pub struct PreparedRenderType {
    pub name: &'static str,
    pub pipeline: Rc<dyn CompiledRenderPipeline>,
    pub dynamic_transforms: Rc<dyn GpuBuffer>,
    pub textures: Vec<Texture>,
}

/// `PreparedRenderType.Texture`.
#[derive(Clone)]
pub struct Texture {
    pub name: String,
    pub texture_view: GpuTextureView,
    pub sampler: Rc<dyn GpuSampler>,
}

/// The record's equality: the same name, pipeline, transforms' slice and textures, each device
/// object by identity (a view by its texture's and its levels).
impl PartialEq for PreparedRenderType {
    fn eq(&self, other: &PreparedRenderType) -> bool {
        self.name == other.name
            && Rc::ptr_eq(&self.pipeline, &other.pipeline)
            && Rc::ptr_eq(&self.dynamic_transforms, &other.dynamic_transforms)
            && self.textures.len() == other.textures.len()
            && self.textures.iter().zip(&other.textures).all(|(a, b)| {
                a.name == b.name
                    && Rc::ptr_eq(a.texture_view.texture(), b.texture_view.texture())
                    && a.texture_view.base_mip_level() == b.texture_view.base_mip_level()
                    && a.texture_view.mip_levels() == b.texture_view.mip_levels()
                    && Rc::ptr_eq(&a.sampler, &b.sampler)
            })
    }
}

impl PreparedRenderType {
    pub fn draw_from_buffer(&self, info: &ExecuteInfo, render_pass: &mut dyn RenderPass) {
        render_pass.push_debug_group(&format!("Render Type {}", self.name));
        render_pass.set_pipeline(&self.pipeline);
        RenderSystem::bind_default_uniforms(render_pass);
        render_pass.set_uniform_buffer("DynamicTransforms", &self.dynamic_transforms);
        render_pass.set_vertex_buffer(0, Some(&gpu_buffer::slice(&info.vertex_buffer)));

        for texture in &self.textures {
            render_pass.set_uniform_sampler(
                &texture.name,
                Some(&texture.texture_view),
                Some(&texture.sampler),
            );
        }

        render_pass.set_index_buffer(&info.index_buffer, info.index_type);
        render_pass.draw_indexed(info.index_count, 1, info.first_index, info.base_vertex, 0);
        render_pass.pop_debug_group();
    }
}
