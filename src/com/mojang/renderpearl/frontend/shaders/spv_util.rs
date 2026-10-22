//! com/mojang/renderpearl/frontend/shaders/SpvUtil.java
//!
//! `getDecorationOffset` isn't ported: nothing calls it.

use crate::com::mojang::renderpearl::api::pipeline::uniform_type::UniformType;
use crate::com::mojang::renderpearl::util::shader_compile_exception::ShaderCompileException;
use crate::org::lwjgl::util::spvc::spvc::{
    SPVC_RESOURCE_TYPE_SAMPLED_IMAGE, SPVC_RESOURCE_TYPE_UNIFORM_BUFFER, SpvcResult,
};

pub struct SpvUtil;

impl SpvUtil {
    /// The resource types a descriptor can have: uniform and storage buffers, storage and sampled
    /// images, separate images and samplers.
    pub const DESCRIPTOR_TYPES: [i32; 6] = [1, 2, 6, 7, 10, 11];

    /// Panics where Java throws `IllegalStateException`.
    pub fn crash_if_error(result: SpvcResult, message: &str) {
        assert!(
            result == 0,
            "{message} ({})",
            SpvUtil::spvc_error_string(result)
        );
    }

    pub fn throw_if_error(result: SpvcResult, message: &str) -> Result<(), ShaderCompileException> {
        if result != 0 {
            return Err(ShaderCompileException(format!(
                "{message} ({})",
                SpvUtil::spvc_error_string(result)
            )));
        }
        Ok(())
    }

    pub fn spvc_error_string(result: SpvcResult) -> String {
        match result {
            -4 => "SPVC_ERROR_INVALID_ARGUMENT".to_owned(),
            -3 => "SPVC_ERROR_OUT_OF_MEMORY".to_owned(),
            -2 => "SPVC_ERROR_UNSUPPORTED_SPIRV".to_owned(),
            -1 => "SPVC_ERROR_INVALID_SPIRV".to_owned(),
            _ => result.to_string(),
        }
    }

    /// A texel buffer is a sampled image too (`samplerBuffer`).
    pub fn resource_type(uniform_type: UniformType) -> i32 {
        match uniform_type {
            UniformType::COMBINED_IMAGE_SAMPLER => SPVC_RESOURCE_TYPE_SAMPLED_IMAGE,
            UniformType::UNIFORM_BUFFER => SPVC_RESOURCE_TYPE_UNIFORM_BUFFER,
            UniformType::TEXEL_BUFFER => SPVC_RESOURCE_TYPE_SAMPLED_IMAGE,
        }
    }

    pub fn base_type_string(base_type: i32) -> &'static str {
        match base_type {
            0 => "unknown",
            1 => "void",
            2 => "bool",
            3 => "int8",
            4 => "uint8",
            5 => "int16",
            6 => "uint16",
            7 => "int32",
            8 => "uint32",
            9 => "int64",
            10 => "uint64",
            11 => "atomic_counter",
            12 => "fp16",
            13 => "fp32",
            14 => "fp64",
            15 => "struct",
            16 => "image",
            17 => "sampled_image",
            18 => "sampler",
            19 => "acceleration_structure",
            _ => "UNKNOWN_TYPE",
        }
    }
}
