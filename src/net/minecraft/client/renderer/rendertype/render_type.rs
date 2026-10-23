//! net/minecraft/client/renderer/rendertype/RenderType.java
//!
//! A way of drawing: a name and a `RenderSetup`. Java compares render types by identity (each is
//! made once, `RenderTypes`' memoized factories), so they're shared as `Rc`s and compared with
//! `Rc::ptr_eq`. Not ported yet: the outline render type a type that affects outlines makes
//! (`outline()`, with the entity outlines), and the scissor (`getScissorStateForRenderTypeDraws`,
//! the GUI's).

use std::rc::Rc;

use crate::com::mojang::blaze3d::systems::render_system::RenderSystem;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::com::mojang::renderpearl::api::vertex::vertex_format::VertexFormat;
use crate::net::minecraft::client::renderer::dynamic_gpu_data::Transform;
use crate::net::minecraft::client::renderer::feature::feature_frame_context::FeatureFrameContext;
use crate::net::minecraft::client::renderer::rendertype::prepared_render_type::PreparedRenderType;
use crate::net::minecraft::client::renderer::rendertype::render_setup::{
    OutlineProperty, RenderSetup, TextureContext,
};

pub struct RenderType {
    state: RenderSetup,
    has_blending: bool,
    pub(crate) name: &'static str,
}

impl RenderType {
    pub const MEGABYTE: i32 = 1048576;
    pub const BIG_BUFFER_SIZE: i32 = 4194304;
    pub const SMALL_BUFFER_SIZE: i32 = 786432;
    pub const TRANSIENT_BUFFER_SIZE: i32 = 1536;

    pub(crate) fn create(name: &'static str, state: RenderSetup) -> Rc<RenderType> {
        let has_blending = state
            .pipeline
            .get_color_target_states()
            .iter()
            .flatten()
            .any(|color_target_state| color_target_state.blend_function.is_some());
        Rc::new(RenderType {
            state,
            has_blending,
            name,
        })
    }

    pub fn has_blending(&self) -> bool {
        self.has_blending
    }

    /// The type ready to draw this frame: its textures, and its transforms for the model view
    /// matrix as it is now.
    pub fn prepare(&self, context: &mut FeatureFrameContext<'_>) -> PreparedRenderType {
        let textures = self.state.prepare_textures(&mut TextureContext {
            device: context.device,
            texture_manager: context.texture_manager,
            sampler_cache: context.sampler_cache,
            overlay_texture: context.overlay_texture,
            lightmap_texture: context.lightmap_texture,
        });
        let mut model_view_matrix = RenderSystem::get_model_view_matrix_copy();
        self.state
            .layering_transform
            .apply(&mut model_view_matrix, context.projection_type);
        let mut transform = Transform::of_model_view(model_view_matrix);
        transform.texture_matrix = self.state.texture_transform.create_matrix();
        let dynamic_transforms = context.write_transform(transform.write());
        PreparedRenderType {
            name: self.name,
            pipeline: context.pipelines.get(context.device, &self.state.pipeline),
            dynamic_transforms,
            textures,
        }
    }

    pub fn format(&self) -> &'static VertexFormat {
        self.state
            .pipeline
            .get_vertex_format_binding(0)
            .expect("a render type's pipeline has a vertex format")
    }

    pub fn primitive_topology(&self) -> PrimitiveTopology {
        self.state.pipeline.get_primitive_topology()
    }

    pub fn is_outline(&self) -> bool {
        self.state.outline_property == OutlineProperty::IS_OUTLINE
    }

    pub fn pipeline(&self) -> &RenderPipeline {
        &self.state.pipeline
    }

    pub fn affects_crumbling(&self) -> bool {
        self.state.affects_crumbling
    }

    pub fn can_consolidate_consecutive_geometry(&self) -> bool {
        !self.primitive_topology().connected_primitives()
    }

    pub fn sort_on_upload(&self) -> bool {
        self.state.sort_on_upload
    }

    pub fn force_solid_model_phase(&self) -> bool {
        self.state.force_solid_model_phase
    }
}
