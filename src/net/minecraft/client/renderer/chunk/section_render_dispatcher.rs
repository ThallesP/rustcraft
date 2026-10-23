//! net/minecraft/client/renderer/chunk/SectionRenderDispatcher.java
//!
//! Compiles sections and puts their meshes in the GPU buffers the terrain draws from. The port
//! compiles synchronously, on the render thread, which only changes when a mesh is ready: the
//! oracle waits for every section to be compiled before it takes a screenshot. Not ported yet:
//! the background executor and its queue (`SectionTaskDynamicQueue`, `SectionBufferBuilderPool`),
//! translucency re-sorting (`ResortTransparencyTask`), and Java's buffers behind the uber buffers:
//! `UberGpuBuffer`'s `TlsfAllocator` heaps and the `StagingBuffer` they're copied through. Here
//! each layer's sections lie one after another in one vertex buffer and one index buffer, rebuilt
//! when a section's mesh comes or goes: the draws read the same vertices and indices from other
//! offsets.

use std::rc::Rc;

use crate::com::mojang::blaze3d::vertex::vertex_sorting::VertexSorting;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::{self, GpuBuffer};
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::net::minecraft::client::renderer::chunk::chunk_section_layer::ChunkSectionLayer;
use crate::net::minecraft::client::renderer::chunk::compiled_section_mesh::CompiledSectionMesh;
use crate::net::minecraft::client::renderer::chunk::render_section_region::RenderSectionRegion;
use crate::net::minecraft::client::renderer::chunk::section_compiler::SectionCompiler;
use crate::net::minecraft::client::renderer::chunk::section_mesh::SectionMesh;
use crate::net::minecraft::client::renderer::chunk::translucency_point_of_view::TranslucencyPointOfView;
use crate::net::minecraft::client::renderer::section_buffer_builder_pack::SectionBufferBuilderPack;
use crate::net::minecraft::client::rotating_section_storage::{RotatingSectionStorage, Value};
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::util::util::Util;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::vec3::Vec3;

pub struct SectionRenderDispatcher {
    fixed_buffers: SectionBufferBuilderPack,
    camera_position: Vec3,
    section_compiler: SectionCompiler,
    chunk_uber_buffers: [SectionUberBuffers; 3],
    /// Meshes whose buffers wait for the next upload, with their section's index in the view
    /// area and its section then: Java's upload callbacks (`vertexBufferUploadCallback`,
    /// `checkSectionMesh`).
    waiting_for_upload: Vec<(usize, i64, Rc<CompiledSectionMesh>)>,
}

/// `RenderSectionBufferSlice`: where a section's layer lies in the layer's buffers.
#[derive(Clone)]
pub struct RenderSectionBufferSlice {
    pub vertex_buffer: Rc<dyn GpuBuffer>,
    pub vertex_buffer_offset: i64,
    pub index_buffer: Option<Rc<dyn GpuBuffer>>,
    pub index_buffer_offset: i64,
}

/// A layer's buffers: each mesh's vertices and sorted indices, in the order they came.
#[derive(Default)]
struct SectionUberBuffers {
    allocations: Vec<Allocation>,
    vertex_buffer: Option<Rc<dyn GpuBuffer>>,
    index_buffer: Option<Rc<dyn GpuBuffer>>,
    dirty: bool,
}

struct Allocation {
    mesh: Rc<CompiledSectionMesh>,
    vertices: Vec<u8>,
    indices: Option<Vec<u8>>,
    vertex_offset: i64,
    index_offset: i64,
}

impl SectionUberBuffers {
    fn get_allocation(&self, mesh: &Rc<CompiledSectionMesh>) -> Option<&Allocation> {
        self.allocations
            .iter()
            .find(|allocation| Rc::ptr_eq(&allocation.mesh, mesh))
    }

    fn remove_allocation(&mut self, mesh: &Rc<CompiledSectionMesh>) {
        let before = self.allocations.len();
        self.allocations
            .retain(|allocation| !Rc::ptr_eq(&allocation.mesh, mesh));
        self.dirty |= self.allocations.len() != before;
    }

    /// Packs the allocations into new buffers when they changed.
    fn upload(&mut self, device: &dyn GpuDevice, layer: ChunkSectionLayer) {
        if !self.dirty {
            return;
        }

        self.dirty = false;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for allocation in &mut self.allocations {
            allocation.vertex_offset = vertices.len() as i64;
            vertices.extend_from_slice(&allocation.vertices);
            if let Some(allocation_indices) = &allocation.indices {
                allocation.index_offset = indices.len() as i64;
                indices.extend_from_slice(allocation_indices);
            }
        }

        let label = layer.label();
        self.vertex_buffer = (!vertices.is_empty()).then(|| {
            device.create_buffer_with_data(Some(label), gpu_buffer::USAGE_VERTEX, &vertices)
        });
        self.index_buffer = (!indices.is_empty()).then(|| {
            device.create_buffer_with_data(Some(label), gpu_buffer::USAGE_INDEX, &indices)
        });
        for allocation in &self.allocations {
            allocation.mesh.set_vertex_buffer_uploaded(layer);
            allocation.mesh.set_index_buffer_uploaded(layer);
        }
    }
}

