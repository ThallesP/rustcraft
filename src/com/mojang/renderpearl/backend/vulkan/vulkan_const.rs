//! com/mojang/renderpearl/backend/vulkan/VulkanConst.java
//!
//! The API's enums and usages as Vulkan's values, as Java's ints (`VK_FORMAT_R8G8B8A8_UNORM` is
//! 37), which the callers make `ash`'s types of. Java's `toVk` overloads get their argument's type
//! (`to_vk_format`, `to_vk_blend_factor`).

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer;
use crate::com::mojang::renderpearl::api::device::gpu_surface::PresentMode;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::pipeline::blend_factor::BlendFactor;
use crate::com::mojang::renderpearl::api::pipeline::blend_op::BlendOp;
use crate::com::mojang::renderpearl::api::pipeline::color_target_state::ColorTargetState;
use crate::com::mojang::renderpearl::api::pipeline::compare_op::CompareOp;
use crate::com::mojang::renderpearl::api::pipeline::polygon_mode::PolygonMode;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::com::mojang::renderpearl::api::pipeline::shader_type::ShaderType;
use crate::com::mojang::renderpearl::api::textures::address_mode::AddressMode;
use crate::com::mojang::renderpearl::api::textures::filter_mode::FilterMode;
use crate::com::mojang::renderpearl::api::textures::gpu_texture;

pub struct VulkanConst;

impl VulkanConst {
    pub fn texture_usage_to_vk(usage: i32, format: GpuFormat) -> i32 {
        let mut vk_usage = 0;
        if usage & gpu_texture::USAGE_RENDER_ATTACHMENT != 0 {
            if format.has_color_aspect() {
                vk_usage |= 16;
            }
            if format.has_depth_aspect() {
                vk_usage |= 32;
            }
        }
        if usage & gpu_texture::USAGE_TEXTURE_BINDING != 0 {
            vk_usage |= 4;
        }
        if usage & gpu_texture::USAGE_COPY_DST != 0 {
            vk_usage |= 2;
        }
        if usage & gpu_texture::USAGE_COPY_SRC != 0 {
            vk_usage |= 1;
        }
        vk_usage
    }

    pub fn buffer_usage_to_vk(usage: i32) -> i32 {
        let mut result = 0;
        if usage & gpu_buffer::USAGE_COPY_DST != 0 {
            result |= 2;
        }
        if usage & gpu_buffer::USAGE_COPY_SRC != 0 {
            result |= 1;
        }
        if usage & gpu_buffer::USAGE_VERTEX != 0 {
            result |= 128;
        }
        if usage & gpu_buffer::USAGE_INDEX != 0 {
            result |= 64;
        }
        if usage & gpu_buffer::USAGE_UNIFORM != 0 {
            result |= 16;
        }
        if usage & gpu_buffer::USAGE_UNIFORM_TEXEL_BUFFER != 0 {
            result |= 4;
        }
        if usage & gpu_buffer::USAGE_INDIRECT_PARAMETERS != 0 {
            result |= 256;
        }
        result
    }

    pub fn format_aspect_mask(format: GpuFormat) -> i32 {
        let mut aspect_mask = 0;
        if format.has_color_aspect() {
            aspect_mask |= 1;
        }
        if format.has_depth_aspect() {
            aspect_mask |= 2;
        }
        if format.has_stencil_aspect() {
            aspect_mask |= 4;
        }
        aspect_mask
    }

    pub fn to_vk_address_mode(address_mode: AddressMode) -> i32 {
        match address_mode {
            AddressMode::REPEAT => 0,
            AddressMode::CLAMP_TO_EDGE => 2,
        }
    }

    pub fn to_vk_filter_mode(filter: FilterMode) -> i32 {
        match filter {
            FilterMode::NEAREST => 0,
            FilterMode::LINEAR => 1,
        }
    }

