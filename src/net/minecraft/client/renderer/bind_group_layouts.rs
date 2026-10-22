//! net/minecraft/client/renderer/BindGroupLayouts.java

use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::pipeline::bind_group_layout::{self, BindGroupLayout};
use crate::com::mojang::renderpearl::api::pipeline::uniform_type::UniformType::{
    self, COMBINED_IMAGE_SAMPLER, TEXEL_BUFFER, UNIFORM_BUFFER,
};
use crate::net::minecraft::client::renderer::level_renderer::LevelRenderer;
use std::sync::LazyLock;

pub struct BindGroupLayouts;

type StaticBindGroupLayout = LazyLock<BindGroupLayout>;

macro_rules! layouts {
    ($($name:ident = $builder:expr;)*) => {
        $(static $name: StaticBindGroupLayout = LazyLock::new(|| $builder.build());)*

        impl BindGroupLayouts {
            $(pub const $name: &StaticBindGroupLayout = &$name;)*
        }

        #[cfg(test)]
        const ALL: &[(&str, &StaticBindGroupLayout)] = &[$((stringify!($name), &$name)),*];
    };
}

fn uniform(name: &str, uniform_type: UniformType) -> bind_group_layout::Builder {
    BindGroupLayout::builder().with_uniform(name, uniform_type)
}

/// The OIT layouts' samplers for the transmittance targets, `Coeff0` onwards.
fn with_coefficients(mut builder: bind_group_layout::Builder) -> bind_group_layout::Builder {
    for i in 0..LevelRenderer::OIT_TRANSMITTANCE_TARGET_COUNT {
        builder = builder.with_uniform(&format!("Coeff{i}"), COMBINED_IMAGE_SAMPLER);
    }
    builder
}

layouts! {
    DYNAMIC_TRANSFORMS = uniform("DynamicTransforms", UNIFORM_BUFFER);
    PROJECTION = uniform("Projection", UNIFORM_BUFFER);
    CHUNK_SECTION = uniform("ChunkSection", UNIFORM_BUFFER);
    TERRAIN_INFO = uniform("TerrainUniform", UNIFORM_BUFFER);
    FOG = uniform("Fog", UNIFORM_BUFFER);
    GLOBALS = uniform("Globals", UNIFORM_BUFFER);
    LIGHTING = uniform("Lighting", UNIFORM_BUFFER);
    SAMPLER0 = uniform("Sampler0", COMBINED_IMAGE_SAMPLER);
    SAMPLER1 = uniform("Sampler1", COMBINED_IMAGE_SAMPLER);
    SAMPLER2 = uniform("Sampler2", COMBINED_IMAGE_SAMPLER);
    SAMPLER0_SAMPLER2 = uniform("Sampler0", COMBINED_IMAGE_SAMPLER).with_uniform("Sampler2", COMBINED_IMAGE_SAMPLER);
    SAMPLER0_SAMPLER1 = uniform("Sampler0", COMBINED_IMAGE_SAMPLER).with_uniform("Sampler1", COMBINED_IMAGE_SAMPLER);
    SAMPLER0_SAMPLER1_SAMPLER2 = uniform("Sampler0", COMBINED_IMAGE_SAMPLER)
        .with_uniform("Sampler1", COMBINED_IMAGE_SAMPLER)
        .with_uniform("Sampler2", COMBINED_IMAGE_SAMPLER);
    CLOUD_INFO = uniform("CloudInfo", UNIFORM_BUFFER).with_uniform_format("CloudFaces", TEXEL_BUFFER, GpuFormat::R8_SINT);
    DISSOLVE_MASK_SAMPLER = uniform("DissolveMaskSampler", COMBINED_IMAGE_SAMPLER);
    IN_SAMPLER = uniform("InSampler", COMBINED_IMAGE_SAMPLER);
    LIGHTMAP_INFO = uniform("LightmapInfo", UNIFORM_BUFFER);
    SPRITE_ANIMATION_INFO = uniform("SpriteAnimationInfo", UNIFORM_BUFFER);
    SPRITE = uniform("Sprite", COMBINED_IMAGE_SAMPLER);
    CURRENT_SPRITE_NEXT_SPRITE = uniform("CurrentSprite", COMBINED_IMAGE_SAMPLER).with_uniform("NextSprite", COMBINED_IMAGE_SAMPLER);
    GLINT_SAMPLER = uniform("GlintSampler", COMBINED_IMAGE_SAMPLER);
    DEPTH_BOUNDS_SAMPLER = uniform("DepthBoundsSampler", COMBINED_IMAGE_SAMPLER);
    OIT_COEFFS_DEPTH_BOUNDS_SAMPLER = with_coefficients(uniform("DepthBoundsSampler", COMBINED_IMAGE_SAMPLER));
    SAMPLER0_OIT_COEFFS_DEPTH_BOUNDS_SAMPLER = with_coefficients(
        uniform("Sampler0", COMBINED_IMAGE_SAMPLER).with_uniform("DepthBoundsSampler", COMBINED_IMAGE_SAMPLER),
    );
    SAMPLER0_SAMPLER2_OIT_COEFFS_DEPTH_BOUNDS_SAMPLER = with_coefficients(
        uniform("Sampler0", COMBINED_IMAGE_SAMPLER)
            .with_uniform("Sampler2", COMBINED_IMAGE_SAMPLER)
            .with_uniform("DepthBoundsSampler", COMBINED_IMAGE_SAMPLER),
    );
    CLOUD_INFO_OIT_COEFFS_DEPTH_BOUNDS_SAMPLER = with_coefficients(
        uniform("CloudInfo", UNIFORM_BUFFER)
            .with_uniform_format("CloudFaces", TEXEL_BUFFER, GpuFormat::R8_SINT)
            .with_uniform("DepthBoundsSampler", COMBINED_IMAGE_SAMPLER),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::tests::encode_bind_group_layout;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.client.renderer.BindGroupLayouts");
        for (name, layout) in ALL {
            golden.field(name, encode_bind_group_layout(layout));
        }
        golden.finish();
    }
}