impl SectionRenderDispatcher {
    pub const NEARBY_SECTION_DISTANCE_IN_BLOCKS: i32 = 32;

    pub fn new(section_compiler: SectionCompiler) -> SectionRenderDispatcher {
        SectionRenderDispatcher {
            fixed_buffers: SectionBufferBuilderPack::new(),
            camera_position: Vec3::ZERO,
            section_compiler,
            chunk_uber_buffers: Default::default(),
            waiting_for_upload: Vec::new(),
        }
    }

    pub fn set_compiler(&mut self, section_compiler: SectionCompiler) {
        self.section_compiler = section_compiler;
    }

    pub fn set_camera_position(&mut self, camera_position: Vec3) {
        self.camera_position = camera_position;
    }

    pub fn get_render_section_slice(
        &self,
        section_mesh: &SectionMesh,
        layer: ChunkSectionLayer,
    ) -> Option<RenderSectionBufferSlice> {
        let SectionMesh::Compiled(mesh) = section_mesh else {
            return None;
        };
        let uber_buffers = &self.chunk_uber_buffers[layer as usize];
        let allocation = uber_buffers.get_allocation(mesh)?;
        let vertex_buffer = uber_buffers.vertex_buffer.clone()?;
        let index_buffer = allocation
            .indices
            .as_ref()
            .and(uber_buffers.index_buffer.clone());
        Some(RenderSectionBufferSlice {
            vertex_buffer,
            vertex_buffer_offset: allocation.vertex_offset,
            index_buffer_offset: if index_buffer.is_some() {
                allocation.index_offset
            } else {
                0
            },
            index_buffer,
        })
    }

    /// Puts the meshes compiled since the last upload into the layers' buffers, and the
    /// sections whose buffers are all there get their meshes (`checkSectionMesh`). Returns the
    /// indices of those sections, whose meshes changed (`onSectionMeshUpdate`). A mesh nothing
    /// holds any more (its section was reset or moved) leaves the buffers.
    pub fn upload_terrain_buffers_to_gpu(
        &mut self,
        device: &dyn GpuDevice,
        sections: &mut RotatingSectionStorage<RenderSection>,
    ) -> Vec<usize> {
        for buffers in &mut self.chunk_uber_buffers {
            let before = buffers.allocations.len();
            buffers
                .allocations
                .retain(|allocation| Rc::strong_count(&allocation.mesh) > 1);
            buffers.dirty |= buffers.allocations.len() != before;
        }
        for layer in ChunkSectionLayer::VALUES {
            self.chunk_uber_buffers[layer as usize].upload(device, layer);
        }

        let mut updated = Vec::new();
        for (index, section_node, mesh) in std::mem::take(&mut self.waiting_for_upload) {
            let all_buffers_updated = ChunkSectionLayer::VALUES.iter().all(|&layer| {
                mesh.get_section_draw(layer).is_none()
                    || mesh.is_index_buffer_uploaded(layer) && mesh.is_vertex_buffer_uploaded(layer)
            });
            let section = sections.get_mut(index);
            if section.section_node != section_node {
                continue;
            }
            let compiled = SectionMesh::Compiled(mesh);
            if all_buffers_updated && !section.section_mesh.same(&compiled) {
                section.set_section_mesh(compiled);
                updated.push(index);
            }
        }
        updated
    }

    fn release_section_mesh(&mut self, old_mesh: &SectionMesh) {
        if let SectionMesh::Compiled(mesh) = old_mesh {
            for buffers in &mut self.chunk_uber_buffers {
                buffers.remove_allocation(mesh);
            }
        }
    }

    /// `RenderSection.compileSync`: the compile task, run now with the fixed buffers. A section
    /// with nothing to draw gets the empty mesh at once (its index is returned, its mesh having
    /// changed); one with layers waits for the next upload.
    pub fn compile_sync(
        &mut self,
        section: &mut RenderSection,
        region: &RenderSectionRegion,
    ) -> bool {
        let start_time_ns = Util::get_nanos();
        let section_node = section.section_node;
        let section_pos = SectionPos::of_long(section_node);
        let camera_pos = self.camera_position;
        let results = self.section_compiler.compile(
            section_pos,
            region,
            &RenderSection::create_vertex_sorting(&section_pos, &camera_pos),
            &mut self.fixed_buffers,
        );
        let translucency_point_of_view = TranslucencyPointOfView::of(&camera_pos, section_node);
        if results.rendered_layers.is_empty() {
            let old_mesh = section.set_section_mesh(SectionMesh::Empty);
            self.release_section_mesh(&old_mesh);
            return true;
        }

        let compiled_section_mesh = Rc::new(CompiledSectionMesh::new(
            translucency_point_of_view,
            results,
            start_time_ns,
        ));
        for (layer, vertices, indices) in compiled_section_mesh.pending_buffers.take() {
            let buffers = &mut self.chunk_uber_buffers[layer as usize];
            buffers.allocations.push(Allocation {
                mesh: Rc::clone(&compiled_section_mesh),
                vertices,
                indices,
                vertex_offset: 0,
                index_offset: 0,
            });
            buffers.dirty = true;
        }
        self.waiting_for_upload
            .retain(|(index, _, _)| *index != section.index);
        self.waiting_for_upload
            .push((section.index, section_node, compiled_section_mesh));
        false
    }

