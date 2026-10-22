//! com/mojang/renderpearl/backend/opengl/GlConst.java
//!
//! Only the format and shader stage mappings so far.

use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat::{self, *};
use crate::com::mojang::renderpearl::api::pipeline::shader_type::ShaderType;

pub struct GlConst;

impl GlConst {
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
