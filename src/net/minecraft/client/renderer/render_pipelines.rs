//! net/minecraft/client/renderer/RenderPipelines.java
//!
//! Every pipeline takes the next sort key when it's built, which orders the GUI's draws, so the
//! pipelines are built together and in Java's order: they're the fields of one struct, made on
//! first use (`RenderPipelines::get()`). Snippets take no sort key, so each is a static of its
//! own, the public ones also as associated consts (`RenderPipelines::GLINT_SNIPPET`).
//! `PIPELINES_BY_LOCATION` and `OPTIONAL_PIPELINES_BY_LOCATION` are Java `HashMap`s, whose order
//! `requiredPipelines()` and `optionalPipelines()` list them in, so they're the ported one.

use crate::com::mojang::blaze3d::vertex::default_vertex_format::DefaultVertexFormat;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::pipeline::blend_factor::BlendFactor;
use crate::com::mojang::renderpearl::api::pipeline::blend_function::BlendFunction;
use crate::com::mojang::renderpearl::api::pipeline::color_target_state::ColorTargetState;
use crate::com::mojang::renderpearl::api::pipeline::compare_op::CompareOp;
use crate::com::mojang::renderpearl::api::pipeline::depth_stencil_state::DepthStencilState;
use crate::com::mojang::renderpearl::api::pipeline::polygon_mode::PolygonMode;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::{RenderPipeline, Snippet};
use crate::java::util::hash_map::HashMap;
use crate::net::minecraft::client::renderer::bind_group_layouts::BindGroupLayouts;
use crate::net::minecraft::client::renderer::level_renderer::LevelRenderer;
use crate::net::minecraft::client::renderer::oit::oit_pipeline_set::OitPipelineSet;
use crate::net::minecraft::resources::identifier::Identifier;
use std::sync::{Arc, LazyLock};

const ALPHA_CUTOUT_THRESHOLD_DEFAULT: f32 = 0.1;
const ALPHA_CUTOUT_THRESHOLD_CUTOUT_TERRAIN: f32 = 0.5;

type StaticSnippet = LazyLock<Snippet>;

macro_rules! snippet {
    ($name:ident = $value:expr) => {
        static $name: StaticSnippet = LazyLock::new(|| $value);
    };
}