    /// `RenderSection.reset`'s release of its mesh's buffers.
    pub fn reset_section(&mut self, section: &mut RenderSection) {
        let old_mesh = section.set_section_mesh(SectionMesh::Uncompiled);
        self.release_section_mesh(&old_mesh);
        section.uploaded_time = 0;
        self.waiting_for_upload
            .retain(|(index, _, _)| *index != section.index);
    }

    pub fn is_queue_empty(&self) -> bool {
        self.waiting_for_upload.is_empty()
    }
}

/// `SectionRenderDispatcher.RenderSection`: a section of the view area, its mesh and where it is.
pub struct RenderSection {
    pub index: usize,
    section_mesh: SectionMesh,
    bb: AABB,
    section_node: i64,
    render_origin: BlockPos,
    uploaded_time: i64,
}

impl RenderSection {
    pub fn new(index: usize, section_node: i64) -> RenderSection {
        let mut section = RenderSection {
            index,
            section_mesh: SectionMesh::Uncompiled,
            bb: AABB::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            section_node: SectionPos::as_long_xyz(-1, -1, -1),
            render_origin: BlockPos::new(-1, -1, -1),
            uploaded_time: 0,
        };
        section.set_section_node(section_node);
        section
    }

    pub fn get_visibility(&self, now: i64, fade_duration: i64) -> f32 {
        let elapsed = now - self.uploaded_time;
        if elapsed >= fade_duration {
            1.0
        } else {
            elapsed as f32 / fade_duration as f32
        }
    }

    pub fn get_bounding_box(&self) -> &AABB {
        &self.bb
    }

    pub fn section_node(&self) -> i64 {
        self.section_node
    }

    pub fn get_section_mesh(&self) -> &SectionMesh {
        &self.section_mesh
    }

    pub fn get_render_origin(&self) -> BlockPos {
        self.render_origin
    }

    pub fn get_neighbor_section_node(&self, direction: Direction) -> i64 {
        let normal = direction.get_unit_vec3i();
        SectionPos::as_long_xyz(
            SectionPos::x_of(self.section_node) + normal.get_x(),
            SectionPos::y_of(self.section_node) + normal.get_y(),
            SectionPos::z_of(self.section_node) + normal.get_z(),
        )
    }

    pub fn has_translucent_geometry(&self) -> bool {
        self.section_mesh.has_translucent_geometry()
    }

    /// Sets the mesh, and returns the old one to release.
    fn set_section_mesh(&mut self, section_mesh: SectionMesh) -> SectionMesh {
        let old_mesh = std::mem::replace(&mut self.section_mesh, section_mesh);
        self.update_upload_time();
        old_mesh
    }

    /// `sectionMesh.compareAndSet(UNCOMPILED, EMPTY)`, for a section of only air.
    pub fn set_empty_if_uncompiled(&mut self) -> bool {
        if self.section_mesh.is_uncompiled() {
            self.section_mesh = SectionMesh::Empty;
            true
        } else {
            false
        }
    }

    pub fn update_upload_time(&mut self) {
        if self.uploaded_time == 0 {
            self.uploaded_time = Util::get_millis();
        }
    }

    fn create_vertex_sorting(section_pos: &SectionPos, camera_pos: &Vec3) -> VertexSorting {
        VertexSorting::by_distance_xyz(
            (camera_pos.x - f64::from(section_pos.min_block_x())) as f32,
            (camera_pos.y - f64::from(section_pos.min_block_y())) as f32,
            (camera_pos.z - f64::from(section_pos.min_block_z())) as f32,
        )
    }
}

impl Value for RenderSection {
    /// A new section: `reset` (whose buffers the dispatcher releases when it's next compiled or
    /// reset) and its place.
    fn set_section_node(&mut self, section_node: i64) {
        self.section_mesh = SectionMesh::Uncompiled;
        self.uploaded_time = 0;
        self.section_node = section_node;
        let x = SectionPos::section_to_block_coord(SectionPos::x_of(section_node));
        let y = SectionPos::section_to_block_coord(SectionPos::y_of(section_node));
        let z = SectionPos::section_to_block_coord(SectionPos::z_of(section_node));
        self.render_origin = BlockPos::new(x, y, z);
        self.bb = AABB::new(
            f64::from(x),
            f64::from(y),
            f64::from(z),
            f64::from(x + 16),
            f64::from(y + 16),
            f64::from(z + 16),
        );
    }

    fn get_section_node(&self) -> i64 {
        self.section_node
    }
}
