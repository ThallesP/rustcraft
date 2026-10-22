//! com/mojang/renderpearl/backend/api/BackendRenderPipeline.java
//!
//! Only `CreateInfo` so far: what the frontend hands a backend to make a pipeline from.

use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::pipeline::bind_group_layout::UniformDescription;
use crate::com::mojang::renderpearl::api::pipeline::color_target_state::ColorTargetState;
use crate::com::mojang::renderpearl::api::pipeline::depth_stencil_state::DepthStencilState;
use crate::com::mojang::renderpearl::api::pipeline::polygon_mode::PolygonMode;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::com::mojang::renderpearl::backend::api::spv_module::SpvModule;

/// `BackendRenderPipeline.CreateInfo`.
pub struct CreateInfo {
    pub name: String,
    pub shaders: Vec<Shader>,
    pub vertex_buffers: Vec<VertexBuffer>,
    pub attrib_bindings: Vec<AttribBinding>,
    pub uniforms: Vec<UniformDescription>,
    pub push_constants_size: i32,
    pub depth_stencil_state: Option<DepthStencilState>,
    pub polygon_mode: PolygonMode,
    pub cull: bool,
    pub color_target_states: Vec<Option<ColorTargetState>>,
    pub primitive_topology: PrimitiveTopology,
}

/// `CreateInfo.AttribBinding`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttribBinding {
    pub buffer_slot: i32,
    pub location: i32,
    pub offset: i32,
    pub format: GpuFormat,
}

/// `CreateInfo.Shader`.
pub struct Shader {
    pub name: String,
    pub entry_point: String,
    pub module: Box<dyn SpvModule>,
}

/// `CreateInfo.VertexBuffer`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VertexBuffer {
    pub buffer_slot: i32,
    pub stride: i32,
    pub step_rate: i32,
}