snippet!(
    GLOBALS_SNIPPET = RenderPipeline::builder(&[])
        .with_bind_group_layout(BindGroupLayouts::GLOBALS)
        .build_snippet()
);
snippet!(
    MATRICES_FOG_SNIPPET = RenderPipeline::builder(&[&GLOBALS_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::PROJECTION)
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_bind_group_layout(BindGroupLayouts::FOG)
        .build_snippet()
);
snippet!(
    MATRICES_FOG_LIGHT_DIR_SNIPPET = RenderPipeline::builder(&[&MATRICES_FOG_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::LIGHTING)
        .build_snippet()
);
snippet!(
    OIT_SNIPPET = RenderPipeline::builder(&[&GLOBALS_SNIPPET])
        .with_shader_define("OIT")
        .with_shader_define_i32("OIT_WAVELET_RANK", 2)
        .with_shader_define_i32("OIT_COEFF_COUNT", LevelRenderer::OIT_COEFFICIENT_COUNT)
        .with_shader_define_i32(
            "OIT_COEFF_ATTACHMENT_COUNT",
            LevelRenderer::OIT_TRANSMITTANCE_TARGET_COUNT
        )
        .build_snippet()
);
snippet!(
    OIT_DEPTH_BOUNDS_SNIPPET = RenderPipeline::builder(&[&OIT_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::GLOBALS)
        .with_bind_group_layout(BindGroupLayouts::PROJECTION)
        .with_color_target_state(ColorTargetState::new(
            Some(BlendFunction::MAX),
            GpuFormat::RGBA32_FLOAT,
            15
        ))
        .with_depth_stencil_state(DepthStencilState::new(
            CompareOp::GREATER_THAN_OR_EQUAL,
            false
        ))
        .with_shader_define("OIT_ALPHA_ONLY")
        .with_shader_define("OIT_DEPTH_BOUNDS")
        .build_snippet()
);
snippet!(
    OIT_TRANSMITTANCE_SNIPPET = RenderPipeline::builder(&[&OIT_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::GLOBALS)
        .with_bind_group_layout(BindGroupLayouts::PROJECTION)
        .with_bind_group_layout(BindGroupLayouts::DEPTH_BOUNDS_SAMPLER)
        .with_shader_define("OIT_ALPHA_ONLY")
        .with_shader_define("OIT_TRANSMITTANCE")
        .with_color_target_states(
            0,
            LevelRenderer::OIT_TRANSMITTANCE_TARGET_COUNT as usize - 1,
            || {
                ColorTargetState::new(Some(BlendFunction::ADDITIVE), GpuFormat::RGBA16_FLOAT, 15)
            }
        )
        .with_depth_stencil_state(DepthStencilState::new(
            CompareOp::GREATER_THAN_OR_EQUAL,
            false
        ))
        .build_snippet()
);
snippet!(
    OIT_ACCUMULATE_SNIPPET = RenderPipeline::builder(&[&OIT_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::PROJECTION)
        .with_bind_group_layout(BindGroupLayouts::OIT_COEFFS_DEPTH_BOUNDS_SAMPLER)
        .with_shader_define("OIT_ACCUMULATE")
        .with_color_target_state(ColorTargetState::new(
            Some(BlendFunction::ADDITIVE),
            GpuFormat::RGBA16_FLOAT,
            15
        ))
        .with_depth_stencil_state(DepthStencilState::new(
            CompareOp::GREATER_THAN_OR_EQUAL,
            false
        ))
        .build_snippet()
);
snippet!(
    GENERIC_BLOCKS_SNIPPET = RenderPipeline::builder(&[&GLOBALS_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::FOG)
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
        .with_vertex_binding(0, DefaultVertexFormat::BLOCK)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_depth_stencil_state(DepthStencilState::DEFAULT)
        .build_snippet()
);
snippet!(
    LIT_BLOCKS_SNIPPET = RenderPipeline::builder(&[&GENERIC_BLOCKS_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::SAMPLER2)
        .build_snippet()
);
snippet!(
    TERRAIN_SNIPPET = RenderPipeline::builder(&[&LIT_BLOCKS_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::PROJECTION)
        .with_bind_group_layout(BindGroupLayouts::CHUNK_SECTION)
        .with_bind_group_layout(BindGroupLayouts::TERRAIN_INFO)
        .with_vertex_shader("core/terrain")
        .with_fragment_shader("core/terrain")
        .build_snippet()
);
snippet!(
    MULTIDRAW_TERRAIN_SNIPPET = RenderPipeline::builder(&[&LIT_BLOCKS_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::PROJECTION)
        .with_bind_group_layout(BindGroupLayouts::TERRAIN_INFO)
        .with_vertex_binding(1, DefaultVertexFormat::CHUNK_DATA_INSTANCED)
        .with_vertex_shader("core/terrain")
        .with_fragment_shader("core/terrain")
        .with_shader_define("MULTIDRAW_TERRAIN")
        .build_snippet()
);
snippet!(
    BLOCK_SNIPPET = RenderPipeline::builder(&[&LIT_BLOCKS_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::PROJECTION)
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_vertex_shader("core/block")
        .with_fragment_shader("core/block")
        .build_snippet()
);
snippet!(
    WATER_MASK_SNIPPET = RenderPipeline::builder(&[&GLOBALS_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::PROJECTION)
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_vertex_shader("core/rendertype_water_mask")
        .with_fragment_shader("core/rendertype_water_mask")
        .with_vertex_binding(0, DefaultVertexFormat::POSITION)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_depth_stencil_state(DepthStencilState::DEFAULT)
        .build_snippet()
);
snippet!(
    LIGHTNING_SNIPPET = RenderPipeline::builder(&[&MATRICES_FOG_SNIPPET])
        .with_vertex_shader("core/rendertype_lightning")
        .with_fragment_shader("core/rendertype_lightning")
        .with_vertex_binding(0, DefaultVertexFormat::POSITION_COLOR)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_depth_stencil_state(DepthStencilState::DEFAULT)
        .build_snippet()
);
snippet!(
    DRAGON_RAYS_SNIPPET = RenderPipeline::builder(&[&MATRICES_FOG_SNIPPET])
        .with_vertex_shader("core/rendertype_lightning")
        .with_fragment_shader("core/rendertype_lightning")
        .with_vertex_binding(0, DefaultVertexFormat::POSITION_COLOR)
        .with_primitive_topology(PrimitiveTopology::TRIANGLES)
        .with_depth_stencil_state(DepthStencilState::DEFAULT)
        .build_snippet()
);
snippet!(
    ENTITY_NO_LIGHTMAP_SNIPPET = RenderPipeline::builder(&[&MATRICES_FOG_SNIPPET])
        .with_vertex_shader("core/entity")
        .with_fragment_shader("core/entity")
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
        .with_vertex_binding(0, DefaultVertexFormat::ENTITY)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_depth_stencil_state(DepthStencilState::DEFAULT)
        .build_snippet()
);
snippet!(
    ENTITY_SNIPPET = RenderPipeline::builder(&[&ENTITY_NO_LIGHTMAP_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::LIGHTING)
        .with_bind_group_layout(BindGroupLayouts::SAMPLER2)
        .build_snippet()
);
snippet!(
    OIT_ENTITY_SNIPPET = RenderPipeline::builder(&[])
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_bind_group_layout(BindGroupLayouts::FOG)
        .with_bind_group_layout(BindGroupLayouts::LIGHTING)
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
        .with_vertex_shader("core/entity")
        .with_fragment_shader("core/entity")
        .with_vertex_binding(0, DefaultVertexFormat::ENTITY)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_depth_stencil_state(DepthStencilState::new(
            CompareOp::GREATER_THAN_OR_EQUAL,
            false
        ))
        .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
        .build_snippet()
);
snippet!(
    ENTITY_EMISSIVE_SNIPPET = RenderPipeline::builder(&[&MATRICES_FOG_LIGHT_DIR_SNIPPET])
        .with_vertex_shader("core/entity")
        .with_fragment_shader("core/entity")
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
        .with_vertex_binding(0, DefaultVertexFormat::ENTITY)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_shader_define("EMISSIVE")
        .with_depth_stencil_state(DepthStencilState::DEFAULT)
        .build_snippet()
);
snippet!(
    EYES_SNIPPET = RenderPipeline::builder(&[])
        .with_shader_define("EMISSIVE")
        .with_shader_define("NO_OVERLAY")
        .with_shader_define("NO_CARDINAL_LIGHTING")
        .build_snippet()
);
snippet!(
    BEACON_BEAM_SNIPPET = RenderPipeline::builder(&[&MATRICES_FOG_SNIPPET])
        .with_vertex_shader("core/rendertype_beacon_beam")
        .with_fragment_shader("core/rendertype_beacon_beam")
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
        .with_vertex_binding(0, DefaultVertexFormat::BLOCK)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_depth_stencil_state(DepthStencilState::DEFAULT)
        .build_snippet()
);
snippet!(
    ITEM_SNIPPET = RenderPipeline::builder(&[&MATRICES_FOG_LIGHT_DIR_SNIPPET])
        .with_vertex_shader("core/item")
        .with_fragment_shader("core/item")
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0_SAMPLER1_SAMPLER2)
        .with_vertex_binding(0, DefaultVertexFormat::ENTITY)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_depth_stencil_state(DepthStencilState::DEFAULT)
        .build_snippet()
);
snippet!(
    OIT_ITEM_SNIPPET = RenderPipeline::builder(&[])
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_bind_group_layout(BindGroupLayouts::FOG)
        .with_bind_group_layout(BindGroupLayouts::LIGHTING)
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
        .with_vertex_shader("core/item")
        .with_fragment_shader("core/item")
        .with_vertex_binding(0, DefaultVertexFormat::ENTITY)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_depth_stencil_state(DepthStencilState::new(
            CompareOp::GREATER_THAN_OR_EQUAL,
            false
        ))
        .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
        .build_snippet()
);
snippet!(
    TEXT_SNIPPET = RenderPipeline::builder(&[&GLOBALS_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::PROJECTION)
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
        .with_vertex_shader("core/text")
        .with_fragment_shader("core/text")
        .with_depth_stencil_state(DepthStencilState::DEFAULT)
        .with_vertex_binding(0, DefaultVertexFormat::POSITION_TEX_COLOR)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .build_snippet()
);
snippet!(
    WORLD_TEXT_SNIPPET = RenderPipeline::builder(&[&TEXT_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::FOG)
        .with_bind_group_layout(BindGroupLayouts::SAMPLER2)
        .with_vertex_binding(0, DefaultVertexFormat::POSITION_TEX_LIGHTMAP_COLOR)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .build_snippet()
);
snippet!(
    END_PORTAL_SNIPPET = RenderPipeline::builder(&[&GLOBALS_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::PROJECTION)
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_bind_group_layout(BindGroupLayouts::FOG)
        .with_vertex_shader("core/rendertype_end_portal")
        .with_fragment_shader("core/rendertype_end_portal")
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0_SAMPLER1)
        .with_vertex_binding(0, DefaultVertexFormat::POSITION)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_color_target_state(ColorTargetState::DEFAULT)
        .with_depth_stencil_state(DepthStencilState::DEFAULT)
        .build_snippet()
);
snippet!(
    CLOUDS_SNIPPET = RenderPipeline::builder(&[&MATRICES_FOG_SNIPPET])
        .with_vertex_shader("core/clouds")
        .with_fragment_shader("core/clouds")
        .with_color_target_state(ColorTargetState::blending(BlendFunction::TRANSLUCENT))
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_bind_group_layout(BindGroupLayouts::CLOUD_INFO)
        .with_depth_stencil_state(DepthStencilState::DEFAULT)
        .build_snippet()
);
snippet!(
    OIT_CLOUDS_SNIPPET = RenderPipeline::builder(&[])
        .with_vertex_shader("core/clouds")
        .with_fragment_shader("core/clouds")
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_bind_group_layout(BindGroupLayouts::FOG)
        .with_bind_group_layout(BindGroupLayouts::CLOUD_INFO)
        .build_snippet()
);
snippet!(
    LINES_SNIPPET = RenderPipeline::builder(&[&MATRICES_FOG_SNIPPET])
        .with_vertex_shader("core/rendertype_lines")
        .with_fragment_shader("core/rendertype_lines")
        .with_cull(false)
        .with_vertex_binding(0, DefaultVertexFormat::POSITION_COLOR_NORMAL_LINE_WIDTH)
        .with_primitive_topology(PrimitiveTopology::LINES)
        .with_depth_stencil_state(DepthStencilState::DEFAULT)
        .build_snippet()
);
snippet!(
    OIT_LINES_SNIPPET = RenderPipeline::builder(&[])
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_bind_group_layout(BindGroupLayouts::FOG)
        .with_vertex_shader("core/rendertype_lines")
        .with_fragment_shader("core/rendertype_lines")
        .with_cull(false)
        .with_vertex_binding(0, DefaultVertexFormat::POSITION_COLOR_NORMAL_LINE_WIDTH)
        .with_primitive_topology(PrimitiveTopology::LINES)
        .build_snippet()
);
snippet!(
    DEBUG_FILLED_SNIPPET = RenderPipeline::builder(&[&GLOBALS_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::PROJECTION)
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_vertex_shader("core/position_color")
        .with_fragment_shader("core/position_color")
        .with_color_target_state(ColorTargetState::blending(BlendFunction::TRANSLUCENT))
        .with_vertex_binding(0, DefaultVertexFormat::POSITION_COLOR)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_depth_stencil_state(DepthStencilState::new(
            CompareOp::GREATER_THAN_OR_EQUAL,
            false
        ))
        .with_cull(false)
        .build_snippet()
);
snippet!(
    OIT_DEBUG_FILLED_SNIPPET = RenderPipeline::builder(&[])
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_vertex_shader("core/position_color")
        .with_fragment_shader("core/position_color")
        .with_vertex_binding(0, DefaultVertexFormat::POSITION_COLOR)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_cull(false)
        .build_snippet()
);
snippet!(
    DEBUG_POINTS_SNIPPET = RenderPipeline::builder(&[])
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_vertex_shader("core/debug_point")
        .with_fragment_shader("core/position_color")
        .with_vertex_binding(0, DefaultVertexFormat::POSITION_COLOR_LINE_WIDTH)
        .with_primitive_topology(PrimitiveTopology::POINTS)
        .with_cull(false)
        .build_snippet()
);
snippet!(
    PARTICLE_SNIPPET = RenderPipeline::builder(&[&MATRICES_FOG_SNIPPET])
        .with_vertex_shader("core/particle")
        .with_fragment_shader("core/particle")
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0_SAMPLER2)
        .with_vertex_binding(0, DefaultVertexFormat::PARTICLE)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_depth_stencil_state(DepthStencilState::DEFAULT)
        .build_snippet()
);
snippet!(
    OIT_PARTICLE_SNIPPET = RenderPipeline::builder(&[])
        .with_vertex_shader("core/particle")
        .with_fragment_shader("core/particle")
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0_SAMPLER2)
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_bind_group_layout(BindGroupLayouts::FOG)
        .with_vertex_binding(0, DefaultVertexFormat::PARTICLE)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_depth_stencil_state(DepthStencilState::new(
            CompareOp::GREATER_THAN_OR_EQUAL,
            false
        ))
        .build_snippet()
);
snippet!(
    WEATHER_SNIPPET = RenderPipeline::builder(&[&PARTICLE_SNIPPET])
        .with_color_target_state(ColorTargetState::blending(BlendFunction::TRANSLUCENT))
        .with_cull(false)
        .build_snippet()
);
snippet!(
    CRUMBLING_SNIPPET = RenderPipeline::builder(&[])
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_bind_group_layout(BindGroupLayouts::FOG)
        .with_vertex_shader("core/rendertype_crumbling")
        .with_fragment_shader("core/rendertype_crumbling")
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
        .with_vertex_binding(0, DefaultVertexFormat::BLOCK)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_depth_stencil_state(DepthStencilState::with_bias(
            CompareOp::GREATER_THAN_OR_EQUAL,
            false,
            1.0,
            10.0
        ))
        .build_snippet()
);
snippet!(
    GUI_SNIPPET = RenderPipeline::builder(&[&GLOBALS_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::PROJECTION)
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_vertex_shader("core/gui")
        .with_fragment_shader("core/gui")
        .with_color_target_state(ColorTargetState::blending(BlendFunction::TRANSLUCENT))
        .with_vertex_binding(0, DefaultVertexFormat::POSITION_COLOR)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .build_snippet()
);
snippet!(
    GUI_TEXTURED_SNIPPET = RenderPipeline::builder(&[&GLOBALS_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::PROJECTION)
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_vertex_shader("core/position_tex_color")
        .with_fragment_shader("core/position_tex_color")
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
        .with_color_target_state(ColorTargetState::blending(BlendFunction::TRANSLUCENT))
        .with_vertex_binding(0, DefaultVertexFormat::POSITION_TEX_COLOR)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .build_snippet()
);
snippet!(
    GUI_TEXT_SNIPPET = RenderPipeline::builder(&[&TEXT_SNIPPET])
        .with_shader_define("IS_GUI")
        .with_color_target_state(ColorTargetState::blending(BlendFunction::TRANSLUCENT))
        .with_depth_stencil_state_optional(None)
        .build_snippet()
);
snippet!(
    OUTLINE_SNIPPET = RenderPipeline::builder(&[&GLOBALS_SNIPPET])
        .with_bind_group_layout(BindGroupLayouts::PROJECTION)
        .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
        .with_vertex_shader("core/rendertype_outline")
        .with_fragment_shader("core/rendertype_outline")
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
        .with_vertex_binding(0, DefaultVertexFormat::POSITION_TEX_COLOR)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_color_target_state(ColorTargetState::DEFAULT)
        .build_snippet()
);
snippet!(
    POST_PROCESSING_SNIPPET = RenderPipeline::builder(&[&GLOBALS_SNIPPET])
        .with_primitive_topology(PrimitiveTopology::TRIANGLES)
        .build_snippet()
);
snippet!(
    ENERGY_SWIRL_SNIPPET = RenderPipeline::builder(&[&MATRICES_FOG_SNIPPET])
        .with_vertex_shader("core/entity")
        .with_fragment_shader("core/entity")
        .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
        .with_shader_define("EMISSIVE")
        .with_shader_define("NO_OVERLAY")
        .with_shader_define("NO_CARDINAL_LIGHTING")
        .with_shader_define("APPLY_TEXTURE_MATRIX")
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
        .with_cull(false)
        .with_vertex_binding(0, DefaultVertexFormat::ENTITY)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .build_snippet()
);
snippet!(
    ENTITY_SHADOW_SNIPPET = RenderPipeline::builder(&[&MATRICES_FOG_SNIPPET])
        .with_vertex_shader("core/rendertype_entity_shadow")
        .with_fragment_shader("core/rendertype_entity_shadow")
        .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
        .with_vertex_binding(0, DefaultVertexFormat::ENTITY)
        .with_primitive_topology(PrimitiveTopology::QUADS)
        .with_depth_stencil_state(DepthStencilState::new(
            CompareOp::GREATER_THAN_OR_EQUAL,
            false
        ))
        .build_snippet()
);
snippet!(
    GLINT_SNIPPET = RenderPipeline::builder(&[])
        .with_shader_define("GLINT")
        .with_bind_group_layout(BindGroupLayouts::GLINT_SAMPLER)
        .build_snippet()
);
snippet!(
    GLINT_SPECIAL_SNIPPET = RenderPipeline::builder(&[&GLINT_SNIPPET])
        .with_shader_define("GLINT_SPECIAL")
        .build_snippet()
);
snippet!(
    ANIMATE_SPRITE_SNIPPET = RenderPipeline::builder(&[&GLOBALS_SNIPPET])
        .with_vertex_shader("core/animate_sprite")
        .with_bind_group_layout(BindGroupLayouts::SPRITE_ANIMATION_INFO)
        .with_primitive_topology(PrimitiveTopology::TRIANGLES)
        .with_color_target_state(ColorTargetState::DEFAULT)
        .build_snippet()
);

/// The pipelines `register` and `registerOptional` put in their maps, in that order.
#[derive(Default)]
struct Registry {
    pipelines_by_location: HashMap<Identifier, Arc<RenderPipeline>>,
    optional_pipelines_by_location: HashMap<Identifier, Arc<RenderPipeline>>,
}

impl Registry {
    fn register(&mut self, pipeline: RenderPipeline) -> Arc<RenderPipeline> {
        let pipeline = Arc::new(pipeline);
        self.pipelines_by_location
            .put(pipeline.get_location().clone(), pipeline.clone());
        pipeline
    }

    fn register_set(&mut self, set: OitPipelineSet) -> OitPipelineSet {
        for pipeline in [
            &set.depth_bounds_pipeline,
            &set.transmittance_pipeline,
            &set.accumulate_pipeline,
        ] {
            self.pipelines_by_location
                .put(pipeline.get_location().clone(), pipeline.clone());
        }
        set
    }

    fn register_optional(&mut self, pipeline: RenderPipeline) -> Arc<RenderPipeline> {
        let pipeline = Arc::new(pipeline);
        self.optional_pipelines_by_location
            .put(pipeline.get_location().clone(), pipeline.clone());
        pipeline
    }
}

/// `new ColorTargetState(BlendFunction)`.
const fn blending(blend_function: BlendFunction) -> ColorTargetState {
    ColorTargetState::blending(blend_function)
}

/// `new DepthStencilState(CompareOp.GREATER_THAN_OR_EQUAL, false)`, testing without writing.
const NO_DEPTH_WRITE: DepthStencilState =
    DepthStencilState::new(CompareOp::GREATER_THAN_OR_EQUAL, false);

macro_rules! render_pipelines {
    ($registry:ident; $($name:ident: $type:ty = $value:expr;)*) => {
        /// `RenderPipelines`' public pipelines and OIT sets.
        #[allow(non_snake_case)]
        pub struct RenderPipelines {
            $(pub $name: $type,)*
            pipelines_by_location: HashMap<Identifier, Arc<RenderPipeline>>,
            optional_pipelines_by_location: HashMap<Identifier, Arc<RenderPipeline>>,
        }

        impl RenderPipelines {
            #[allow(non_snake_case)]
            fn new() -> RenderPipelines {
                let mut $registry = Registry::default();
                $(let $name: $type = $value;)*
                RenderPipelines {
                    $($name,)*
                    pipelines_by_location: $registry.pipelines_by_location,
                    optional_pipelines_by_location: $registry.optional_pipelines_by_location,
                }
            }

            #[cfg(test)]
            fn fields(&self) -> Vec<(&'static str, serde_json::Value)> {
                vec![$((stringify!($name), tests::EncodeField::encode_field(&self.$name)),)*]
            }
        }
    };
}

render_pipelines! {
    registry;
    SOLID_BLOCK: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&BLOCK_SNIPPET])
            .with_location("pipeline/solid_block")
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    SOLID_TERRAIN: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&TERRAIN_SNIPPET])
            .with_location("pipeline/solid_terrain")
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    SOLID_TERRAIN_MULTIDRAW: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&MULTIDRAW_TERRAIN_SNIPPET])
            .with_location("pipeline/solid_terrain_multidraw")
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    WIREFRAME: Arc<RenderPipeline> = registry.register_optional(
        RenderPipeline::builder(&[&TERRAIN_SNIPPET])
            .with_location("pipeline/wireframe")
            .with_polygon_mode(PolygonMode::WIREFRAME)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    WIREFRAME_MULTIDRAW: Arc<RenderPipeline> = registry.register_optional(
        RenderPipeline::builder(&[&MULTIDRAW_TERRAIN_SNIPPET])
            .with_location("pipeline/wireframe_multidraw")
            .with_polygon_mode(PolygonMode::WIREFRAME)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    CUTOUT_BLOCK: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&BLOCK_SNIPPET])
            .with_location("pipeline/cutout_block")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_CUTOUT_TERRAIN)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    CUTOUT_TERRAIN: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&TERRAIN_SNIPPET])
            .with_location("pipeline/cutout_terrain")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_CUTOUT_TERRAIN)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    CUTOUT_TERRAIN_MULTIDRAW: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&MULTIDRAW_TERRAIN_SNIPPET])
            .with_location("pipeline/cutout_terrain_multidraw")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_CUTOUT_TERRAIN)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    TRANSLUCENT_TERRAIN: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&TERRAIN_SNIPPET])
            .with_location("pipeline/translucent_terrain")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .build(),
    );
    TRANSLUCENT_TERRAIN_MULTIDRAW: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&MULTIDRAW_TERRAIN_SNIPPET])
            .with_location("pipeline/translucent_terrain_multidraw")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .build(),
    );
    OIT_TERRAIN: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder(
            "terrain",
            RenderPipeline::builder(&[&GENERIC_BLOCKS_SNIPPET])
                .with_vertex_shader("core/terrain")
                .with_fragment_shader("core/terrain")
                .with_bind_group_layout(BindGroupLayouts::CHUNK_SECTION)
                .with_bind_group_layout(BindGroupLayouts::TERRAIN_INFO)
                .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT),
        )
        .with_accumulate_modifier(|accumulate| {
            accumulate.with_bind_group_layout(BindGroupLayouts::SAMPLER2);
        })
        .build(),
    );
    OIT_TERRAIN_MULTIDRAW: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder(
            "terrain_multidraw",
            RenderPipeline::builder(&[&GENERIC_BLOCKS_SNIPPET])
                .with_vertex_shader("core/terrain")
                .with_fragment_shader("core/terrain")
                .with_bind_group_layout(BindGroupLayouts::TERRAIN_INFO)
                .with_vertex_binding(1, DefaultVertexFormat::CHUNK_DATA_INSTANCED)
                .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
                .with_shader_define("MULTIDRAW_TERRAIN"),
        )
        .with_accumulate_modifier(|accumulate| {
            accumulate.with_bind_group_layout(BindGroupLayouts::SAMPLER2);
        })
        .build(),
    );
    TRANSLUCENT_BLOCK: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&BLOCK_SNIPPET])
            .with_location("pipeline/translucent_block")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_depth_stencil_state(DepthStencilState::DEFAULT)
            .build(),
    );
    OIT_TRANSLUCENT_BLOCK: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder(
            "translucent_block",
            RenderPipeline::builder(&[&GENERIC_BLOCKS_SNIPPET])
                .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
                .with_vertex_shader("core/block")
                .with_fragment_shader("core/block")
                .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT),
        )
        .with_accumulate_modifier(|accumulate| {
            accumulate.with_bind_group_layout(BindGroupLayouts::SAMPLER2);
        })
        .build(),
    );
    ARMOR_CUTOUT_NO_CULL: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET])
            .with_location("pipeline/armor_cutout_no_cull")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_shader_define("NO_OVERLAY")
            .with_shader_define("PER_FACE_LIGHTING")
            .with_cull(false)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    ARMOR_DECAL_CUTOUT_NO_CULL: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET])
            .with_location("pipeline/armor_decal_cutout_no_cull")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_shader_define("NO_OVERLAY")
            .with_shader_define("PER_FACE_LIGHTING")
            .with_cull(false)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .with_depth_stencil_state(DepthStencilState::new(CompareOp::EQUAL, false))
            .build(),
    );
    WOLF_ARMOR_CRACKS: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET])
            .with_location("pipeline/armor_translucent")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_shader_define("NO_OVERLAY")
            .with_shader_define("PER_FACE_LIGHTING")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_cull(false)
            .build(),
    );
    ENTITY_SOLID: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET])
            .with_location("pipeline/entity_solid")
            .with_bind_group_layout(BindGroupLayouts::SAMPLER1)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    ENTITY_SOLID_Z_OFFSET_FORWARD: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET])
            .with_location("pipeline/entity_solid_offset_forward")
            .with_bind_group_layout(BindGroupLayouts::SAMPLER1)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    ENTITY_CUTOUT_CULL: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET])
            .with_location("pipeline/entity_cutout_cull")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_bind_group_layout(BindGroupLayouts::SAMPLER1)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    ENTITY_CUTOUT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET])
            .with_location("pipeline/entity_cutout")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_shader_define("PER_FACE_LIGHTING")
            .with_bind_group_layout(BindGroupLayouts::SAMPLER1)
            .with_cull(false)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    ENTITY_CUTOUT_Z_OFFSET: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET])
            .with_location("pipeline/entity_cutout_z_offset")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_shader_define("PER_FACE_LIGHTING")
            .with_bind_group_layout(BindGroupLayouts::SAMPLER1)
            .with_cull(false)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    ENTITY_CUTOUT_DISSOLVE: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET])
            .with_location("pipeline/entity_cutout_dissolve")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_shader_define("PER_FACE_LIGHTING")
            .with_shader_define("DISSOLVE")
            .with_bind_group_layout(BindGroupLayouts::SAMPLER1)
            .with_bind_group_layout(BindGroupLayouts::DISSOLVE_MASK_SAMPLER)
            .with_cull(false)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    ENTITY_TRANSLUCENT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET])
            .with_location("pipeline/entity_translucent")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_shader_define("PER_FACE_LIGHTING")
            .with_bind_group_layout(BindGroupLayouts::SAMPLER1)
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_cull(false)
            .build(),
    );
    OIT_ENTITY: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("entity", RenderPipeline::builder(&[&OIT_ENTITY_SNIPPET]).with_cull(false))
            .with_accumulate_modifier(|accumulate| {
                accumulate
                    .with_shader_define("PER_FACE_LIGHTING")
                    .with_bind_group_layout(BindGroupLayouts::SAMPLER1)
                    .with_bind_group_layout(BindGroupLayouts::SAMPLER2);
            })
            .build(),
    );
    ENTITY_TRANSLUCENT_EMISSIVE: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_EMISSIVE_SNIPPET])
            .with_location("pipeline/entity_translucent_emissive")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_shader_define("PER_FACE_LIGHTING")
            .with_bind_group_layout(BindGroupLayouts::SAMPLER1)
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_cull(false)
            .with_depth_stencil_state(NO_DEPTH_WRITE)
            .build(),
    );
    OIT_ENTITY_EMISSIVE: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("entity_emissive", RenderPipeline::builder(&[&OIT_ENTITY_SNIPPET]).with_cull(false))
            .with_accumulate_modifier(|accumulate| {
                accumulate
                    .with_shader_define("PER_FACE_LIGHTING")
                    .with_shader_define("EMISSIVE")
                    .with_bind_group_layout(BindGroupLayouts::SAMPLER1);
            })
            .build(),
    );
    ENTITY_TRANSLUCENT_CULL: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET])
            .with_location("pipeline/entity_translucent_cull")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_bind_group_layout(BindGroupLayouts::SAMPLER1)
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .build(),
    );
    OIT_ENTITY_CULL: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("entity_cull", &RenderPipeline::builder(&[&OIT_ENTITY_SNIPPET]))
            .with_accumulate_modifier(|accumulate| {
                accumulate
                    .with_bind_group_layout(BindGroupLayouts::SAMPLER1)
                    .with_bind_group_layout(BindGroupLayouts::SAMPLER2);
            })
            .build(),
    );
    END_CRYSTAL_BEAM: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET])
            .with_location("pipeline/end_crystal_beam")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_shader_define("NO_OVERLAY")
            .with_cull(false)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    BANNER_PATTERN: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET])
            .with_location("pipeline/banner_pattern")
            .with_shader_define("NO_OVERLAY")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_depth_stencil_state(NO_DEPTH_WRITE)
            .build(),
    );
    BREEZE_WIND: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET])
            .with_location("pipeline/breeze_wind")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_shader_define("APPLY_TEXTURE_MATRIX")
            .with_shader_define("NO_OVERLAY")
            .with_shader_define("NO_CARDINAL_LIGHTING")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_cull(false)
            .build(),
    );
    OIT_BREEZE_WIND: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder(
            "breeze_wind",
            RenderPipeline::builder(&[&OIT_ENTITY_SNIPPET])
                .with_shader_define("APPLY_TEXTURE_MATRIX")
                .with_shader_define("NO_OVERLAY")
                .with_shader_define("NO_CARDINAL_LIGHTING")
                .with_cull(false),
        )
        .with_accumulate_modifier(|accumulate| {
            accumulate.with_bind_group_layout(BindGroupLayouts::SAMPLER2);
        })
        .build(),
    );
    ENERGY_SWIRL: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENERGY_SWIRL_SNIPPET])
            .with_location("pipeline/energy_swirl")
            .with_color_target_state(blending(BlendFunction::ADDITIVE))
            .with_depth_stencil_state(DepthStencilState::DEFAULT)
            .build(),
    );
    OIT_ENERGY_SWIRL: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder(
            "energy_swirl",
            RenderPipeline::builder(&[&ENERGY_SWIRL_SNIPPET]).with_shader_define("OIT_ADDITIVE"),
        )
        .build(),
    );
    EYES: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_NO_LIGHTMAP_SNIPPET, &EYES_SNIPPET])
            .with_location("pipeline/eyes")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_depth_stencil_state(NO_DEPTH_WRITE)
            .build(),
    );
    OIT_EYES: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("eyes", &RenderPipeline::builder(&[&OIT_ENTITY_SNIPPET, &EYES_SNIPPET])).build(),
    );
    ENTITY_SHADOW: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SHADOW_SNIPPET])
            .with_location("pipeline/entity_shadow")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .build(),
    );
    OIT_ENTITY_SHADOW: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("entity_shadow", &RenderPipeline::builder(&[&ENTITY_SHADOW_SNIPPET])).build(),
    );
    ARMOR_CUTOUT_NO_CULL_GLINT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET, &GLINT_SNIPPET])
            .with_location("pipeline/armor_cutout_no_cull_glint")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_shader_define("NO_OVERLAY")
            .with_shader_define("PER_FACE_LIGHTING")
            .with_cull(false)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    ENTITY_SOLID_GLINT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ENTITY_SNIPPET, &GLINT_SNIPPET])
            .with_location("pipeline/entity_solid_glint")
            .with_bind_group_layout(BindGroupLayouts::SAMPLER1)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    ITEM_CUTOUT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ITEM_SNIPPET])
            .with_location("pipeline/item_cutout")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    ITEM_CUTOUT_GLINT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ITEM_SNIPPET, &GLINT_SNIPPET])
            .with_location("pipeline/item_cutout")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    ITEM_CUTOUT_GLINT_SPECIAL: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ITEM_SNIPPET, &GLINT_SPECIAL_SNIPPET])
            .with_location("pipeline/item_cutout")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_vertex_binding(0, DefaultVertexFormat::ENTITY_GLINT_SPECIAL)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    ITEM_TRANSLUCENT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ITEM_SNIPPET])
            .with_location("pipeline/item_translucent")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .build(),
    );
    OIT_ITEM: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("item", &RenderPipeline::builder(&[&OIT_ITEM_SNIPPET]))
            .with_accumulate_modifier(|accumulate| {
                accumulate
                    .with_bind_group_layout(BindGroupLayouts::SAMPLER1)
                    .with_bind_group_layout(BindGroupLayouts::SAMPLER2);
            })
            .build(),
    );
    ITEM_TRANSLUCENT_GLINT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ITEM_SNIPPET, &GLINT_SNIPPET])
            .with_location("pipeline/item_translucent_glint")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .build(),
    );
    ITEM_TRANSLUCENT_GLINT_SPECIAL: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ITEM_SNIPPET, &GLINT_SPECIAL_SNIPPET])
            .with_location("pipeline/item_translucent_glint")
            .with_shader_define_f32("ALPHA_CUTOUT", ALPHA_CUTOUT_THRESHOLD_DEFAULT)
            .with_vertex_binding(0, DefaultVertexFormat::ENTITY_GLINT_SPECIAL)
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .build(),
    );
    OIT_ITEM_GLINT: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("item_glint", &RenderPipeline::builder(&[&OIT_ITEM_SNIPPET]))
            .with_accumulate_modifier(|accumulate| {
                accumulate
                    .with_snippet(&GLINT_SNIPPET)
                    .with_bind_group_layout(BindGroupLayouts::GLOBALS)
                    .with_bind_group_layout(BindGroupLayouts::SAMPLER1)
                    .with_bind_group_layout(BindGroupLayouts::SAMPLER2);
            })
            .build(),
    );
    OIT_ITEM_GLINT_SPECIAL: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder(
            "item_glint_special",
            RenderPipeline::builder(&[&OIT_ITEM_SNIPPET]).with_vertex_binding(0, DefaultVertexFormat::ENTITY_GLINT_SPECIAL),
        )
        .with_accumulate_modifier(|accumulate| {
            accumulate
                .with_snippet(&GLINT_SPECIAL_SNIPPET)
                .with_bind_group_layout(BindGroupLayouts::GLOBALS)
                .with_bind_group_layout(BindGroupLayouts::SAMPLER1)
                .with_bind_group_layout(BindGroupLayouts::SAMPLER2);
        })
        .build(),
    );
    BEACON_BEAM_OPAQUE: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&BEACON_BEAM_SNIPPET])
            .with_location("pipeline/beacon_beam_opaque")
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    BEACON_BEAM_TRANSLUCENT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&BEACON_BEAM_SNIPPET])
            .with_location("pipeline/beacon_beam_translucent")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_depth_stencil_state(NO_DEPTH_WRITE)
            .build(),
    );
    OIT_BEACON_BEAM: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("beacon_beam", &RenderPipeline::builder(&[&BEACON_BEAM_SNIPPET])).build(),
    );
    LEASH: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&MATRICES_FOG_SNIPPET])
            .with_location("pipeline/leash")
            .with_vertex_shader("core/rendertype_leash")
            .with_fragment_shader("core/rendertype_leash")
            .with_bind_group_layout(BindGroupLayouts::SAMPLER2)
            .with_cull(false)
            .with_vertex_binding(0, DefaultVertexFormat::POSITION_COLOR_LIGHTMAP)
            .with_primitive_topology(PrimitiveTopology::TRIANGLE_STRIP)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .with_depth_stencil_state(DepthStencilState::DEFAULT)
            .build(),
    );
    WATER_MASK: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&WATER_MASK_SNIPPET])
            .with_location("pipeline/water_mask")
            .with_color_target_state(ColorTargetState::new(Some(BlendFunction::TRANSLUCENT), GpuFormat::RGBA8_UNORM, 0))
            .build(),
    );
    OIT_WATER_MASK: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&WATER_MASK_SNIPPET])
            .with_location("pipeline/oit_water_mask")
            .build(),
    );
    GLINT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GLOBALS_SNIPPET])
            .with_bind_group_layout(BindGroupLayouts::PROJECTION)
            .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
            .with_bind_group_layout(BindGroupLayouts::FOG)
            .with_location("pipeline/glint")
            .with_vertex_shader("core/glint")
            .with_fragment_shader("core/glint")
            .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
            .with_cull(false)
            .with_color_target_state(blending(BlendFunction::GLINT))
            .with_vertex_binding(0, DefaultVertexFormat::POSITION_TEX)
            .with_primitive_topology(PrimitiveTopology::QUADS)
            .with_depth_stencil_state(DepthStencilState::new(CompareOp::EQUAL, false))
            .build(),
    );
    CRUMBLING: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&CRUMBLING_SNIPPET, &GLOBALS_SNIPPET])
            .with_bind_group_layout(BindGroupLayouts::PROJECTION)
            .with_location("pipeline/crumbling")
            .with_color_target_state(blending(BlendFunction::separate(
                BlendFactor::DST_COLOR,
                BlendFactor::SRC_COLOR,
                BlendFactor::ONE,
                BlendFactor::ZERO,
            )))
            .build(),
    );
    OIT_CRUMBLING: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("crumbling", &RenderPipeline::builder(&[&CRUMBLING_SNIPPET])).build(),
    );
    TEXT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&WORLD_TEXT_SNIPPET])
            .with_location("pipeline/text")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .build(),
    );
    OIT_TEXT: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("text", &RenderPipeline::builder(&[&WORLD_TEXT_SNIPPET])).build(),
    );
    GUI_TEXT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GUI_TEXT_SNIPPET])
            .with_location("pipeline/gui_text")
            .build(),
    );
    TEXT_GRAYSCALE: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&WORLD_TEXT_SNIPPET])
            .with_location("pipeline/text_grayscale")
            .with_shader_define("IS_GRAYSCALE")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .build(),
    );
    OIT_TEXT_GRAYSCALE: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder(
            "text_grayscale",
            RenderPipeline::builder(&[&WORLD_TEXT_SNIPPET]).with_shader_define("IS_GRAYSCALE"),
        )
        .build(),
    );
    GUI_TEXT_GRAYSCALE: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GUI_TEXT_SNIPPET])
            .with_location("pipeline/gui_text_grayscale")
            .with_shader_define("IS_GRAYSCALE")
            .build(),
    );
    TEXT_POLYGON_OFFSET: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&WORLD_TEXT_SNIPPET])
            .with_location("pipeline/text_polygon_offset")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_depth_stencil_state(DepthStencilState::with_bias(CompareOp::GREATER_THAN_OR_EQUAL, true, 1.0, 10.0))
            .build(),
    );
    OIT_TEXT_POLYGON_OFFSET: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("text_polygon_offset", &RenderPipeline::builder(&[&WORLD_TEXT_SNIPPET])).build(),
    );
    TEXT_GRAYSCALE_POLYGON_OFFSET: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&WORLD_TEXT_SNIPPET])
            .with_location("pipeline/text_grayscale_polygon_offset")
            .with_shader_define("IS_GRAYSCALE")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_depth_stencil_state(DepthStencilState::with_bias(CompareOp::GREATER_THAN_OR_EQUAL, true, 1.0, 10.0))
            .build(),
    );
    OIT_TEXT_GRAYSCALE_POLYGON_OFFSET: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder(
            "text_grayscale_polygon_offset",
            RenderPipeline::builder(&[&WORLD_TEXT_SNIPPET]).with_shader_define("IS_GRAYSCALE"),
        )
        .build(),
    );
    TEXT_SEE_THROUGH: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&TEXT_SNIPPET])
            .with_location("pipeline/text_see_through")
            .with_shader_define("IS_SEE_THROUGH")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_depth_stencil_state_optional(None)
            .build(),
    );
    TEXT_GRAYSCALE_SEE_THROUGH: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&TEXT_SNIPPET])
            .with_location("pipeline/text_grayscale_see_through")
            .with_shader_define("IS_GRAYSCALE")
            .with_shader_define("IS_SEE_THROUGH")
            .with_color_target_state(ColorTargetState::DEFAULT)
            .with_depth_stencil_state_optional(None)
            .build(),
    );
    LIGHTNING: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&LIGHTNING_SNIPPET])
            .with_location("pipeline/lightning")
            .with_color_target_state(blending(BlendFunction::LIGHTNING))
            .build(),
    );
    OIT_LIGHTNING: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder(
            "lightning",
            RenderPipeline::builder(&[&LIGHTNING_SNIPPET]).with_shader_define("OIT_ADDITIVE"),
        )
        .build(),
    );
    DRAGON_RAYS: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&DRAGON_RAYS_SNIPPET])
            .with_location("pipeline/dragon_rays")
            .with_color_target_state(blending(BlendFunction::LIGHTNING))
            .build(),
    );
    OIT_DRAGON_RAYS: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder(
            "dragon_rays",
            RenderPipeline::builder(&[&DRAGON_RAYS_SNIPPET]).with_shader_define("OIT_ADDITIVE"),
        )
        .build(),
    );
    END_PORTAL: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&END_PORTAL_SNIPPET])
            .with_location("pipeline/end_portal")
            .with_shader_define_i32("PORTAL_LAYERS", 15)
            .build(),
    );
    END_GATEWAY: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&END_PORTAL_SNIPPET])
            .with_location("pipeline/end_gateway")
            .with_shader_define_i32("PORTAL_LAYERS", 16)
            .build(),
    );
    FLAT_CLOUDS: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&CLOUDS_SNIPPET])
            .with_location("pipeline/flat_clouds")
            .with_cull(false)
            .build(),
    );
    CLOUDS: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&CLOUDS_SNIPPET])
            .with_location("pipeline/clouds")
            .build(),
    );
    OIT_CLOUDS: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("clouds", &RenderPipeline::builder(&[&OIT_CLOUDS_SNIPPET]))
            .with_depth_bounds_modifier(|depth_bounds| {
                depth_bounds.with_depth_stencil_state(DepthStencilState::DEFAULT);
            })
            .build(),
    );
    OIT_FLAT_CLOUDS: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("flat_clouds", RenderPipeline::builder(&[&OIT_CLOUDS_SNIPPET]).with_cull(false)).build(),
    );
    LINES: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&LINES_SNIPPET])
            .with_location("pipeline/lines")
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    LINES_TRANSLUCENT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&LINES_SNIPPET])
            .with_location("pipeline/lines_translucent")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .build(),
    );
    LINES_TRANSLUCENT_NO_DEPTH_WRITE: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&LINES_SNIPPET])
            .with_location("pipeline/lines_translucent_no_depth_write")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_depth_stencil_state(NO_DEPTH_WRITE)
            .build(),
    );
    OIT_LINES_TRANSLUCENT: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("lines_translucent", &RenderPipeline::builder(&[&OIT_LINES_SNIPPET])).build(),
    );
    LINES_DEPTH_BIAS: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&LINES_SNIPPET])
            .with_location("pipeline/lines_depth_bias")
            .with_color_target_state(ColorTargetState::DEFAULT)
            .with_depth_stencil_state(DepthStencilState::with_bias(CompareOp::GREATER_THAN_OR_EQUAL, true, 1.0, 1.0))
            .build(),
    );
    SECONDARY_BLOCK_OUTLINE: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&LINES_SNIPPET])
            .with_location("pipeline/secondary_block_outline")
            .with_color_target_state(ColorTargetState::DEFAULT)
            .with_depth_stencil_state(DepthStencilState::DEFAULT)
            .build(),
    );
    DEBUG_POINTS: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&DEBUG_POINTS_SNIPPET])
            .with_location("pipeline/debug_points")
            .with_bind_group_layout(BindGroupLayouts::GLOBALS)
            .with_bind_group_layout(BindGroupLayouts::PROJECTION)
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_depth_stencil_state(DepthStencilState::DEFAULT)
            .build(),
    );
    OIT_DEBUG_POINTS: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("debug_points", &RenderPipeline::builder(&[&DEBUG_POINTS_SNIPPET])).build(),
    );
    DEBUG_FILLED_BOX: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&DEBUG_FILLED_SNIPPET])
            .with_location("pipeline/debug_filled_box")
            .build(),
    );
    OIT_DEBUG_FILLED_BOX: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("debug_filled_box", &RenderPipeline::builder(&[&OIT_DEBUG_FILLED_SNIPPET])).build(),
    );
    DEBUG_QUADS: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&DEBUG_FILLED_SNIPPET])
            .with_location("pipeline/debug_quads")
            .with_cull(false)
            .build(),
    );
    OIT_DEBUG_QUADS: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("debug_quads", &RenderPipeline::builder(&[&OIT_DEBUG_FILLED_SNIPPET])).build(),
    );
    DEBUG_TRIANGLE_FAN: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&DEBUG_FILLED_SNIPPET])
            .with_location("pipeline/debug_triangle_fan")
            .with_cull(false)
            .with_vertex_binding(0, DefaultVertexFormat::POSITION_COLOR)
            .with_primitive_topology(PrimitiveTopology::TRIANGLE_FAN)
            .build(),
    );
    OIT_DEBUG_TRIANGLE_FAN: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder(
            "debug_triangle_fan",
            RenderPipeline::builder(&[&OIT_DEBUG_FILLED_SNIPPET])
                .with_vertex_binding(0, DefaultVertexFormat::POSITION_COLOR)
                .with_primitive_topology(PrimitiveTopology::TRIANGLE_FAN),
        )
        .build(),
    );
    WORLD_BORDER: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GLOBALS_SNIPPET])
            .with_bind_group_layout(BindGroupLayouts::PROJECTION)
            .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
            .with_location("pipeline/world_border")
            .with_vertex_shader("core/world_border")
            .with_fragment_shader("core/world_border")
            .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
            .with_color_target_state(blending(BlendFunction::OVERLAY))
            .with_cull(false)
            .with_vertex_binding(0, DefaultVertexFormat::POSITION_TEX)
            .with_primitive_topology(PrimitiveTopology::QUADS)
            .with_depth_stencil_state(DepthStencilState::with_bias(CompareOp::GREATER_THAN_OR_EQUAL, true, 3.0, 3.0))
            .build(),
    );
    OIT_WORLD_BORDER: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder(
            "world_border",
            RenderPipeline::builder(&[&GLOBALS_SNIPPET])
                .with_vertex_shader("core/world_border")
                .with_fragment_shader("core/world_border")
                .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
                .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
                .with_cull(false)
                .with_primitive_topology(PrimitiveTopology::QUADS)
                .with_vertex_binding(0, DefaultVertexFormat::POSITION_TEX)
                .with_depth_stencil_state(DepthStencilState::with_bias(CompareOp::GREATER_THAN_OR_EQUAL, false, 3.0, 3.0))
                .with_shader_define("OIT_ADDITIVE"),
        )
        .build(),
    );
    OPAQUE_PARTICLE: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&PARTICLE_SNIPPET])
            .with_location("pipeline/opaque_particle")
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    TRANSLUCENT_PARTICLE: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&PARTICLE_SNIPPET])
            .with_location("pipeline/translucent_particle")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .build(),
    );
    OIT_PARTICLE: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("particle", &RenderPipeline::builder(&[&OIT_PARTICLE_SNIPPET])).build(),
    );
    WEATHER: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&WEATHER_SNIPPET])
            .with_location("pipeline/weather")
            .with_depth_stencil_state(NO_DEPTH_WRITE)
            .build(),
    );
    OIT_WEATHER: OitPipelineSet = registry.register_set(
        OitPipelineSet::builder("weather", RenderPipeline::builder(&[&OIT_PARTICLE_SNIPPET]).with_cull(false)).build(),
    );
    SKY: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&MATRICES_FOG_SNIPPET])
            .with_location("pipeline/sky")
            .with_vertex_shader("core/sky")
            .with_fragment_shader("core/sky")
            .with_vertex_binding(0, DefaultVertexFormat::POSITION)
            .with_primitive_topology(PrimitiveTopology::TRIANGLE_FAN)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    END_SKY: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GLOBALS_SNIPPET])
            .with_bind_group_layout(BindGroupLayouts::PROJECTION)
            .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
            .with_location("pipeline/end_sky")
            .with_vertex_shader("core/position_tex_color")
            .with_fragment_shader("core/position_tex_color")
            .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_vertex_binding(0, DefaultVertexFormat::POSITION_TEX_COLOR)
            .with_primitive_topology(PrimitiveTopology::QUADS)
            .build(),
    );
    SUNRISE_SUNSET: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GLOBALS_SNIPPET])
            .with_bind_group_layout(BindGroupLayouts::PROJECTION)
            .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
            .with_location("pipeline/sunrise_sunset")
            .with_vertex_shader("core/position_color")
            .with_fragment_shader("core/position_color")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT))
            .with_vertex_binding(0, DefaultVertexFormat::POSITION_COLOR)
            .with_primitive_topology(PrimitiveTopology::TRIANGLE_FAN)
            .build(),
    );
    STARS: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GLOBALS_SNIPPET])
            .with_bind_group_layout(BindGroupLayouts::PROJECTION)
            .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
            .with_location("pipeline/stars")
            .with_vertex_shader("core/stars")
            .with_fragment_shader("core/stars")
            .with_color_target_state(blending(BlendFunction::OVERLAY))
            .with_vertex_binding(0, DefaultVertexFormat::POSITION)
            .with_primitive_topology(PrimitiveTopology::QUADS)
            .build(),
    );
    CELESTIAL: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GLOBALS_SNIPPET])
            .with_bind_group_layout(BindGroupLayouts::PROJECTION)
            .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
            .with_location("pipeline/celestial")
            .with_vertex_shader("core/position_tex")
            .with_fragment_shader("core/position_tex")
            .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
            .with_color_target_state(blending(BlendFunction::OVERLAY))
            .with_vertex_binding(0, DefaultVertexFormat::POSITION_TEX)
            .with_primitive_topology(PrimitiveTopology::QUADS)
            .build(),
    );
    GUI: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GUI_SNIPPET])
            .with_location("pipeline/gui")
            .build(),
    );
    GUI_INVERT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GUI_SNIPPET])
            .with_location("pipeline/gui_invert")
            .with_color_target_state(blending(BlendFunction::INVERT))
            .build(),
    );
    GUI_TEXT_HIGHLIGHT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GUI_SNIPPET])
            .with_location("pipeline/gui_text_highlight")
            .with_color_target_state(blending(BlendFunction::ADDITIVE))
            .build(),
    );
    GUI_TEXTURED: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GUI_TEXTURED_SNIPPET])
            .with_location("pipeline/gui_textured")
            .build(),
    );
    GUI_TEXTURED_PREMULTIPLIED_ALPHA: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GUI_TEXTURED_SNIPPET])
            .with_location("pipeline/gui_textured_premultiplied_alpha")
            .with_color_target_state(blending(BlendFunction::TRANSLUCENT_PREMULTIPLIED_ALPHA))
            .build(),
    );
    BLOCK_SCREEN_EFFECT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GUI_TEXTURED_SNIPPET])
            .with_location("pipeline/block_screen_effect")
            .build(),
    );
    FIRE_SCREEN_EFFECT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GUI_TEXTURED_SNIPPET])
            .with_location("pipeline/fire_screen_effect")
            .build(),
    );
    GUI_OPAQUE_TEXTURED_BACKGROUND: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GUI_TEXTURED_SNIPPET])
            .with_location("pipeline/gui_opaque_textured_background")
            .with_color_target_state(ColorTargetState::new(None, GpuFormat::RGBA8_UNORM, 15))
            .build(),
    );
    GUI_NAUSEA_OVERLAY: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GUI_TEXTURED_SNIPPET])
            .with_location("pipeline/gui_nausea_overlay")
            .with_color_target_state(blending(BlendFunction::ADDITIVE))
            .build(),
    );
    VIGNETTE: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GUI_TEXTURED_SNIPPET])
            .with_location("pipeline/vignette")
            .with_color_target_state(blending(BlendFunction::separate(
                BlendFactor::ZERO,
                BlendFactor::ONE_MINUS_SRC_COLOR,
                BlendFactor::ZERO,
                BlendFactor::ONE,
            )))
            .build(),
    );
    CROSSHAIR: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GUI_TEXTURED_SNIPPET])
            .with_location("pipeline/crosshair")
            .with_color_target_state(blending(BlendFunction::INVERT))
            .build(),
    );
    MOJANG_LOGO: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GUI_TEXTURED_SNIPPET])
            .with_location("pipeline/mojang_logo")
            .with_color_target_state(blending(BlendFunction::of(BlendFactor::SRC_ALPHA, BlendFactor::ONE)))
            .build(),
    );
    ENTITY_OUTLINE_BLIT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GLOBALS_SNIPPET])
            .with_location("pipeline/entity_outline_blit")
            .with_vertex_shader("core/screenquad")
            .with_fragment_shader("core/blit_screen")
            .with_bind_group_layout(BindGroupLayouts::IN_SAMPLER)
            .with_color_target_state(ColorTargetState::new(Some(BlendFunction::ENTITY_OUTLINE_BLIT), GpuFormat::RGBA8_UNORM, 7))
            .with_primitive_topology(PrimitiveTopology::TRIANGLES)
            .build(),
    );
    TRACY_BLIT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GLOBALS_SNIPPET])
            .with_location("pipeline/tracy_blit")
            .with_vertex_shader("core/screenquad")
            .with_fragment_shader("core/blit_screen")
            .with_bind_group_layout(BindGroupLayouts::IN_SAMPLER)
            .with_primitive_topology(PrimitiveTopology::TRIANGLES)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    PANORAMA: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GLOBALS_SNIPPET])
            .with_bind_group_layout(BindGroupLayouts::PROJECTION)
            .with_bind_group_layout(BindGroupLayouts::DYNAMIC_TRANSFORMS)
            .with_location("pipeline/panorama")
            .with_vertex_shader("core/panorama")
            .with_fragment_shader("core/panorama")
            .with_bind_group_layout(BindGroupLayouts::SAMPLER0)
            .with_vertex_binding(0, DefaultVertexFormat::POSITION)
            .with_primitive_topology(PrimitiveTopology::QUADS)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    OUTLINE_CULL: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&OUTLINE_SNIPPET])
            .with_location("pipeline/outline_cull")
            .build(),
    );
    OUTLINE_NO_CULL: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&OUTLINE_SNIPPET])
            .with_location("pipeline/outline_no_cull")
            .with_cull(false)
            .build(),
    );
    LIGHTMAP: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GLOBALS_SNIPPET])
            .with_location("pipeline/lightmap")
            .with_vertex_shader("core/screenquad")
            .with_fragment_shader("core/lightmap")
            .with_bind_group_layout(BindGroupLayouts::LIGHTMAP_INFO)
            .with_primitive_topology(PrimitiveTopology::TRIANGLES)
            .with_color_target_state(ColorTargetState::DEFAULT)
            .build(),
    );
    ANIMATE_SPRITE_BLIT: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ANIMATE_SPRITE_SNIPPET])
            .with_fragment_shader("core/animate_sprite_blit")
            .with_location("pipeline/animate_sprite_blit")
            .with_bind_group_layout(BindGroupLayouts::SPRITE)
            .build(),
    );
    ANIMATE_SPRITE_INTERPOLATE: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&ANIMATE_SPRITE_SNIPPET])
            .with_fragment_shader("core/animate_sprite_interpolate")
            .with_location("pipeline/animate_sprite_interpolate")
            .with_bind_group_layout(BindGroupLayouts::CURRENT_SPRITE_NEXT_SPRITE)
            .build(),
    );
    BLIT_DEPTH_BOUNDS: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GLOBALS_SNIPPET])
            .with_location("pipeline/blit_depth_bounds")
            .with_vertex_shader("core/screenquad")
            .with_fragment_shader("core/blit_screen")
            .with_bind_group_layout(BindGroupLayouts::IN_SAMPLER)
            .with_primitive_topology(PrimitiveTopology::TRIANGLES)
            .with_color_target_state(ColorTargetState::new(None, GpuFormat::RGBA32_FLOAT, 15))
            .build(),
    );
    OIT_DEPTH_BOUNDS_CULL: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&OIT_SNIPPET])
            .with_vertex_shader("core/screenquad")
            .with_fragment_shader("core/oit_depth_bounds_cull")
            .with_location("pipeline/oit_depth_bounds_cull")
            .with_primitive_topology(PrimitiveTopology::TRIANGLES)
            .with_bind_group_layout(BindGroupLayouts::DEPTH_BOUNDS_SAMPLER)
            .with_bind_group_layout(BindGroupLayouts::PROJECTION)
            .with_color_target_state(ColorTargetState::new(None, GpuFormat::RGBA32_FLOAT, 15))
            .with_depth_stencil_state(DepthStencilState::DEFAULT)
            .build(),
    );
    BLIT_DEPTH_DURING_DEPTH_BOUNDS: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GLOBALS_SNIPPET])
            .with_location("pipeline/blit_depth_during_depth_bounds")
            .with_vertex_shader("core/screenquad")
            .with_fragment_shader("core/blit_depth")
            .with_bind_group_layout(BindGroupLayouts::IN_SAMPLER)
            .with_primitive_topology(PrimitiveTopology::TRIANGLES)
            .with_depth_stencil_state(DepthStencilState::new(CompareOp::ALWAYS_PASS, true))
            .with_color_target_state(ColorTargetState::new(None, GpuFormat::RGBA32_FLOAT, 0))
            .build(),
    );
    BLIT_DEPTH: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&GLOBALS_SNIPPET])
            .with_location("pipeline/blit_depth")
            .with_vertex_shader("core/screenquad")
            .with_fragment_shader("core/blit_depth")
            .with_bind_group_layout(BindGroupLayouts::IN_SAMPLER)
            .with_primitive_topology(PrimitiveTopology::TRIANGLES)
            .with_depth_stencil_state(DepthStencilState::new(CompareOp::ALWAYS_PASS, true))
            .build(),
    );
    OIT_COMPOSITE: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[&OIT_SNIPPET])
            .with_bind_group_layout(BindGroupLayouts::PROJECTION)
            .with_vertex_shader("core/screenquad")
            .with_fragment_shader("core/oit_composite")
            .with_location("pipeline/oit_composite")
            .with_primitive_topology(PrimitiveTopology::TRIANGLES)
            .with_bind_group_layout(BindGroupLayouts::SAMPLER0_OIT_COEFFS_DEPTH_BOUNDS_SAMPLER)
            .with_color_target_state(ColorTargetState::new(
                Some(BlendFunction::TRANSLUCENT_PREMULTIPLIED_ALPHA),
                GpuFormat::RGBA8_UNORM,
                15,
            ))
            .with_depth_stencil_state(DepthStencilState::new(CompareOp::ALWAYS_PASS, true))
            .build(),
    );
    INTEGRATE_DEPTH: Arc<RenderPipeline> = registry.register(
        RenderPipeline::builder(&[])
            .with_vertex_shader("core/screenquad")
            .with_fragment_shader("core/integrate_depth")
            .with_location("pipeline/integrate_depth")
            .with_primitive_topology(PrimitiveTopology::TRIANGLES)
            .with_bind_group_layout(BindGroupLayouts::IN_SAMPLER)
            .with_depth_stencil_state(DepthStencilState::new(CompareOp::ALWAYS_PASS, true))
            .build(),
    );
}

