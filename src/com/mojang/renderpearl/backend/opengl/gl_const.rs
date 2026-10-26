//! com/mojang/renderpearl/backend/opengl/GlConst.java
//!
//! Java's `toGl` overloads take their type's name (`to_gl_compare_op`), except `ShaderType`'s,
//! which came first. The buffer usage flags come with mapped buffers.

use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat::{self, *};
use crate::com::mojang::renderpearl::api::pipeline::blend_factor::BlendFactor;
use crate::com::mojang::renderpearl::api::pipeline::blend_op::BlendOp;
use crate::com::mojang::renderpearl::api::pipeline::compare_op::CompareOp;
use crate::com::mojang::renderpearl::api::pipeline::index_type::IndexType;
use crate::com::mojang::renderpearl::api::pipeline::polygon_mode::PolygonMode;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::com::mojang::renderpearl::api::pipeline::shader_type::ShaderType;
use crate::com::mojang::renderpearl::api::textures::address_mode::AddressMode;

pub struct GlConst;

impl GlConst {
    /// `CUBEMAP_TARGETS`: a cube map's faces, by layer.
    pub const CUBEMAP_TARGETS: [u32; 6] = [
        glow::TEXTURE_CUBE_MAP_POSITIVE_X,
        glow::TEXTURE_CUBE_MAP_NEGATIVE_X,
        glow::TEXTURE_CUBE_MAP_POSITIVE_Y,
        glow::TEXTURE_CUBE_MAP_NEGATIVE_Y,
        glow::TEXTURE_CUBE_MAP_POSITIVE_Z,
        glow::TEXTURE_CUBE_MAP_NEGATIVE_Z,
    ];

    pub fn to_gl_compare_op(compare_op: CompareOp) -> u32 {
        match compare_op {
            CompareOp::ALWAYS_PASS => 519,
            CompareOp::LESS_THAN => 513,
            CompareOp::LESS_THAN_OR_EQUAL => 515,
            CompareOp::EQUAL => 514,
            CompareOp::NOT_EQUAL => 517,
            CompareOp::GREATER_THAN_OR_EQUAL => 518,
            CompareOp::GREATER_THAN => 516,
            CompareOp::NEVER_PASS => 512,
        }
    }

    pub fn to_gl_polygon_mode(polygon_mode: PolygonMode) -> u32 {
        match polygon_mode {
            PolygonMode::WIREFRAME => 6913,
            PolygonMode::FILL => 6914,
        }
    }

    pub fn to_gl_blend_factor(blend_factor: BlendFactor) -> u32 {
        match blend_factor {
            BlendFactor::CONSTANT_ALPHA => 32771,
            BlendFactor::CONSTANT_COLOR => 32769,
            BlendFactor::DST_ALPHA => 772,
            BlendFactor::DST_COLOR => 774,
            BlendFactor::ONE => 1,
            BlendFactor::ONE_MINUS_CONSTANT_ALPHA => 32772,
            BlendFactor::ONE_MINUS_CONSTANT_COLOR => 32770,
            BlendFactor::ONE_MINUS_DST_ALPHA => 773,
            BlendFactor::ONE_MINUS_DST_COLOR => 775,
            BlendFactor::ONE_MINUS_SRC_ALPHA => 771,
            BlendFactor::ONE_MINUS_SRC_COLOR => 769,
            BlendFactor::SRC_ALPHA => 770,
            BlendFactor::SRC_ALPHA_SATURATE => 776,
            BlendFactor::SRC_COLOR => 768,
            BlendFactor::ZERO => 0,
        }
    }

    pub fn to_gl_blend_op(blend_op: BlendOp) -> u32 {
        match blend_op {
            BlendOp::ADD => 32774,
            BlendOp::SUBTRACT => 32778,
            BlendOp::REVERSE_SUBTRACT => 32779,
            BlendOp::MIN => 32775,
            BlendOp::MAX => 32776,
        }
    }

    /// Quads are drawn as indexed triangles, so they're triangles to GL, like lines.
    pub fn to_gl_primitive_topology(primitive_topology: PrimitiveTopology) -> u32 {
        match primitive_topology {
            PrimitiveTopology::LINES => 4,
            PrimitiveTopology::DEBUG_LINES => 1,
            PrimitiveTopology::DEBUG_LINE_STRIP => 3,
            PrimitiveTopology::POINTS => 0,
            PrimitiveTopology::TRIANGLES => 4,
            PrimitiveTopology::TRIANGLE_STRIP => 5,
            PrimitiveTopology::TRIANGLE_FAN => 6,
            PrimitiveTopology::QUADS => 4,
        }
    }

    pub fn to_gl_index_type(index_type: IndexType) -> u32 {
        match index_type {
            IndexType::SHORT => 5123,
            IndexType::INT => 5125,
        }
    }

    pub fn to_gl_address_mode(address_mode: AddressMode) -> u32 {
        match address_mode {
            AddressMode::REPEAT => 10497,
            AddressMode::CLAMP_TO_EDGE => 33071,
        }
    }

    pub fn gl_format_channel_count(gl_external_id: u32) -> i32 {
        match gl_external_id {
            36249 | 6408 => 4,
            36248 | 6407 => 3,
            33320 | 33319 => 2,
            36244 | 6403 => 1,
            _ => 0,
        }
    }

    pub fn is_gl_format_integer(gl_external_id: u32) -> bool {
        matches!(gl_external_id, 36249 | 36248 | 33320 | 36244)
    }

    pub fn is_format_normalized(gpu_format: GpuFormat) -> bool {
        matches!(
            gpu_format,
            R8_UNORM
                | R8_SNORM
                | R16_UNORM
                | R16_SNORM
                | RG8_UNORM
                | RG8_SNORM
                | RG16_UNORM
                | RG16_SNORM
                | RGB8_UNORM
                | RGB8_SNORM
                | RGB16_UNORM
                | RGB16_SNORM
                | RGBA8_UNORM
                | RGBA8_SNORM
                | RGBA16_UNORM
                | RGB10A2_UNORM
                | D16_UNORM
        )
    }

