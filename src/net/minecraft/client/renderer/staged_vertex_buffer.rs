//! net/minecraft/client/renderer/StagedVertexBuffer.java
//!
//! The vertices a frame's features draw, gathered draw by draw, then uploaded into one vertex
//! buffer, with the sorted draws' indices in one index buffer, each draw at its offset. Java
//! writes both through a staging buffer into buffers from pools it recycles once the GPU is done
//! with them (`GpuBufferPool`); here the upload makes buffers with the same bytes, like the
//! port's other ring buffers (PORTING.md, "The frame"). A draw's vertices are each a mesh of
//! their own `BufferBuilder` (Java's share one `ByteBufferBuilder`), appended in the order they
//! were written. Draws that aren't sorted use the shared sequential indices, of quads and lines
//! so far.

use std::rc::Rc;

use crate::com::mojang::blaze3d::systems::render_system::AutoStorageIndexBuffer;
use crate::com::mojang::blaze3d::vertex::buffer_builder::BufferBuilder;
use crate::com::mojang::blaze3d::vertex::byte_buffer_builder::ByteBufferBuilder;
use crate::com::mojang::blaze3d::vertex::compact_vector_array::CompactVectorArray;
use crate::com::mojang::blaze3d::vertex::mesh_data::{MeshData, SortState};
use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::com::mojang::blaze3d::vertex::vertex_sorting::VertexSorting;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::pipeline::index_type::IndexType;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::com::mojang::renderpearl::api::vertex::vertex_format::VertexFormat;
use crate::net::minecraft::client::renderer::feature::feature_frame_context::SequentialBuffers;
use crate::net::minecraft::util::mth::Mth;

/// A draw's index in the buffer's draws: Java's `StagedVertexBuffer.Draw`.
pub type DrawId = usize;

#[derive(Default)]
pub struct StagedVertexBuffer {
    draws: Vec<Draw>,
    current_vertex_buffer: Option<Rc<dyn GpuBuffer>>,
    current_index_buffer: Option<Rc<dyn GpuBuffer>>,
}

impl StagedVertexBuffer {
    pub fn new() -> StagedVertexBuffer {
        StagedVertexBuffer::default()
    }

    /// Panics where Java throws: after the upload, or to sort a draw that isn't quads.
    pub fn append_draw(
        &mut self,
        format: &'static VertexFormat,
        primitive_topology: PrimitiveTopology,
        quad_sorting: Option<VertexSorting>,
    ) -> DrawId {
        assert!(
            self.current_vertex_buffer.is_none(),
            "Cannot append draw after upload"
        );
        assert!(
            quad_sorting.is_none() || primitive_topology == PrimitiveTopology::QUADS,
            "Cannot sort draw with {primitive_topology:?}"
        );
        self.draws.push(Draw {
            format,
            primitive_topology,
            quad_sorting,
            vertex_buffer_slices: Vec::new(),
            vertex_buffer_size: 0,
            vertex_count: 0,
            index_count: 0,
            vertex_offset: 0,
            index_offset: 0,
        });
        self.draws.len() - 1
    }

    /// `getVertexBuilder(draw)`, lent to `build`: what it writes is appended to the draw.
    pub fn with_vertex_builder(
        &mut self,
        draw: DrawId,
        build: impl FnOnce(&mut dyn VertexConsumer),
    ) {
        assert!(
            self.current_vertex_buffer.is_none(),
            "Cannot append draw after upload"
        );
        let draw = &mut self.draws[draw];
        let mut staging_buffer = ByteBufferBuilder::new(INITIAL_BUFFER_SIZE);
        let mut builder =
            BufferBuilder::new(&mut staging_buffer, draw.primitive_topology, draw.format);
        build(&mut builder);
        if let Some(mesh) = builder.build() {
            draw.append(mesh);
        }
    }

    /// Every draw's vertices into one buffer, each at a multiple of its vertex size, and the
    /// sorted draws' indices into another.
    pub fn upload(
        &mut self,
        device: &dyn GpuDevice,
        sequential_buffers: &mut SequentialBuffers<'_>,
    ) {
        assert!(self.current_vertex_buffer.is_none(), "Already uploaded");
        if self.draws.is_empty() {
            return;
        }

        let mut next_vertex_offset = 0;
        let mut next_index_offset = 0;
        for draw in &mut self.draws {
            if !draw.is_empty() {
                draw.vertex_offset =
                    Mth::round_toward_i32(next_vertex_offset, draw.format.get_vertex_size());
                next_vertex_offset = draw.vertex_offset + draw.vertex_buffer_size;
                if draw.quad_sorting.is_some() {
                    let index_type = draw.index_type();
                    draw.index_offset =
                        Mth::round_toward_i32(next_index_offset, index_type.bytes());
                    next_index_offset = draw.index_offset + draw.index_count * index_type.bytes();
                } else {
                    StagedVertexBuffer::sequential_buffer(
                        draw.primitive_topology,
                        sequential_buffers,
                    )
                    .get_buffer(device, draw.index_count);
                }
            }
        }

        let vertex_buffer_size = next_vertex_offset as usize;
        let index_buffer_size = next_index_offset as usize;
        if vertex_buffer_size == 0 {
            return;
        }

        let mut vertices = vec![0u8; vertex_buffer_size];
        let mut indices = vec![0u8; index_buffer_size];
        for draw in &mut self.draws {
            if draw.is_empty() {
                continue;
            }
            let mut offset = draw.vertex_offset as usize;
            for slice in &draw.vertex_buffer_slices {
                vertices[offset..offset + slice.len()].copy_from_slice(slice);
                offset += slice.len();
            }
            if let Some(quad_sorting) = &draw.quad_sorting {
                let sort_state = SortState {
                    centroids: StagedVertexBuffer::decode_sorting_points(draw),
                    index_type: draw.index_type(),
                };
                let sorted = sort_state.write_sorted_index_buffer(quad_sorting);
                let start = draw.index_offset as usize;
                indices[start..start + sorted.len()].copy_from_slice(&sorted);
            }
            draw.vertex_buffer_slices.clear();
        }

        self.current_vertex_buffer = Some(device.create_buffer_with_data(
            Some("Staged Vertex Buffer - Vertex"),
            gpu_buffer::USAGE_VERTEX,
            &vertices,
        ));
        self.current_index_buffer = (index_buffer_size > 0).then(|| {
            device.create_buffer_with_data(
                Some("Staged Vertex Buffer - Index"),
                gpu_buffer::USAGE_INDEX,
                &indices,
            )
        });
    }

