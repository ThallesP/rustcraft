//! com/mojang/blaze3d/vertex/BufferBuilder.java
//!
//! Writes vertices of a format into a `ByteBufferBuilder`, little-endian like the game's x86
//! builds: each element at its offset, once per vertex, and every element the format has before
//! the next vertex starts. Java writes through the vertex's address; the port keeps its offset.

use crate::com::mojang::blaze3d::vertex::byte_buffer_builder::ByteBufferBuilder;
use crate::com::mojang::blaze3d::vertex::default_vertex_format::DefaultVertexFormat;
use crate::com::mojang::blaze3d::vertex::mesh_data::{DrawState, MeshData};
use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::com::mojang::renderpearl::api::pipeline::index_type::IndexType;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::com::mojang::renderpearl::api::vertex::vertex_format::VertexFormat;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::mth::Mth;

pub struct BufferBuilder<'a> {
    buffer: &'a mut ByteBufferBuilder,
    /// Java's `vertexPointer`: the vertex being written's offset, `None` for `NOT_BUILDING`.
    vertex_pointer: Option<usize>,
    vertices: i32,
    format: &'static VertexFormat,
    primitive_topology: PrimitiveTopology,
    block_format: bool,
    entity_format: bool,
    vertex_size: i32,
    initial_elements_to_fill: i32,
    elements_to_fill: i32,
    building: bool,
    /// Each element's offset in a vertex, if the format has it.
    elements: [Option<i32>; 8],
}

const MAX_VERTEX_COUNT: i32 = 16777215;
const ELEMENT_NAMES: [&str; 8] = [
    "Position",
    "Color",
    "UV0",
    "UV1",
    "UV2",
    "UV3",
    "Normal",
    "LineWidth",
];

fn normal_int_value(c: f32) -> u8 {
    ((Mth::clamp_f32(c, -1.0, 1.0) * 127.0) as i32 & 0xFF) as u8
}

impl<'a> BufferBuilder<'a> {
    /// Panics where Java throws `IllegalArgumentException` for a format without positions.
    pub fn new(
        buffer: &'a mut ByteBufferBuilder,
        primitive_topology: PrimitiveTopology,
        format: &'static VertexFormat,
    ) -> BufferBuilder<'a> {
        assert!(
            format.contains("Position"),
            "Cannot build mesh with no position element"
        );
        let mut elements_mask = 0;
        let mut elements = [None; 8];
        for (i, element_name) in ELEMENT_NAMES.iter().enumerate() {
            let element = format
                .get_element(element_name)
                .map(|element| element.offset);
            if element.is_some() {
                elements_mask |= 1 << i;
            }
            elements[i] = element;
        }
        BufferBuilder {
            buffer,
            vertex_pointer: None,
            vertices: 0,
            format,
            primitive_topology,
            block_format: std::ptr::eq(format, &**DefaultVertexFormat::BLOCK),
            entity_format: std::ptr::eq(format, &**DefaultVertexFormat::ENTITY),
            vertex_size: format.get_vertex_size(),
            initial_elements_to_fill: elements_mask & -2,
            elements_to_fill: 0,
            building: true,
            elements,
        }
    }

    pub fn build(&mut self) -> Option<MeshData> {
        self.ensure_building();
        self.end_last_vertex();
        let mesh = self.store_mesh();
        self.building = false;
        self.vertex_pointer = None;
        mesh
    }

    /// Panics where Java throws `IllegalStateException` for no vertices.
    pub fn build_or_throw(&mut self) -> MeshData {
        self.build().expect("BufferBuilder was empty")
    }

    fn ensure_building(&self) {
        assert!(self.building, "Not building!");
    }

    fn store_mesh(&mut self) -> Option<MeshData> {
        if self.vertices == 0 {
            return None;
        }
        let vertex_buffer = self.buffer.build()?;
        let indices = self.primitive_topology.index_count(self.vertices);
        let index_type = IndexType::least(self.vertices);
        Some(MeshData::new(
            vertex_buffer,
            DrawState {
                format: self.format,
                vertex_count: self.vertices,
                index_count: indices,
                primitive_topology: self.primitive_topology,
                index_type,
            },
        ))
    }

    fn begin_vertex(&mut self) -> usize {
        self.ensure_building();
        self.end_last_vertex();
        assert!(
            self.vertices < MAX_VERTEX_COUNT,
            "Trying to write too many vertices (>16777215) into BufferBuilder"
        );
        self.vertices += 1;
        let pointer = self.buffer.reserve(self.vertex_size);
        self.vertex_pointer = Some(pointer);
        pointer
    }

    /// The element's offset if the vertex still needs it, or `None` (Java's -1).
    fn begin_element(&mut self, semantic_id: usize) -> Option<usize> {
        let old_elements = self.elements_to_fill;
        let new_elements = old_elements & !(1 << semantic_id);
        let offset = self.elements[semantic_id]?;
        if new_elements == old_elements {
            return None;
        }
        self.elements_to_fill = new_elements;
        let vertex_pointer = self.vertex_pointer.expect("Not currently building vertex");
        Some(vertex_pointer + offset as usize)
    }

    /// Panics where Java throws `IllegalStateException` for a vertex missing elements. Java's
    /// message tests each element's index against the mask rather than its bit, which the port
    /// keeps.
    fn end_last_vertex(&mut self) {
        if self.vertices == 0 {
            return;
        }
        if self.elements_to_fill != 0 {
            let missing_elements: Vec<&str> = (0..ELEMENT_NAMES.len())
                .filter(|&i| self.elements_to_fill & i as i32 != 0)
                .map(|i| ELEMENT_NAMES[i])
                .collect();
            panic!(
                "Missing elements in vertex: {}",
                missing_elements.join(", ")
            );
        }
        if self.primitive_topology == PrimitiveTopology::LINES {
            let size = self.vertex_size as usize;
            let pointer = self.buffer.reserve(self.vertex_size);
            let previous = self.buffer.bytes_mut(pointer - size, size).to_vec();
            self.buffer
                .bytes_mut(pointer, size)
                .copy_from_slice(&previous);
            self.vertices += 1;
        }
    }

    fn put(&mut self, pointer: usize, bytes: &[u8]) {
        self.buffer
            .bytes_mut(pointer, bytes.len())
            .copy_from_slice(bytes);
    }

    fn put_rgba(&mut self, pointer: usize, argb: i32) {
        self.put(pointer, &ARGB::to_abgr(argb).to_le_bytes());
    }

    fn put_packed_uv(&mut self, pointer: usize, packed_uv: i32) {
        self.put(pointer, &packed_uv.to_le_bytes());
    }

    fn put_vec3f(&mut self, pointer: usize, x: f32, y: f32, z: f32) {
        self.put(pointer, &x.to_le_bytes());
        self.put(pointer + 4, &y.to_le_bytes());
        self.put(pointer + 8, &z.to_le_bytes());
    }

    fn put_normals(&mut self, pointer: usize, nx: f32, ny: f32, nz: f32) {
        self.put(
            pointer,
            &[
                normal_int_value(nx),
                normal_int_value(ny),
                normal_int_value(nz),
            ],
        );
    }

    fn uv_short(&mut self, u: i16, v: i16, semantic_id: usize) -> &mut dyn VertexConsumer {
        if let Some(pointer) = self.begin_element(semantic_id) {
            self.put(pointer, &u.to_le_bytes());
            self.put(pointer + 2, &v.to_le_bytes());
        }
        self
    }

    fn put_floats(&mut self, semantic_id: usize, values: &[f32]) -> &mut dyn VertexConsumer {
        if let Some(pointer) = self.begin_element(semantic_id) {
            for (i, value) in values.iter().enumerate() {
                self.put(pointer + 4 * i, &value.to_le_bytes());
            }
        }
        self
    }
}

