//! com/mojang/renderpearl/api/pipeline/RenderPipeline.java
//!
//! A pipeline's bind group layouts are an `ObjectOpenHashSet` in Java, whose order changes from
//! run to run: a layout's hash is its record's, which hashes `UniformType`'s constants by
//! identity. Nothing the game draws depends on that order (shader bindings come from the
//! compiled shaders, see `PipelineBuilder`), so the port keeps them as a `Vec` without repeats,
//! in the order they were added.
//!
//! The builder's errors are panics, where Java throws `IllegalStateException`. A builder's
//! methods take and return `&mut Builder`, like Java's, so `OitPipelineSet`'s modifiers can be
//! `Fn(&mut Builder)`s. `getSortKey`'s shuffle for `DEBUG_SHUFFLE_UI_RENDERING_ORDER`, a debug
//! flag that's off, isn't ported.

use crate::com::mojang::renderpearl::api::pipeline::bind_group_layout::BindGroupLayout;
use crate::com::mojang::renderpearl::api::pipeline::color_target_state::ColorTargetState;
use crate::com::mojang::renderpearl::api::pipeline::depth_stencil_state::DepthStencilState;
use crate::com::mojang::renderpearl::api::pipeline::polygon_mode::PolygonMode;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::com::mojang::renderpearl::api::pipeline::shader_type::ShaderType;
use crate::com::mojang::renderpearl::api::vertex::vertex_format::VertexFormat;
use crate::net::minecraft::client::renderer::shader_defines::{self, ShaderDefines};
use crate::net::minecraft::resources::identifier::Identifier;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::atomic::{AtomicI32, Ordering};

const MAX_VERTEX_BUFFERS: usize = 16;

/// Each buffer slot's vertex format, if the pipeline binds one there.
pub type VertexFormats = [Option<&'static VertexFormat>; MAX_VERTEX_BUFFERS];

/// The shaders by stage, in `ShaderType`'s order like Java's `EnumMap`.
pub type Shaders = BTreeMap<ShaderType, Identifier>;

#[derive(Clone, Debug)]
pub struct RenderPipeline {
    location: Identifier,
    shaders: Shaders,
    shader_defines: ShaderDefines,
    bind_group_layouts: Vec<BindGroupLayout>,
    depth_stencil_state: Option<DepthStencilState>,
    polygon_mode: PolygonMode,
    cull: bool,
    color_target_states: Vec<Option<ColorTargetState>>,
    vertex_format_per_buffer: VertexFormats,
    primitive_topology: PrimitiveTopology,
    push_constant_size: i32,
    sort_key: i32,
}

/// `Builder.nextPipelineSortKey`: every pipeline built gets the next one, which orders the GUI's
/// draws.
static NEXT_PIPELINE_SORT_KEY: AtomicI32 = AtomicI32::new(0);

impl RenderPipeline {
    pub fn get_sort_key(&self) -> i32 {
        self.sort_key
    }

    pub fn get_polygon_mode(&self) -> PolygonMode {
        self.polygon_mode
    }

    pub fn is_cull(&self) -> bool {
        self.cull
    }

    pub fn get_color_target_states(&self) -> &[Option<ColorTargetState>] {
        &self.color_target_states
    }

    pub fn get_depth_stencil_state(&self) -> Option<DepthStencilState> {
        self.depth_stencil_state
    }

    pub fn get_location(&self) -> &Identifier {
        &self.location
    }

    pub fn get_vertex_format_bindings(&self) -> &VertexFormats {
        &self.vertex_format_per_buffer
    }

    pub fn get_vertex_format_binding(&self, binding_index: usize) -> Option<&'static VertexFormat> {
        self.vertex_format_per_buffer[binding_index]
    }

    pub fn get_primitive_topology(&self) -> PrimitiveTopology {
        self.primitive_topology
    }

    pub fn get_shaders(&self) -> &Shaders {
        &self.shaders
    }

    pub fn get_shader_defines(&self) -> &ShaderDefines {
        &self.shader_defines
    }

    pub fn get_bind_group_layouts(&self) -> &[BindGroupLayout] {
        &self.bind_group_layouts
    }

    pub fn wants_depth_texture(&self) -> bool {
        self.depth_stencil_state.is_some()
    }

    pub fn push_constant_size(&self) -> i32 {
        self.push_constant_size
    }

    pub fn builder(snippets: &[&Snippet]) -> Builder {
        let mut builder = Builder::default();
        for snippet in snippets {
            builder.with_snippet(snippet);
        }
        builder
    }
}

