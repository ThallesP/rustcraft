//! com/mojang/blaze3d/vertex/DefaultVertexFormat.java

use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::vertex::vertex_format::VertexFormat;
use std::sync::LazyLock;

pub struct DefaultVertexFormat;

const POSITION_FORMAT: GpuFormat = GpuFormat::RGB32_FLOAT;
const COLOR_FORMAT: GpuFormat = GpuFormat::RGBA8_UNORM;
const UV0_FORMAT: GpuFormat = GpuFormat::RG32_FLOAT;
const UV1_FORMAT: GpuFormat = GpuFormat::RG16_SINT;
const UV2_FORMAT: GpuFormat = GpuFormat::RG16_SINT;
const UV3_FORMAT: GpuFormat = GpuFormat::RG32_FLOAT;
const NORMAL_FORMAT: GpuFormat = GpuFormat::RGBA8_SNORM;
const LINE_WIDTH_FORMAT: GpuFormat = GpuFormat::R32_FLOAT;
const CHUNK_POSITION_FORMAT: GpuFormat = GpuFormat::RGB32_SINT;
const CHUNK_VISIBILITY_FORMAT: GpuFormat = GpuFormat::R32_FLOAT;

type StaticVertexFormat = LazyLock<VertexFormat>;

static BLOCK: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(0)
        .add_attribute("Position", POSITION_FORMAT)
        .add_attribute("Color", COLOR_FORMAT)
        .add_attribute("UV0", UV0_FORMAT)
        .add_attribute("UV2", UV2_FORMAT)
        .build()
});
static CHUNK_DATA_INSTANCED: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(1)
        .add_attribute("ChunkPosition", CHUNK_POSITION_FORMAT)
        .add_attribute("ChunkVisibility", CHUNK_VISIBILITY_FORMAT)
        .build()
});
static ENTITY: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(0)
        .add_attribute("Position", POSITION_FORMAT)
        .add_attribute("Color", COLOR_FORMAT)
        .add_attribute("UV0", UV0_FORMAT)
        .add_attribute("UV1", UV1_FORMAT)
        .add_attribute("UV2", UV2_FORMAT)
        .add_attribute("Normal", NORMAL_FORMAT)
        .build()
});
static ENTITY_GLINT_SPECIAL: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(0)
        .add_attribute("Position", POSITION_FORMAT)
        .add_attribute("Color", COLOR_FORMAT)
        .add_attribute("UV0", UV0_FORMAT)
        .add_attribute("UV1", UV1_FORMAT)
        .add_attribute("UV2", UV2_FORMAT)
        .add_attribute("UV3", UV3_FORMAT)
        .add_attribute("Normal", NORMAL_FORMAT)
        .build()
});
static PARTICLE: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(0)
        .add_attribute("Position", POSITION_FORMAT)
        .add_attribute("UV0", UV0_FORMAT)
        .add_attribute("Color", COLOR_FORMAT)
        .add_attribute("UV2", UV2_FORMAT)
        .build()
});
static POSITION: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(0)
        .add_attribute("Position", POSITION_FORMAT)
        .build()
});
static POSITION_COLOR: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(0)
        .add_attribute("Position", POSITION_FORMAT)
        .add_attribute("Color", COLOR_FORMAT)
        .build()
});
static POSITION_COLOR_NORMAL: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(0)
        .add_attribute("Position", POSITION_FORMAT)
        .add_attribute("Color", COLOR_FORMAT)
        .add_attribute("Normal", NORMAL_FORMAT)
        .build()
});
static POSITION_COLOR_LIGHTMAP: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(0)
        .add_attribute("Position", POSITION_FORMAT)
        .add_attribute("Color", COLOR_FORMAT)
        .add_attribute("UV2", UV2_FORMAT)
        .build()
});
static POSITION_TEX: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(0)
        .add_attribute("Position", POSITION_FORMAT)
        .add_attribute("UV0", UV0_FORMAT)
        .build()
});
static POSITION_TEX_COLOR: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(0)
        .add_attribute("Position", POSITION_FORMAT)
        .add_attribute("UV0", UV0_FORMAT)
        .add_attribute("Color", COLOR_FORMAT)
        .build()
});
static POSITION_COLOR_TEX_LIGHTMAP: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(0)
        .add_attribute("Position", POSITION_FORMAT)
        .add_attribute("Color", COLOR_FORMAT)
        .add_attribute("UV0", UV0_FORMAT)
        .add_attribute("UV2", UV2_FORMAT)
        .build()
});
static POSITION_TEX_LIGHTMAP_COLOR: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(0)
        .add_attribute("Position", POSITION_FORMAT)
        .add_attribute("UV0", UV0_FORMAT)
        .add_attribute("UV2", UV2_FORMAT)
        .add_attribute("Color", COLOR_FORMAT)
        .build()
});
static POSITION_TEX_COLOR_NORMAL: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(0)
        .add_attribute("Position", POSITION_FORMAT)
        .add_attribute("UV0", UV0_FORMAT)
        .add_attribute("Color", COLOR_FORMAT)
        .add_attribute("Normal", NORMAL_FORMAT)
        .build()
});
static POSITION_COLOR_LINE_WIDTH: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(0)
        .add_attribute("Position", POSITION_FORMAT)
        .add_attribute("Color", COLOR_FORMAT)
        .add_attribute("LineWidth", LINE_WIDTH_FORMAT)
        .build()
});
static POSITION_COLOR_NORMAL_LINE_WIDTH: StaticVertexFormat = LazyLock::new(|| {
    VertexFormat::builder(0)
        .add_attribute("Position", POSITION_FORMAT)
        .add_attribute("Color", COLOR_FORMAT)
        .add_attribute("Normal", NORMAL_FORMAT)
        .add_attribute("LineWidth", LINE_WIDTH_FORMAT)
        .build()
});