static PIPELINES: LazyLock<RenderPipelines> = LazyLock::new(RenderPipelines::new);

impl RenderPipelines {
    pub const OIT_DEPTH_BOUNDS_SNIPPET: &StaticSnippet = &OIT_DEPTH_BOUNDS_SNIPPET;
    pub const OIT_TRANSMITTANCE_SNIPPET: &StaticSnippet = &OIT_TRANSMITTANCE_SNIPPET;
    pub const OIT_ACCUMULATE_SNIPPET: &StaticSnippet = &OIT_ACCUMULATE_SNIPPET;
    pub const POST_PROCESSING_SNIPPET: &StaticSnippet = &POST_PROCESSING_SNIPPET;
    pub const GLINT_SNIPPET: &StaticSnippet = &GLINT_SNIPPET;
    pub const GLINT_SPECIAL_SNIPPET: &StaticSnippet = &GLINT_SPECIAL_SNIPPET;
    pub const ANIMATE_SPRITE_SNIPPET: &StaticSnippet = &ANIMATE_SPRITE_SNIPPET;

    /// The pipelines, built in Java's order the first time anything asks for them.
    pub fn get() -> &'static RenderPipelines {
        &PIPELINES
    }

    /// The pipelines `ShaderManager` must compile, in `PIPELINES_BY_LOCATION`'s order: the last
    /// one registered at each location.
    pub fn required_pipelines() -> Vec<Arc<RenderPipeline>> {
        PIPELINES.pipelines_by_location.values().cloned().collect()
    }

    pub fn optional_pipelines() -> Vec<Arc<RenderPipeline>> {
        PIPELINES
            .optional_pipelines_by_location
            .values()
            .cloned()
            .collect()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::tests::{
        encode_pipeline, encode_snippet,
    };
    use crate::golden::Golden;
    use serde_json::{Value, json};

    /// A field's golden encoding: a pipeline's, or an OIT set's three pipelines.
    pub trait EncodeField {
        fn encode_field(&self) -> Value;
    }

    impl EncodeField for Arc<RenderPipeline> {
        fn encode_field(&self) -> Value {
            encode_pipeline(self)
        }
    }

    impl EncodeField for OitPipelineSet {
        fn encode_field(&self) -> Value {
            json!([
                encode_pipeline(&self.depth_bounds_pipeline),
                encode_pipeline(&self.transmittance_pipeline),
                encode_pipeline(&self.accumulate_pipeline)
            ])
        }
    }

    fn locations(pipelines: Vec<Arc<RenderPipeline>>) -> Vec<String> {
        pipelines
            .iter()
            .map(|pipeline| pipeline.get_location().to_string())
            .collect()
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.client.renderer.RenderPipelines");
        for (name, value) in RenderPipelines::get().fields() {
            golden.field(name, value);
        }
        for (name, snippet) in [
            (
                "OIT_DEPTH_BOUNDS_SNIPPET",
                RenderPipelines::OIT_DEPTH_BOUNDS_SNIPPET,
            ),
            (
                "OIT_TRANSMITTANCE_SNIPPET",
                RenderPipelines::OIT_TRANSMITTANCE_SNIPPET,
            ),
            (
                "OIT_ACCUMULATE_SNIPPET",
                RenderPipelines::OIT_ACCUMULATE_SNIPPET,
            ),
            (
                "POST_PROCESSING_SNIPPET",
                RenderPipelines::POST_PROCESSING_SNIPPET,
            ),
            ("GLINT_SNIPPET", RenderPipelines::GLINT_SNIPPET),
            (
                "GLINT_SPECIAL_SNIPPET",
                RenderPipelines::GLINT_SPECIAL_SNIPPET,
            ),
            (
                "ANIMATE_SPRITE_SNIPPET",
                RenderPipelines::ANIMATE_SPRITE_SNIPPET,
            ),
        ] {
            golden.field(name, encode_snippet(snippet));
        }
        golden.check("requiredPipelines()Ljava/util/List;", || {
            locations(RenderPipelines::required_pipelines())
        });
        golden.check("optionalPipelines()Ljava/util/List;", || {
            locations(RenderPipelines::optional_pipelines())
        });
        golden.finish();
    }
}

#[cfg(test)]
pub(crate) mod pipelines_by_location {
    use super::*;

    /// The pipeline `ShaderManager` compiles at `location`, required or optional.
    pub fn pipeline(location: &str) -> Arc<RenderPipeline> {
        RenderPipelines::required_pipelines()
            .into_iter()
            .chain(RenderPipelines::optional_pipelines())
            .find(|pipeline| pipeline.get_location().to_string() == location)
            .unwrap_or_else(|| panic!("no pipeline at {location}"))
    }
}