impl fmt::Display for RenderPipeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.location.fmt(f)
    }
}

/// `RenderPipeline.Builder`. Java's `Optional` fields are `Option`s, and its arrays the same
/// fixed-size arrays.
#[derive(Clone, Debug, Default)]
pub struct Builder {
    location: Option<Identifier>,
    shaders: Shaders,
    defines_builder: Option<shader_defines::Builder>,
    bind_group_layouts: Option<Vec<BindGroupLayout>>,
    depth_stencil_state: Option<DepthStencilState>,
    polygon_mode: Option<PolygonMode>,
    cull: Option<bool>,
    color_target_states: [Option<ColorTargetState>; ColorTargetState::MAX_COLOR_TARGETS],
    active_color_target_state_count: usize,
    vertex_format_per_buffer: VertexFormats,
    primitive_topology: Option<PrimitiveTopology>,
    push_constant_size: i32,
}

impl Builder {
    /// `withLocation(String)`, in the default namespace.
    pub fn with_location(&mut self, location: &str) -> &mut Builder {
        self.location = Some(Identifier::with_default_namespace(location));
        self
    }

    /// `withLocation(Identifier)`.
    pub fn with_location_id(&mut self, location: Identifier) -> &mut Builder {
        self.location = Some(location);
        self
    }

    pub fn with_fragment_shader(&mut self, fragment_shader: &str) -> &mut Builder {
        self.with_fragment_shader_id(Identifier::with_default_namespace(fragment_shader))
    }

    pub fn with_fragment_shader_id(&mut self, fragment_shader: Identifier) -> &mut Builder {
        self.shaders.insert(ShaderType::FRAGMENT, fragment_shader);
        self
    }

    pub fn with_vertex_shader(&mut self, vertex_shader: &str) -> &mut Builder {
        self.with_vertex_shader_id(Identifier::with_default_namespace(vertex_shader))
    }

    pub fn with_vertex_shader_id(&mut self, vertex_shader: Identifier) -> &mut Builder {
        self.shaders.insert(ShaderType::VERTEX, vertex_shader);
        self
    }

    fn defines(&mut self) -> &mut shader_defines::Builder {
        self.defines_builder
            .get_or_insert_with(ShaderDefines::builder)
    }

    /// `withShaderDefine(String)`, a flag.
    pub fn with_shader_define(&mut self, key: &str) -> &mut Builder {
        self.defines().define_flag(key);
        self
    }

    /// `withShaderDefine(String, int)`.
    pub fn with_shader_define_i32(&mut self, key: &str, value: i32) -> &mut Builder {
        self.defines().define_i32(key, value);
        self
    }

    /// `withShaderDefine(String, float)`.
    pub fn with_shader_define_f32(&mut self, key: &str, value: f32) -> &mut Builder {
        self.defines().define_f32(key, value);
        self
    }

    /// Adds to a set: a layout already there isn't added again.
    pub fn with_bind_group_layout(&mut self, bind_group_layout: &BindGroupLayout) -> &mut Builder {
        let layouts = self.bind_group_layouts.get_or_insert_with(Vec::new);
        if !layouts.contains(bind_group_layout) {
            layouts.push(bind_group_layout.clone());
        }
        self
    }

    pub fn with_polygon_mode(&mut self, polygon_mode: PolygonMode) -> &mut Builder {
        self.polygon_mode = Some(polygon_mode);
        self
    }

    pub fn with_cull(&mut self, cull: bool) -> &mut Builder {
        self.cull = Some(cull);
        self
    }

    /// `withColorTargetState(int, ColorTargetState)`.
    pub fn with_color_target_state_at(
        &mut self,
        index: usize,
        color_target_state: ColorTargetState,
    ) -> &mut Builder {
        self.color_target_states[index] = Some(color_target_state);
        self.active_color_target_state_count = self.active_color_target_state_count.max(index + 1);
        self
    }