impl DefaultVertexFormat {
    pub const POSITION_SEMANTIC_NAME: &str = "Position";
    pub const COLOR_SEMANTIC_NAME: &str = "Color";
    pub const UV0_SEMANTIC_NAME: &str = "UV0";
    pub const UV1_SEMANTIC_NAME: &str = "UV1";
    pub const UV2_SEMANTIC_NAME: &str = "UV2";
    pub const UV3_SEMANTIC_NAME: &str = "UV3";
    pub const NORMAL_SEMANTIC_NAME: &str = "Normal";
    pub const LINE_WIDTH_SEMANTIC_NAME: &str = "LineWidth";
    pub const CHUNK_POSITION_SEMANTIC_NAME: &str = "ChunkPosition";
    pub const CHUNK_VISIBILITY_SEMANTIC_NAME: &str = "ChunkVisibility";
    pub const BLOCK: &StaticVertexFormat = &BLOCK;
    pub const CHUNK_DATA_INSTANCED: &StaticVertexFormat = &CHUNK_DATA_INSTANCED;
    pub const ENTITY: &StaticVertexFormat = &ENTITY;
    pub const ENTITY_GLINT_SPECIAL: &StaticVertexFormat = &ENTITY_GLINT_SPECIAL;
    pub const PARTICLE: &StaticVertexFormat = &PARTICLE;
    pub const POSITION: &StaticVertexFormat = &POSITION;
    pub const POSITION_COLOR: &StaticVertexFormat = &POSITION_COLOR;
    pub const POSITION_COLOR_NORMAL: &StaticVertexFormat = &POSITION_COLOR_NORMAL;
    pub const POSITION_COLOR_LIGHTMAP: &StaticVertexFormat = &POSITION_COLOR_LIGHTMAP;
    pub const POSITION_TEX: &StaticVertexFormat = &POSITION_TEX;
    pub const POSITION_TEX_COLOR: &StaticVertexFormat = &POSITION_TEX_COLOR;
    pub const POSITION_COLOR_TEX_LIGHTMAP: &StaticVertexFormat = &POSITION_COLOR_TEX_LIGHTMAP;
    pub const POSITION_TEX_LIGHTMAP_COLOR: &StaticVertexFormat = &POSITION_TEX_LIGHTMAP_COLOR;
    pub const POSITION_TEX_COLOR_NORMAL: &StaticVertexFormat = &POSITION_TEX_COLOR_NORMAL;
    pub const POSITION_COLOR_LINE_WIDTH: &StaticVertexFormat = &POSITION_COLOR_LINE_WIDTH;
    pub const POSITION_COLOR_NORMAL_LINE_WIDTH: &StaticVertexFormat =
        &POSITION_COLOR_NORMAL_LINE_WIDTH;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::tests::encode_vertex_format;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.blaze3d.vertex.DefaultVertexFormat");
        for (name, value) in [
            (
                "POSITION_SEMANTIC_NAME",
                DefaultVertexFormat::POSITION_SEMANTIC_NAME,
            ),
            (
                "COLOR_SEMANTIC_NAME",
                DefaultVertexFormat::COLOR_SEMANTIC_NAME,
            ),
            ("UV0_SEMANTIC_NAME", DefaultVertexFormat::UV0_SEMANTIC_NAME),
            ("UV1_SEMANTIC_NAME", DefaultVertexFormat::UV1_SEMANTIC_NAME),
            ("UV2_SEMANTIC_NAME", DefaultVertexFormat::UV2_SEMANTIC_NAME),
            ("UV3_SEMANTIC_NAME", DefaultVertexFormat::UV3_SEMANTIC_NAME),
            (
                "NORMAL_SEMANTIC_NAME",
                DefaultVertexFormat::NORMAL_SEMANTIC_NAME,
            ),
            (
                "LINE_WIDTH_SEMANTIC_NAME",
                DefaultVertexFormat::LINE_WIDTH_SEMANTIC_NAME,
            ),
            (
                "CHUNK_POSITION_SEMANTIC_NAME",
                DefaultVertexFormat::CHUNK_POSITION_SEMANTIC_NAME,
            ),
            (
                "CHUNK_VISIBILITY_SEMANTIC_NAME",
                DefaultVertexFormat::CHUNK_VISIBILITY_SEMANTIC_NAME,
            ),
        ] {
            golden.field(name, value.to_owned());
        }
        for (name, format) in [
            ("BLOCK", DefaultVertexFormat::BLOCK),
            (
                "CHUNK_DATA_INSTANCED",
                DefaultVertexFormat::CHUNK_DATA_INSTANCED,
            ),
            ("ENTITY", DefaultVertexFormat::ENTITY),
            (
                "ENTITY_GLINT_SPECIAL",
                DefaultVertexFormat::ENTITY_GLINT_SPECIAL,
            ),
            ("PARTICLE", DefaultVertexFormat::PARTICLE),
            ("POSITION", DefaultVertexFormat::POSITION),
            ("POSITION_COLOR", DefaultVertexFormat::POSITION_COLOR),
            (
                "POSITION_COLOR_NORMAL",
                DefaultVertexFormat::POSITION_COLOR_NORMAL,
            ),
            (
                "POSITION_COLOR_LIGHTMAP",
                DefaultVertexFormat::POSITION_COLOR_LIGHTMAP,
            ),
            ("POSITION_TEX", DefaultVertexFormat::POSITION_TEX),
            (
                "POSITION_TEX_COLOR",
                DefaultVertexFormat::POSITION_TEX_COLOR,
            ),
            (
                "POSITION_COLOR_TEX_LIGHTMAP",
                DefaultVertexFormat::POSITION_COLOR_TEX_LIGHTMAP,
            ),
            (
                "POSITION_TEX_LIGHTMAP_COLOR",
                DefaultVertexFormat::POSITION_TEX_LIGHTMAP_COLOR,
            ),
            (
                "POSITION_TEX_COLOR_NORMAL",
                DefaultVertexFormat::POSITION_TEX_COLOR_NORMAL,
            ),
            (
                "POSITION_COLOR_LINE_WIDTH",
                DefaultVertexFormat::POSITION_COLOR_LINE_WIDTH,
            ),
            (
                "POSITION_COLOR_NORMAL_LINE_WIDTH",
                DefaultVertexFormat::POSITION_COLOR_NORMAL_LINE_WIDTH,
            ),
        ] {
            golden.field(name, encode_vertex_format(format));
        }
        golden.finish();
    }
}