    pub fn to_vk_format(format: GpuFormat) -> i32 {
        use GpuFormat::*;
        match format {
            R8_UNORM => 9,
            R8_SNORM => 10,
            RG8_UNORM => 16,
            RG8_SNORM => 17,
            RGB8_UNORM => 23,
            RGB8_SNORM => 24,
            RGBA8_UNORM => 37,
            RGBA8_SNORM => 38,
            R16_UNORM => 70,
            R16_SNORM => 71,
            RG16_UNORM => 77,
            RG16_SNORM => 78,
            RGB16_UNORM => 84,
            RGB16_SNORM => 85,
            RGBA16_UNORM => 91,
            RGBA16_SNORM => 92,
            R8_UINT => 13,
            R8_SINT => 14,
            RG8_UINT => 20,
            RG8_SINT => 21,
            RGB8_UINT => 27,
            RGB8_SINT => 28,
            RGBA8_UINT => 41,
            RGBA8_SINT => 42,
            R16_UINT => 74,
            R16_SINT => 75,
            RG16_UINT => 81,
            RG16_SINT => 82,
            RGB16_UINT => 88,
            RGB16_SINT => 89,
            RGBA16_UINT => 95,
            RGBA16_SINT => 96,
            R32_UINT => 98,
            R32_SINT => 99,
            RG32_UINT => 101,
            RG32_SINT => 102,
            RGB32_UINT => 104,
            RGB32_SINT => 105,
            RGBA32_UINT => 107,
            RGBA32_SINT => 108,
            R16_FLOAT => 76,
            RG16_FLOAT => 83,
            RGB16_FLOAT => 90,
            RGBA16_FLOAT => 97,
            R32_FLOAT => 100,
            RG32_FLOAT => 103,
            RGB32_FLOAT => 106,
            RGBA32_FLOAT => 109,
            RGB10A2_UNORM => 64,
            RGB10A2_UINT => 68,
            RG11B10_FLOAT => 122,
            D32_FLOAT => 126,
            D32_FLOAT_S8_UINT => 130,
            D24_UNORM_S8_UINT => 129,
            D16_UNORM => 124,
            S8_UINT => 127,
        }
    }

    pub fn to_vk_blend_factor(factor: BlendFactor) -> i32 {
        use BlendFactor::*;
        match factor {
            CONSTANT_ALPHA => 12,
            CONSTANT_COLOR => 10,
            ONE_MINUS_CONSTANT_ALPHA => 13,
            ONE_MINUS_CONSTANT_COLOR => 11,
            DST_ALPHA => 8,
            DST_COLOR => 4,
            ONE => 1,
            ONE_MINUS_DST_ALPHA => 9,
            ONE_MINUS_DST_COLOR => 5,
            ONE_MINUS_SRC_ALPHA => 7,
            ONE_MINUS_SRC_COLOR => 3,
            SRC_ALPHA => 6,
            SRC_ALPHA_SATURATE => 14,
            SRC_COLOR => 2,
            ZERO => 0,
        }
    }

    pub fn to_vk_blend_op(blend_op: BlendOp) -> i32 {
        match blend_op {
            BlendOp::ADD => 0,
            BlendOp::SUBTRACT => 1,
            BlendOp::REVERSE_SUBTRACT => 2,
            BlendOp::MIN => 3,
            BlendOp::MAX => 4,
        }
    }

    pub fn to_vk_compare_op(op: CompareOp) -> i32 {
        match op {
            CompareOp::ALWAYS_PASS => 7,
            CompareOp::LESS_THAN => 1,
            CompareOp::LESS_THAN_OR_EQUAL => 3,
            CompareOp::EQUAL => 2,
            CompareOp::NOT_EQUAL => 5,
            CompareOp::GREATER_THAN_OR_EQUAL => 6,
            CompareOp::GREATER_THAN => 4,
            CompareOp::NEVER_PASS => 0,
        }
    }

    pub fn to_vk_polygon_mode(polygon_mode: PolygonMode) -> i32 {
        match polygon_mode {
            PolygonMode::FILL => 0,
            PolygonMode::WIREFRAME => 1,
        }
    }

    /// Lines and quads are drawn as triangles: the vertex shaders widen lines, and quads have
    /// their own index buffers.
    pub fn to_vk_primitive_topology(primitive_topology: PrimitiveTopology) -> i32 {
        use PrimitiveTopology::*;
        match primitive_topology {
            LINES => 3,
            DEBUG_LINES => 1,
            DEBUG_LINE_STRIP => 2,
            POINTS => 0,
            TRIANGLES => 3,
            TRIANGLE_STRIP => 4,
            TRIANGLE_FAN => 5,
            QUADS => 3,
        }
    }

    /// `toVk(ColorTargetState)`: its write mask.
    pub fn to_vk_color_target_state(color_target_state: &ColorTargetState) -> i32 {
        let mut result = 0;
        if color_target_state.write_alpha() {
            result |= 8;
        }
        if color_target_state.write_red() {
            result |= 1;
        }
        if color_target_state.write_green() {
            result |= 2;
        }
        if color_target_state.write_blue() {
            result |= 4;
        }
        result
    }