    pub fn with_color_target_states(
        &mut self,
        start_index: usize,
        end_index: usize,
        color_target_state: impl Fn() -> ColorTargetState,
    ) -> &mut Builder {
        for index in start_index..=end_index {
            self.color_target_states[index] = Some(color_target_state());
            self.active_color_target_state_count =
                self.active_color_target_state_count.max(index + 1);
        }
        self
    }

    pub fn with_unused_color_target_state(&mut self, index: usize) -> &mut Builder {
        self.color_target_states[index] = None;
        self.active_color_target_state_count = self.active_color_target_state_count.max(index + 1);
        self
    }

    /// `withColorTargetState(ColorTargetState)`, the first target's.
    pub fn with_color_target_state(
        &mut self,
        color_target_state: ColorTargetState,
    ) -> &mut Builder {
        self.with_color_target_state_at(0, color_target_state)
    }

    pub fn with_depth_stencil_state(
        &mut self,
        depth_stencil_state: DepthStencilState,
    ) -> &mut Builder {
        self.depth_stencil_state = Some(depth_stencil_state);
        self
    }

    /// `withDepthStencilState(Optional)`, where an empty one turns depth testing off.
    pub fn with_depth_stencil_state_optional(
        &mut self,
        depth_stencil_state: Option<DepthStencilState>,
    ) -> &mut Builder {
        self.depth_stencil_state = depth_stencil_state;
        self
    }

    pub fn with_vertex_binding(
        &mut self,
        binding_index: usize,
        vertex_format: &'static VertexFormat,
    ) -> &mut Builder {
        self.vertex_format_per_buffer[binding_index] = Some(vertex_format);
        self
    }

    pub fn with_primitive_topology(
        &mut self,
        primitive_topology: PrimitiveTopology,
    ) -> &mut Builder {
        self.primitive_topology = Some(primitive_topology);
        self
    }

    pub fn with_push_constant_size(&mut self, push_constant_size: i32) -> &mut Builder {
        self.push_constant_size = push_constant_size;
        self
    }

    pub fn with_snippet(&mut self, snippet: &Snippet) -> &mut Builder {
        self.shaders.extend(
            snippet
                .shaders
                .iter()
                .map(|(stage, id)| (*stage, id.clone())),
        );
        if let Some(snippet_defines) = &snippet.shader_defines {
            let defines = self.defines();
            for (key, value) in snippet_defines.values() {
                defines.define(key, value);
            }
            for flag in snippet_defines.flags() {
                defines.define_flag(flag);
            }
        }

        if let Some(snippet_layouts) = &snippet.bind_group_layouts {
            for layout in snippet_layouts {
                self.with_bind_group_layout(layout);
            }
        }
        if snippet.depth_stencil_state.is_some() {
            self.depth_stencil_state = snippet.depth_stencil_state;
        }
        if snippet.cull.is_some() {
            self.cull = snippet.cull;
        }

        for index in 0..snippet.active_color_target_state_count {
            if self.color_target_states[index].is_none()
                && snippet.color_target_states[index].is_some()
            {
                self.color_target_states[index] = snippet.color_target_states[index];
            }
        }
        self.active_color_target_state_count = self
            .active_color_target_state_count
            .max(snippet.active_color_target_state_count);

        for (slot, vertex_format) in snippet.vertex_format_per_buffer.iter().enumerate() {
            if vertex_format.is_some() {
                self.vertex_format_per_buffer[slot] = *vertex_format;
            }
        }
        if snippet.vertex_format_mode.is_some() {
            self.primitive_topology = snippet.vertex_format_mode;
        }
        if snippet.polygon_mode.is_some() {
            self.polygon_mode = snippet.polygon_mode;
        }
        self.push_constant_size = self.push_constant_size.max(snippet.push_constant_size);
        self
    }

    pub fn build_snippet(&self) -> Snippet {
        Snippet {
            shaders: self.shaders.clone(),
            shader_defines: self
                .defines_builder
                .as_ref()
                .map(shader_defines::Builder::build),
            bind_group_layouts: self.bind_group_layouts.clone(),
            color_target_states: self.color_target_states,
            active_color_target_state_count: self.active_color_target_state_count,
            depth_stencil_state: self.depth_stencil_state,
            polygon_mode: self.polygon_mode,
            cull: self.cull,
            vertex_format_per_buffer: self.vertex_format_per_buffer,
            vertex_format_mode: self.primitive_topology,
            push_constant_size: self.push_constant_size,
        }
    }

