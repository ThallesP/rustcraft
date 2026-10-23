//! com/mojang/blaze3d/vertex/MeshData.java
//!
//! A built mesh: its vertex bytes (a copy, see `ByteBufferBuilder`), its sorted index bytes once
//! its quads are sorted, and how to draw them. Buffers are little-endian, like LWJGL's native ones
//! on x86.

use crate::com::mojang::blaze3d::vertex::byte_buffer_builder::ByteBufferBuilder;
use crate::com::mojang::blaze3d::vertex::compact_vector_array::CompactVectorArray;
use crate::com::mojang::blaze3d::vertex::vertex_sorting::VertexSorting;
use crate::com::mojang::renderpearl::api::pipeline::index_type::IndexType;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::com::mojang::renderpearl::api::vertex::vertex_format::VertexFormat;

pub struct MeshData {
    vertex_buffer: Vec<u8>,
    index_buffer: Option<Vec<u8>>,
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
            index_buffer: None,
            draw_state,
        }
    }

    /// Each quad's centroid, the middle of its first and third vertices, into `output` from
    /// `output_index` on. Panics where Java throws for a format without positions.
    pub fn decode_quad_centroids(
        vertex_buffer: &[u8],
        vertex_count: i32,
        format: &VertexFormat,
        output: &mut CompactVectorArray,
        output_index: usize,
    ) {
        let position_element = format
            .get_element("Position")
            .expect("Cannot identify quad centers with no position element");
        let position_offset = position_element.offset as usize;
        let vertex_stride = format.get_vertex_size() as usize;
        let quad_stride = vertex_stride * 4;
        let quad_count = (vertex_count / 4) as usize;
        let float = |offset: usize| {
            f32::from_le_bytes(
                vertex_buffer[offset..offset + 4]
                    .try_into()
                    .expect("four bytes"),
            )
        };
        for i in 0..quad_count {
            let first_pos_offset = i * quad_stride + position_offset;
            let second_pos_offset = first_pos_offset + vertex_stride * 2;
            let x0 = float(first_pos_offset);
            let y0 = float(first_pos_offset + 4);
            let z0 = float(first_pos_offset + 8);
            let x1 = float(second_pos_offset);
            let y1 = float(second_pos_offset + 4);
            let z1 = float(second_pos_offset + 8);
            let x_mid = (x0 + x1) / 2.0;
            let y_mid = (y0 + y1) / 2.0;
            let z_mid = (z0 + z1) / 2.0;
            output.set(output_index + i, x_mid, y_mid, z_mid);
        }
    }

    pub fn vertex_buffer(&self) -> &[u8] {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> Option<&[u8]> {
        self.index_buffer.as_deref()
    }

    pub fn draw_state(&self) -> &DrawState {
        &self.draw_state
    }

    /// Sorts the quads by their centroids into an index buffer built out of
    /// `index_buffer_target`, and returns what a later sort needs, or `None` for a mesh that
    /// isn't quads.
    pub fn sort_quads(
        &mut self,
        index_buffer_target: &mut ByteBufferBuilder,
        sorting: &VertexSorting,
    ) -> Option<SortState> {
        if self.draw_state.primitive_topology != PrimitiveTopology::QUADS {
            return None;
        }

        let mut centroids = CompactVectorArray::new(self.draw_state.vertex_count / 4);
        MeshData::decode_quad_centroids(
            &self.vertex_buffer,
            self.draw_state.vertex_count,
            self.draw_state.format,
            &mut centroids,
            0,
        );
        let sort_state = SortState {
            centroids,
            index_type: self.draw_state.index_type,
        };
        self.index_buffer = sort_state.build_sorted_index_buffer(index_buffer_target, sorting);
        Some(sort_state)
    }
}

/// `MeshData.SortState`: the quads' centroids, to sort them again from another point of view.
pub struct SortState {
    pub centroids: CompactVectorArray,
    pub index_type: IndexType,
}

impl SortState {
    pub fn build_sorted_index_buffer(
        &self,
        target: &mut ByteBufferBuilder,
        sorting: &VertexSorting,
    ) -> Option<Vec<u8>> {
        let start_indices = sorting.sort(&self.centroids);
        let bytes = self.index_type.bytes() as usize;
        let size = start_indices.len() * 6 * bytes;
        let offset = target.reserve(size as i32);
        let index_type = self.index_type;
        let output = target.bytes_mut(offset, size);
        let mut next = 0;
        SortState::write_indices(&start_indices, |value| {
            match index_type {
                IndexType::SHORT => {
                    output[next..next + 2].copy_from_slice(&(value as i16).to_le_bytes())
                }
                IndexType::INT => output[next..next + 4].copy_from_slice(&value.to_le_bytes()),
            }
            next += bytes;
        });
        target.build()
    }

    fn write_indices(start_indices: &[i32], mut output: impl FnMut(i32)) {
        for &start_index in start_indices {
            output(start_index * 4);
            output(start_index * 4 + 1);
            output(start_index * 4 + 2);
            output(start_index * 4 + 2);
            output(start_index * 4 + 3);
            output(start_index * 4);
        }
    }
}