impl VertexConsumer for BufferBuilder<'_> {
    fn add_vertex(&mut self, x: f32, y: f32, z: f32) -> &mut dyn VertexConsumer {
        let position_offset = self.elements[0].expect("a position element");
        let pointer = self.begin_vertex() + position_offset as usize;
        self.elements_to_fill = self.initial_elements_to_fill;
        self.put_vec3f(pointer, x, y, z);
        self
    }

    fn set_color_rgba(&mut self, r: i32, g: i32, b: i32, a: i32) -> &mut dyn VertexConsumer {
        if let Some(pointer) = self.begin_element(1) {
            self.put(pointer, &[r as u8, g as u8, b as u8, a as u8]);
        }
        self
    }

    fn set_color(&mut self, color: i32) -> &mut dyn VertexConsumer {
        if let Some(pointer) = self.begin_element(1) {
            self.put_rgba(pointer, color);
        }
        self
    }

    fn set_uv(&mut self, u: f32, v: f32) -> &mut dyn VertexConsumer {
        self.put_floats(2, &[u, v])
    }

    fn set_uv1(&mut self, u: i32, v: i32) -> &mut dyn VertexConsumer {
        self.uv_short(u as i16, v as i16, 3)
    }

    fn set_overlay(&mut self, packed_overlay_coords: i32) -> &mut dyn VertexConsumer {
        if let Some(pointer) = self.begin_element(3) {
            self.put_packed_uv(pointer, packed_overlay_coords);
        }
        self
    }

    fn set_uv2(&mut self, u: i32, v: i32) -> &mut dyn VertexConsumer {
        self.uv_short(u as i16, v as i16, 4)
    }

    fn set_uv3(&mut self, u: f32, v: f32) -> &mut dyn VertexConsumer {
        self.put_floats(5, &[u, v])
    }

    fn set_light(&mut self, packed_light_coords: i32) -> &mut dyn VertexConsumer {
        if let Some(pointer) = self.begin_element(4) {
            self.put_packed_uv(pointer, packed_light_coords);
        }
        self
    }

    fn set_normal(&mut self, x: f32, y: f32, z: f32) -> &mut dyn VertexConsumer {
        if let Some(pointer) = self.begin_element(6) {
            self.put_normals(pointer, x, y, z);
        }
        self
    }

    fn set_line_width(&mut self, width: f32) -> &mut dyn VertexConsumer {
        self.put_floats(7, &[width])
    }

    fn add_vertex_full(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        color: i32,
        u: f32,
        v: f32,
        overlay_coords: i32,
        light_coords: i32,
        nx: f32,
        ny: f32,
        nz: f32,
    ) {
        if self.block_format {
            let pointer = self.begin_vertex();
            self.put_vec3f(pointer, x, y, z);
            self.put_rgba(pointer + 12, color);
            self.put(pointer + 16, &u.to_le_bytes());
            self.put(pointer + 20, &v.to_le_bytes());
            self.put_packed_uv(pointer + 24, light_coords);
        } else if self.entity_format {
            let pointer = self.begin_vertex();
            self.put_vec3f(pointer, x, y, z);
            self.put_rgba(pointer + 12, color);
            self.put(pointer + 16, &u.to_le_bytes());
            self.put(pointer + 20, &v.to_le_bytes());
            self.put_packed_uv(pointer + 24, overlay_coords);
            self.put_packed_uv(pointer + 28, light_coords);
            self.put_normals(pointer + 32, nx, ny, nz);
        } else {
            self.add_vertex(x, y, z);
            self.set_color(color);
            self.set_uv(u, v);
            self.set_overlay(overlay_coords);
            self.set_light(light_coords);
            self.set_normal(nx, ny, nz);
        }
    }
}
