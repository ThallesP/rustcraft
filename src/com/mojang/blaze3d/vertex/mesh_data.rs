//! com/mojang/blaze3d/vertex/MeshData.java
//!
//! A built mesh: its vertex bytes (a copy, see `ByteBufferBuilder`) and how to draw them. Sorting
//! quads (`sortQuads`, `decodeQuadCentroids`) and index buffers come with translucent terrain.

use crate::com::mojang::renderpearl::api::pipeline::index_type::IndexType;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::com::mojang::renderpearl::api::vertex::vertex_format::VertexFormat;

pub struct MeshData {
    vertex_buffer: Vec<u8>,
    draw_state: DrawState,
}

/// `MeshData.DrawState`.
#[derive(Clone, Copy, Debug)]
pub struct DrawState {
    pub format: &'static VertexFormat,
    pub vertex_count: i32,
    pub index_count: i32,
    pub primitive_topology: PrimitiveTopology,
    pub index_type: IndexType,
}

impl MeshData {
    pub fn new(vertex_buffer: Vec<u8>, draw_state: DrawState) -> MeshData {
        MeshData {
            vertex_buffer,
            draw_state,
        }
    }

    pub fn vertex_buffer(&self) -> &[u8] {
        &self.vertex_buffer
    }

    pub fn draw_state(&self) -> &DrawState {
        &self.draw_state
    }
}
