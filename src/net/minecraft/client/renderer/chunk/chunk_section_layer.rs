//! net/minecraft/client/renderer/chunk/ChunkSectionLayer.java
//!
//! The layers a chunk section's mesh is split into, by how a quad's texture lets light through.

use std::sync::Arc;

use crate::com::mojang::blaze3d::platform::transparency::Transparency;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::com::mojang::renderpearl::api::vertex::vertex_format::VertexFormat;
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ChunkSectionLayer {
    SOLID,
    CUTOUT,
    TRANSLUCENT,
}

impl ChunkSectionLayer {
    pub const VALUES: [ChunkSectionLayer; 3] = [
        ChunkSectionLayer::SOLID,
        ChunkSectionLayer::CUTOUT,
        ChunkSectionLayer::TRANSLUCENT,
    ];

    pub fn by_transparency(transparency: Transparency) -> ChunkSectionLayer {
        if transparency.has_translucent {
            ChunkSectionLayer::TRANSLUCENT
        } else if transparency.has_transparent {
            ChunkSectionLayer::CUTOUT
        } else {
            ChunkSectionLayer::SOLID
        }
    }

    pub fn pipeline(&self, multi_draw: bool) -> &'static Arc<RenderPipeline> {
        let pipelines = RenderPipelines::get();
        match (self, multi_draw) {
            (ChunkSectionLayer::SOLID, false) => &pipelines.SOLID_TERRAIN,
            (ChunkSectionLayer::SOLID, true) => &pipelines.SOLID_TERRAIN_MULTIDRAW,
            (ChunkSectionLayer::CUTOUT, false) => &pipelines.CUTOUT_TERRAIN,
            (ChunkSectionLayer::CUTOUT, true) => &pipelines.CUTOUT_TERRAIN_MULTIDRAW,
            (ChunkSectionLayer::TRANSLUCENT, false) => &pipelines.TRANSLUCENT_TERRAIN,
            (ChunkSectionLayer::TRANSLUCENT, true) => &pipelines.TRANSLUCENT_TERRAIN_MULTIDRAW,
        }
    }

    pub fn buffer_size(&self) -> i32 {
        match self {
            ChunkSectionLayer::SOLID | ChunkSectionLayer::CUTOUT => 4194304,
            ChunkSectionLayer::TRANSLUCENT => 786432,
        }
    }

    /// The constant's name in lower case.
    pub fn label(&self) -> &'static str {
        match self {
            ChunkSectionLayer::SOLID => "solid",
            ChunkSectionLayer::CUTOUT => "cutout",
            ChunkSectionLayer::TRANSLUCENT => "translucent",
        }
    }

    pub fn translucent(&self) -> bool {
        *self == ChunkSectionLayer::TRANSLUCENT
    }

    pub fn vertex_format(&self) -> &'static VertexFormat {
        self.pipeline(false)
            .get_vertex_format_binding(0)
            .expect("terrain pipelines have a vertex buffer")
    }
}
