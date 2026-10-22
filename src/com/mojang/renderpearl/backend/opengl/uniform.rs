//! com/mojang/renderpearl/backend/opengl/Uniform.java
//!
//! How a program reads each of a pipeline's uniforms: a uniform block at a binding, a sampler at
//! a texture unit, or a texel buffer at a unit with a buffer texture of its own, which `GlProgram`
//! deletes when it's dropped (Java's `close()`).

use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uniform {
    Sampler {
        sampler_index: i32,
    },
    Ubo {
        block_binding: i32,
    },
    Utb {
        sampler_index: i32,
        format: GpuFormat,
        texture: glow::NativeTexture,
    },
}
