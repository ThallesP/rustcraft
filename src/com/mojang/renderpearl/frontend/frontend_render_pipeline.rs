//! com/mojang/renderpearl/frontend/FrontendRenderPipeline.java
//!
//! `uniformIndices`, Java's `Object2IntMap`, is the create info's uniform names, whose index is
//! their binding (`PipelineBuilder`'s `uniformBindings`).

use crate::com::mojang::renderpearl::api::pipeline::bind_group_layout::UniformDescription;
use crate::com::mojang::renderpearl::api::pipeline::color_target_state::ColorTargetState;
use crate::com::mojang::renderpearl::api::pipeline::compiled_render_pipeline::CompiledRenderPipeline;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::VertexFormats;
use crate::com::mojang::renderpearl::backend::api::backend_render_pipeline::BackendRenderPipeline;
use std::any::Any;
use std::rc::Rc;

pub struct FrontendRenderPipeline {
    pub name: String,
    pub backend_render_pipeline: Rc<dyn BackendRenderPipeline>,
    pub vertex_formats: VertexFormats,
    pub uniform_indices: Vec<String>,
    pub uniforms: Vec<UniformDescription>,
    pub color_target_states: Vec<Option<ColorTargetState>>,
    pub wants_depth_texture: bool,
    pub push_constant_size: i32,
}

impl FrontendRenderPipeline {
    /// `uniformIndices().getOrDefault(name, -1)`.
    pub fn uniform_index(&self, name: &str) -> i32 {
        self.uniform_indices
            .iter()
            .position(|uniform| uniform == name)
            .map_or(-1, |index| index as i32)
    }
}

impl CompiledRenderPipeline for FrontendRenderPipeline {
    fn is_closed(&self) -> bool {
        self.backend_render_pipeline.is_closed()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