    fn decode_sorting_points(draw: &Draw) -> CompactVectorArray {
        let format = draw.format;
        let mut points = CompactVectorArray::new(draw.vertex_count / 4);
        let mut offset = 0;
        for vertex_buffer in &draw.vertex_buffer_slices {
            let vertex_count = vertex_buffer.len() as i32 / format.get_vertex_size();
            MeshData::decode_quad_centroids(
                vertex_buffer,
                vertex_count,
                format,
                &mut points,
                offset,
            );
            offset += (vertex_count / 4) as usize;
        }
        points
    }

    /// Where an uploaded draw's vertices and indices are, or `None` for an empty draw. Panics
    /// where Java throws before the upload.
    pub fn get_execute_info(
        &self,
        draw: DrawId,
        device: &dyn GpuDevice,
        sequential_buffers: &mut SequentialBuffers<'_>,
    ) -> Option<ExecuteInfo> {
        let draw = &self.draws[draw];
        if draw.is_empty() {
            return None;
        }
        let vertex_buffer = self
            .current_vertex_buffer
            .clone()
            .expect("Cannot execute before upload");
        let base_vertex = draw.vertex_offset / draw.format.get_vertex_size();
        if let (Some(index_buffer), Some(_)) = (&self.current_index_buffer, &draw.quad_sorting) {
            let index_type = draw.index_type();
            return Some(ExecuteInfo {
                vertex_buffer,
                index_buffer: Rc::clone(index_buffer),
                index_type,
                base_vertex,
                first_index: draw.index_offset / index_type.bytes(),
                index_count: draw.index_count,
            });
        }
        let auto_indices =
            StagedVertexBuffer::sequential_buffer(draw.primitive_topology, sequential_buffers);
        auto_indices.request_index_count(draw.index_count);
        Some(ExecuteInfo {
            vertex_buffer,
            index_buffer: auto_indices.get_buffer(device, draw.index_count),
            index_type: auto_indices.index_type(),
            base_vertex,
            first_index: 0,
            index_count: draw.index_count,
        })
    }

    pub fn request_index_count(
        &self,
        draw: DrawId,
        sequential_buffers: &mut SequentialBuffers<'_>,
    ) {
        let draw = &self.draws[draw];
        if self.current_index_buffer.is_none() || draw.quad_sorting.is_none() {
            StagedVertexBuffer::sequential_buffer(draw.primitive_topology, sequential_buffers)
                .request_index_count(draw.index_count);
        }
    }

    /// `RenderSystem.getSequentialBuffer`: the game renderer's quads' or lines' indices, the
    /// kinds its features draw so far.
    fn sequential_buffer<'a>(
        primitive_topology: PrimitiveTopology,
        sequential_buffers: &'a mut SequentialBuffers<'_>,
    ) -> &'a mut AutoStorageIndexBuffer {
        match primitive_topology {
            PrimitiveTopology::QUADS => sequential_buffers.quads,
            PrimitiveTopology::LINES => sequential_buffers.lines,
            _ => panic!("the sequential indices of {primitive_topology:?} aren't ported"),
        }
    }

    pub fn end_draw(&mut self) {
        self.draws.clear();
        self.current_vertex_buffer = None;
        self.current_index_buffer = None;
    }
}

/// The initial capacity of each draw's builder: `RenderType.TRANSIENT_BUFFER_SIZE`.
const INITIAL_BUFFER_SIZE: i32 = 1536;

/// `StagedVertexBuffer.Draw`.
pub struct Draw {
    format: &'static VertexFormat,
    primitive_topology: PrimitiveTopology,
    quad_sorting: Option<VertexSorting>,
    vertex_buffer_slices: Vec<Vec<u8>>,
    vertex_buffer_size: i32,
    vertex_count: i32,
    index_count: i32,
    vertex_offset: i32,
    index_offset: i32,
}

impl Draw {
    fn append(&mut self, mesh: MeshData) {
        assert!(mesh.index_buffer().is_none());
        self.vertex_buffer_size += mesh.vertex_buffer().len() as i32;
        self.vertex_count += mesh.draw_state().vertex_count;
        self.index_count += mesh.draw_state().index_count;
        self.vertex_buffer_slices
            .push(mesh.vertex_buffer().to_vec());
    }

    fn index_type(&self) -> IndexType {
        IndexType::least(self.vertex_count)
    }

    pub fn is_empty(&self) -> bool {
        self.vertex_count == 0
    }
}

/// `StagedVertexBuffer.ExecuteInfo`, its index buffer the sequential one where the draw has none
/// of its own (`indexBuffer()`).
pub struct ExecuteInfo {
    pub vertex_buffer: Rc<dyn GpuBuffer>,
    pub index_buffer: Rc<dyn GpuBuffer>,
    pub index_type: IndexType,
    pub base_vertex: i32,
    pub first_index: i32,
    pub index_count: i32,
}