    pub fn build(&self) -> RenderPipeline {
        let location = self.location.clone().expect("Missing location");
        assert!(
            self.shaders.contains_key(&ShaderType::VERTEX),
            "Missing vertex shader"
        );
        assert!(
            self.shaders.contains_key(&ShaderType::FRAGMENT),
            "Missing fragment shader"
        );
        let primitive_topology = self.primitive_topology.expect("Missing primitive topology");

        let active_color_target_states =
            self.color_target_states[..self.active_color_target_state_count].to_vec();
        let mut last_blend = None;
        for state in active_color_target_states.iter().flatten() {
            if let Some(current_blend) = state.blend_function {
                match last_blend {
                    None => last_blend = Some(current_blend),
                    Some(last) => assert!(
                        current_blend == last,
                        "Blend functions must currently be the same for all color targets"
                    ),
                }
            }
        }

        let bound_vertex_attrib_count: usize = self
            .vertex_format_per_buffer
            .iter()
            .flatten()
            .map(|format| format.get_elements().len())
            .sum();
        assert!(
            bound_vertex_attrib_count <= 16,
            "Binding more than 16 vertex attributes is not supported"
        );
        assert!(
            self.push_constant_size <= 128,
            "Maximum push constant size is 128 bytes"
        );

        RenderPipeline {
            location,
            shaders: self.shaders.clone(),
            shader_defines: self
                .defines_builder
                .as_ref()
                .map_or(ShaderDefines::EMPTY, shader_defines::Builder::build),
            bind_group_layouts: self.bind_group_layouts.clone().unwrap_or_default(),
            depth_stencil_state: self.depth_stencil_state,
            polygon_mode: self.polygon_mode.unwrap_or(PolygonMode::FILL),
            cull: self.cull.unwrap_or(true),
            color_target_states: active_color_target_states,
            vertex_format_per_buffer: self.vertex_format_per_buffer,
            primitive_topology,
            push_constant_size: self.push_constant_size,
            sort_key: NEXT_PIPELINE_SORT_KEY.fetch_add(1, Ordering::Relaxed),
        }
    }
}

/// `RenderPipeline.Snippet`: a builder's state, which another builder can start from.
#[derive(Clone, Debug)]
pub struct Snippet {
    pub shaders: Shaders,
    pub shader_defines: Option<ShaderDefines>,
    pub bind_group_layouts: Option<Vec<BindGroupLayout>>,
    pub color_target_states: [Option<ColorTargetState>; ColorTargetState::MAX_COLOR_TARGETS],
    pub active_color_target_state_count: usize,
    pub depth_stencil_state: Option<DepthStencilState>,
    pub polygon_mode: Option<PolygonMode>,
    pub cull: Option<bool>,
    pub vertex_format_per_buffer: VertexFormats,
    pub vertex_format_mode: Option<PrimitiveTopology>,
    pub push_constant_size: i32,
}

#[cfg(test)]
pub(crate) mod tests {
    //! The golden encodings of pipelines and their parts (`RenderSuites` in oracle/golden/).

    use super::*;
    use crate::com::mojang::renderpearl::api::pipeline::bind_group_layout::UniformDescription;
    use crate::com::mojang::renderpearl::api::pipeline::blend_equation::BlendEquation;
    use crate::com::mojang::renderpearl::api::pipeline::blend_function::BlendFunction;
    use crate::golden::Encode;
    use serde_json::{Value, json};

    pub fn encode_vertex_format(format: &VertexFormat) -> Value {
        let elements: Vec<Value> = format
            .get_elements()
            .iter()
            .map(|element| {
                json!([
                    element.name,
                    element.offset,
                    format!("{:?}", element.format)
                ])
            })
            .collect();
        json!([format.get_step_rate(), format.get_vertex_size(), elements])
    }

    pub fn encode_uniforms(uniforms: &[UniformDescription]) -> Value {
        uniforms
            .iter()
            .map(|uniform| {
                json!([
                    uniform.name,
                    format!("{:?}", uniform.uniform_type),
                    uniform.gpu_format.map(|format| format!("{format:?}"))
                ])
            })
            .collect()
    }