    /// The texture's internal format, or 0 for one GL textures can't have.
    pub fn to_gl_internal_id(gpu_format: GpuFormat) -> u32 {
        match gpu_format {
            R8_UNORM => 33321,
            R8_SNORM => 36756,
            R16_UNORM => 33322,
            R16_SNORM => 36760,
            RG8_UNORM => 33323,
            RG8_SNORM => 36757,
            RG16_UNORM => 33324,
            RG16_SNORM => 36761,
            RGBA8_UNORM => 32856,
            RGBA8_SNORM => 36759,
            RGBA16_UNORM => 32859,
            RGB10A2_UNORM => 32857,
            D16_UNORM => 33189,
            RGBA16_SNORM => 36763,
            R8_UINT => 33330,
            R8_SINT => 33329,
            RG8_UINT => 33336,
            RG8_SINT => 33335,
            RGBA8_UINT => 36220,
            RGBA8_SINT => 36238,
            R16_UINT => 33332,
            R16_SINT => 33331,
            RG16_UINT => 33338,
            RG16_SINT => 33337,
            RGBA16_UINT => 36214,
            RGBA16_SINT => 36232,
            R32_UINT => 33334,
            R32_SINT => 33333,
            RG32_UINT => 33340,
            RG32_SINT => 33339,
            RGB32_UINT => 36209,
            RGB32_SINT => 36227,
            RGBA32_UINT => 36208,
            RGBA32_SINT => 36226,
            R16_FLOAT => 33325,
            RG16_FLOAT => 33327,
            RGBA16_FLOAT => 34842,
            R32_FLOAT => 33326,
            RG32_FLOAT => 33328,
            RGBA32_FLOAT => 34836,
            RGB10A2_UINT => 36975,
            RG11B10_FLOAT => 35898,
            D32_FLOAT => 36012,
            D32_FLOAT_S8_UINT => 36013,
            D24_UNORM_S8_UINT => 35056,
            S8_UINT => 36168,
            _ => 0,
        }
    }

    pub fn to_gl_external_id(gpu_format: GpuFormat) -> u32 {
        match gpu_format {
            R8_UNORM | R8_SNORM | R16_UNORM | R16_SNORM | R16_FLOAT | R32_FLOAT => 6403,
            RG8_UNORM | RG8_SNORM | RG16_UNORM | RG16_SNORM | RG16_FLOAT | RG32_FLOAT => 33319,
            RGB8_UNORM | RGB8_SNORM | RGB16_UNORM | RGB16_SNORM | RG11B10_FLOAT | RGB16_FLOAT
            | RGB32_FLOAT => 6407,
            RGBA8_UNORM | RGBA8_SNORM | RGBA16_UNORM | RGB10A2_UNORM | RGBA16_SNORM
            | RGBA16_FLOAT | RGBA32_FLOAT => 6408,
            D16_UNORM | D32_FLOAT => 6402,
            R8_UINT | R8_SINT | R16_UINT | R16_SINT | R32_UINT | R32_SINT => 36244,
            RG8_UINT | RG8_SINT | RG16_UINT | RG16_SINT | RG32_UINT | RG32_SINT => 33320,
            RGBA8_UINT | RGBA8_SINT | RGBA16_UINT | RGBA16_SINT | RGBA32_UINT | RGBA32_SINT
            | RGB10A2_UINT => 36249,
            RGB32_UINT | RGB32_SINT | RGB8_UINT | RGB8_SINT | RGB16_UINT | RGB16_SINT => 36248,
            D32_FLOAT_S8_UINT | D24_UNORM_S8_UINT => 34041,
            S8_UINT => 6401,
        }
    }

    pub fn to_gl_type(gpu_format: GpuFormat) -> u32 {
        match gpu_format {
            R8_UNORM | RG8_UNORM | RGB8_UNORM | RGBA8_UNORM | R8_UINT | RG8_UINT | RGBA8_UINT
            | S8_UINT | RGB8_UINT => 5121,
            R8_SNORM | RG8_SNORM | RGB8_SNORM | RGBA8_SNORM | R8_SINT | RG8_SINT | RGBA8_SINT
            | RGB8_SINT => 5120,
            R16_UNORM | RG16_UNORM | RGB16_UNORM | RGBA16_UNORM | D16_UNORM | R16_UINT
            | RG16_UINT | RGBA16_UINT | RGB16_UINT => 5123,
            R16_SNORM | RG16_SNORM | RGB16_SNORM | RGBA16_SNORM | R16_SINT | RG16_SINT
            | RGBA16_SINT | RGB16_SINT => 5122,
            RGB10A2_UNORM | RGB10A2_UINT => 33640,
            R32_UINT | RG32_UINT | RGB32_UINT | RGBA32_UINT => 5125,
            R32_SINT | RG32_SINT | RGB32_SINT | RGBA32_SINT => 5124,
            R16_FLOAT | RG16_FLOAT | RGBA16_FLOAT | RGB16_FLOAT => 5131,
            R32_FLOAT | RG32_FLOAT | RGBA32_FLOAT | D32_FLOAT | RGB32_FLOAT => 5126,
            RG11B10_FLOAT => 35899,
            D32_FLOAT_S8_UINT => 36269,
            D24_UNORM_S8_UINT => 34042,
        }
    }

    pub fn to_gl(shader_type: ShaderType) -> u32 {
        match shader_type {
            ShaderType::VERTEX => 35633,
            ShaderType::FRAGMENT => 35632,
        }
    }
}