    pub fn to_vk_present_mode(mode: PresentMode) -> i32 {
        match mode {
            PresentMode::IMMEDIATE => 0,
            PresentMode::MAILBOX => 1,
            PresentMode::FIFO => 2,
            PresentMode::FIFO_RELAXED => 3,
        }
    }

    pub fn to_vk_shader_type(shader_type: ShaderType) -> i32 {
        match shader_type {
            ShaderType::VERTEX => 1,
            ShaderType::FRAGMENT => 16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::golden::Golden;
    use serde_json::Value;

    /// Each case of `method`, its result against `actual` of its arguments.
    fn check(golden: &Golden, method: &str, actual: impl Fn(&[Value]) -> i32) {
        for (args, result) in golden.cases(method) {
            assert_eq!(
                Some(i64::from(actual(&args))),
                result.as_i64(),
                "{method} of {args:?}"
            );
        }
    }

    /// The constant of `values` named by the argument.
    fn constant<E: std::fmt::Debug + Copy>(values: &[E], arg: &Value) -> E {
        *values
            .iter()
            .find(|value| format!("{value:?}") == arg.as_str().expect("a name"))
            .expect("a constant")
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.renderpearl.backend.vulkan.VulkanConst");
        let format = |arg: &Value| constant(&GpuFormat::VALUES, arg);
        let int = |arg: &Value| arg.as_i64().expect("an int") as i32;
        const P: &str = "Lcom/mojang/renderpearl/api/";
        check(&golden, &format!("toVk({P}GpuFormat;)I"), |args| {
            VulkanConst::to_vk_format(format(&args[0]))
        });
        check(
            &golden,
            &format!("formatAspectMask({P}GpuFormat;)I"),
            |args| VulkanConst::format_aspect_mask(format(&args[0])),
        );
        check(
            &golden,
            &format!("textureUsageToVk(I{P}GpuFormat;)I"),
            |args| VulkanConst::texture_usage_to_vk(int(&args[0]), format(&args[1])),
        );
        check(&golden, "bufferUsageToVk(I)I", |args| {
            VulkanConst::buffer_usage_to_vk(int(&args[0]))
        });
        check(
            &golden,
            &format!("toVk({P}textures/AddressMode;)I"),
            |args| VulkanConst::to_vk_address_mode(constant(&AddressMode::VALUES, &args[0])),
        );
        check(
            &golden,
            &format!("toVk({P}textures/FilterMode;)I"),
            |args| VulkanConst::to_vk_filter_mode(constant(&FilterMode::VALUES, &args[0])),
        );
        check(
            &golden,
            &format!("toVk({P}pipeline/BlendFactor;)I"),
            |args| VulkanConst::to_vk_blend_factor(constant(&BlendFactor::VALUES, &args[0])),
        );
        check(&golden, &format!("toVk({P}pipeline/BlendOp;)I"), |args| {
            VulkanConst::to_vk_blend_op(constant(&BlendOp::VALUES, &args[0]))
        });
        check(&golden, &format!("toVk({P}pipeline/CompareOp;)I"), |args| {
            VulkanConst::to_vk_compare_op(constant(&CompareOp::VALUES, &args[0]))
        });
        check(
            &golden,
            &format!("toVk({P}pipeline/PolygonMode;)I"),
            |args| VulkanConst::to_vk_polygon_mode(constant(&PolygonMode::VALUES, &args[0])),
        );
        check(
            &golden,
            &format!("toVk({P}pipeline/PrimitiveTopology;)I"),
            |args| {
                VulkanConst::to_vk_primitive_topology(constant(
                    &PrimitiveTopology::VALUES,
                    &args[0],
                ))
            },
        );
        check(
            &golden,
            &format!("toVk({P}device/GpuSurface$PresentMode;)I"),
            |args| VulkanConst::to_vk_present_mode(constant(&PresentMode::VALUES, &args[0])),
        );
        check(
            &golden,
            &format!("toVk({P}pipeline/ShaderType;)I"),
            |args| VulkanConst::to_vk_shader_type(constant(&ShaderType::VALUES, &args[0])),
        );
        check(
            &golden,
            &format!("toVk({P}pipeline/ColorTargetState;)I"),
            |args| {
                VulkanConst::to_vk_color_target_state(&ColorTargetState::new(
                    None,
                    GpuFormat::RGBA8_UNORM,
                    int(&args[0]),
                ))
            },
        );
        golden.finish();
    }
}