    /// Sorted by their uniforms' names, since Java's order changes from run to run.
    fn encode_bind_group_layouts(layouts: &[BindGroupLayout]) -> Value {
        let mut sorted: Vec<&BindGroupLayout> = layouts.iter().collect();
        let names = |layout: &BindGroupLayout| {
            layout
                .uniforms
                .iter()
                .map(|uniform| uniform.name.as_str())
                .collect::<Vec<_>>()
                .join(",")
        };
        sorted.sort_by_key(|layout| names(layout));
        sorted
            .into_iter()
            .map(|layout| encode_uniforms(&layout.uniforms))
            .collect()
    }

    pub fn encode_bind_group_layout(layout: &BindGroupLayout) -> Value {
        encode_uniforms(&layout.uniforms)
    }

    fn encode_blend_equation(equation: &BlendEquation) -> Value {
        json!([
            format!("{:?}", equation.source_factor),
            format!("{:?}", equation.dest_factor),
            format!("{:?}", equation.op)
        ])
    }

    fn encode_blend_function(function: &BlendFunction) -> Value {
        json!([
            encode_blend_equation(&function.color),
            encode_blend_equation(&function.alpha)
        ])
    }

    pub fn encode_color_target_state(state: &ColorTargetState) -> Value {
        json!([
            state.blend_function.as_ref().map(encode_blend_function),
            format!("{:?}", state.format),
            state.write_mask
        ])
    }

    pub fn encode_depth_stencil_state(state: &DepthStencilState) -> Value {
        json!([
            format!("{:?}", state.depth_test),
            state.write_depth,
            state.depth_bias_scale_factor.encode(),
            state.depth_bias_constant.encode()
        ])
    }

    fn encode_shaders(shaders: &Shaders) -> Value {
        shaders
            .iter()
            .map(|(stage, id)| json!([format!("{stage:?}"), id.to_string()]))
            .collect()
    }

    pub fn encode_defines(defines: &ShaderDefines) -> Value {
        let values: Vec<Value> = defines
            .values()
            .iter()
            .map(|(key, value)| json!([key, value]))
            .collect();
        json!([values, defines.flags()])
    }

    fn encode_vertex_formats(formats: &VertexFormats) -> Value {
        formats
            .iter()
            .map(|format| format.map_or(Value::Null, encode_vertex_format))
            .collect()
    }

    fn encode_color_target_states(states: &[Option<ColorTargetState>]) -> Value {
        states
            .iter()
            .map(|state| {
                state
                    .as_ref()
                    .map_or(Value::Null, encode_color_target_state)
            })
            .collect()
    }

    pub fn encode_pipeline(pipeline: &RenderPipeline) -> Value {
        json!([
            pipeline.location.to_string(),
            encode_shaders(&pipeline.shaders),
            encode_defines(&pipeline.shader_defines),
            encode_bind_group_layouts(&pipeline.bind_group_layouts),
            encode_color_target_states(&pipeline.color_target_states),
            pipeline
                .depth_stencil_state
                .as_ref()
                .map(encode_depth_stencil_state),
            format!("{:?}", pipeline.polygon_mode),
            pipeline.cull,
            encode_vertex_formats(&pipeline.vertex_format_per_buffer),
            format!("{:?}", pipeline.primitive_topology),
            pipeline.push_constant_size,
            pipeline.sort_key
        ])
    }

    pub fn encode_snippet(snippet: &Snippet) -> Value {
        json!([
            encode_shaders(&snippet.shaders),
            snippet.shader_defines.as_ref().map(encode_defines),
            snippet
                .bind_group_layouts
                .as_deref()
                .map(encode_bind_group_layouts),
            encode_color_target_states(&snippet.color_target_states),
            snippet.active_color_target_state_count,
            snippet
                .depth_stencil_state
                .as_ref()
                .map(encode_depth_stencil_state),
            snippet.polygon_mode.map(|mode| format!("{mode:?}")),
            snippet.cull,
            encode_vertex_formats(&snippet.vertex_format_per_buffer),
            snippet.vertex_format_mode.map(|mode| format!("{mode:?}")),
            snippet.push_constant_size
        ])
    }
}
