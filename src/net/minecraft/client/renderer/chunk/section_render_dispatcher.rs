//! net/minecraft/client/renderer/chunk/SectionRenderDispatcher.java
//!
//! Compiles sections and puts their meshes in the GPU buffers the terrain draws from. The port
//! compiles synchronously, on the render thread, which only changes when a mesh is ready: the
//! oracle waits for every section to be compiled before it takes a screenshot. Translucent quads
//! are sorted again (`ResortTransparencyTask`) synchronously too. Not ported yet: the background
//! executor and its queue (`SectionTaskDynamicQueue`, `SectionBufferBuilderPool`) and the
//! `StagingBuffer` the uber buffers copy through (`UberGpuBuffer` writes its heaps straight). Java
//! releases a mesh's allocations when its section lets go of it; here the dispatcher keeps each
//! mesh it put in the uber buffers, in the order they came, and takes out of them the ones nothing
//! else holds any more, at each upload, before the new ones go in.

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use crate::com::mojang::blaze3d::vertex::uber_gpu_buffer::UberGpuBuffer;

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
use crate::rustcraft::bench;

pub struct SectionRenderDispatcher {
    fixed_buffers: SectionBufferBuilderPack,
    camera_position: Vec3,
    section_compiler: SectionCompiler,
    chunk_uber_buffers: [SectionUberBuffers; 3],
    /// The meshes with allocations in the uber buffers, in the order they came, and each one's
    /// place in that order by its address, which keys its allocations.
    meshes: BTreeMap<u64, Rc<CompiledSectionMesh>>,
    mesh_order: HashMap<usize, u64>,
    next_mesh: u64,
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

/// `SectionUberBuffers`: a layer's meshes' vertices and sorted indices, keyed by the meshes'
/// addresses.
struct SectionUberBuffers {
    vertex_buffer: UberGpuBuffer<usize>,
    index_buffer: UberGpuBuffer<usize>,
}

impl SectionUberBuffers {
    /// The constructor's: heaps of 128 MiB of vertices aligned to the layer's vertex size, and of
    /// 32 MiB of indices aligned to 8 bytes.
    fn new(layer: ChunkSectionLayer) -> SectionUberBuffers {
        SectionUberBuffers {
            vertex_buffer: UberGpuBuffer::new(
                layer.label(),
                gpu_buffer::USAGE_VERTEX,
                134217728,
                layer.vertex_format().get_vertex_size(),
            ),
            index_buffer: UberGpuBuffer::new(layer.label(), gpu_buffer::USAGE_INDEX, 33554432, 8),
        }
    }
}

/// A mesh's key in the uber buffers: its address, which no other mesh has while it's kept.
fn key(mesh: &Rc<CompiledSectionMesh>) -> usize {
    Rc::as_ptr(mesh) as usize
}

impl SectionRenderDispatcher {
    pub const NEARBY_SECTION_DISTANCE_IN_BLOCKS: i32 = 32;

    pub fn new(section_compiler: SectionCompiler) -> SectionRenderDispatcher {
        SectionRenderDispatcher {
            fixed_buffers: SectionBufferBuilderPack::new(),
            camera_position: Vec3::ZERO,
            section_compiler,
            chunk_uber_buffers: ChunkSectionLayer::VALUES.map(SectionUberBuffers::new),
            meshes: BTreeMap::new(),
            mesh_order: HashMap::new(),
            next_mesh: 0,
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
        let vertex_slice = uber_buffers.vertex_buffer.get_allocation(key(mesh))?;
        let vertex_buffer_offset = vertex_slice.get_offset_from_heap();
        let (index_buffer, index_buffer_offset) =
            match uber_buffers.index_buffer.get_allocation(key(mesh)) {
                Some(index_slice) => (
                    Some(Rc::clone(
                        uber_buffers.index_buffer.get_gpu_buffer(index_slice),
                    )),
                    index_slice.get_offset_from_heap(),
                ),
                None => (None, 0),
            };
        Some(RenderSectionBufferSlice {
            vertex_buffer: Rc::clone(uber_buffers.vertex_buffer.get_gpu_buffer(vertex_slice)),
            vertex_buffer_offset,
            index_buffer,
            index_buffer_offset,
        })
    }

    /// `uploadTerrainBuffersToGpu`: the meshes nothing holds any more (their section was reset or
    /// moved, or compiled again) taken out of the uber buffers, then the staged allocations of each
    /// layer uploaded, until a layer's vertices make or let go of a heap; then the sections whose
    /// buffers are all there get their meshes (`checkSectionMesh`). Returns the indices of those
    /// sections, whose meshes changed (`onSectionMeshUpdate`).
    pub fn upload_terrain_buffers_to_gpu(
        &mut self,
        device: &dyn GpuDevice,
        sections: &mut RotatingSectionStorage<RenderSection>,
    ) -> Vec<usize> {
        let released: Vec<u64> = self
            .meshes
            .iter()
            .filter(|(_, mesh)| Rc::strong_count(mesh) == 1)
            .map(|(&order, _)| order)
            .collect();
        for order in released {
            let mesh = self.meshes.remove(&order).expect("a kept mesh");
            self.remove_allocations(&mesh);
        }
        for layer in ChunkSectionLayer::VALUES {
            let buffers = &mut self.chunk_uber_buffers[layer as usize];
            let (performed_buffer_resize, vertices) =
                buffers.vertex_buffer.upload_staged_allocations(device);
            let (_, indices) = buffers.index_buffer.upload_staged_allocations(device);
            for (uploaded, vertex) in [(vertices, true), (indices, false)]
                .into_iter()
                .flat_map(|(keys, vertex)| keys.into_iter().map(move |key| (key, vertex)))
            {
                let mesh = &self.meshes[&self.mesh_order[&uploaded]];
                if vertex {
                    mesh.set_vertex_buffer_uploaded(layer);
                } else {
                    mesh.set_index_buffer_uploaded(layer);
                }
            }
            if performed_buffer_resize {
                break;
            }
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
            if !all_buffers_updated {
                // Its other layers' turn comes at a later upload.
                self.waiting_for_upload.push((index, section_node, mesh));
                continue;
            }
            let compiled = SectionMesh::Compiled(mesh);
            if !section.section_mesh.same(&compiled) {
                section.set_section_mesh(compiled);
                updated.push(index);
            }
        }
        updated
    }

    /// `removeAllocation` of the mesh in every layer's uber buffers.
    fn remove_allocations(&mut self, mesh: &Rc<CompiledSectionMesh>) {
        let key = key(mesh);
        self.mesh_order.remove(&key);
        for buffers in &mut self.chunk_uber_buffers {
            buffers.vertex_buffer.remove_allocation(key);
            buffers.index_buffer.remove_allocation(key);
        }
    }

    fn release_section_mesh(&mut self, old_mesh: &SectionMesh) {
        if let SectionMesh::Compiled(mesh) = old_mesh
            && let Some(order) = self.mesh_order.get(&key(mesh))
        {
            self.meshes.remove(order);
            self.remove_allocations(mesh);
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
        let compile_start = bench::now();
        let results = self.section_compiler.compile(
            section_pos,
            region,
            &RenderSection::create_vertex_sorting(&section_pos, &camera_pos),
            &mut self.fixed_buffers,
        );
        bench::record(bench::Kind::Compile, compile_start);
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
        // `addSectionBuffersToUberBuffer` of each layer: a layer without indices of its own
        // draws with the shared quad indices, uploaded already.
        let mesh_key = key(&compiled_section_mesh);
        for (layer, vertices, indices) in compiled_section_mesh.pending_buffers.take() {
            let buffers = &mut self.chunk_uber_buffers[layer as usize];
            buffers.vertex_buffer.add_allocation(mesh_key, vertices);
            match indices {
                Some(indices) => buffers.index_buffer.add_allocation(mesh_key, indices),
                None => compiled_section_mesh.set_index_buffer_uploaded(layer),
            }
        }
        self.meshes
            .insert(self.next_mesh, Rc::clone(&compiled_section_mesh));
        self.mesh_order.insert(mesh_key, self.next_mesh);
        self.next_mesh += 1;
        self.waiting_for_upload
            .retain(|(index, _, _)| *index != section.index);
        self.waiting_for_upload
            .push((section.index, section_node, compiled_section_mesh));
        false
    }

    /// `RenderSection.resortTransparency` and its `ResortTransparencyTask`, run now: the section's
    /// translucent quads sorted again from where the camera is, into the layer's buffers at the
    /// next upload, unless the camera is on the same side of the section as when they were last
    /// sorted and in none of its slabs.
    pub fn resort_transparency(&mut self, section: &RenderSection) {
        let SectionMesh::Compiled(mesh) = section.get_section_mesh() else {
            return;
        };
        let Some(state) = mesh.get_transparency_state() else {
            return;
        };
        if mesh.is_empty(ChunkSectionLayer::TRANSLUCENT) {
            return;
        }
        let camera_pos = self.camera_position;
        let section_node = section.section_node;
        let vertex_sorting =
            RenderSection::create_vertex_sorting(&SectionPos::of_long(section_node), &camera_pos);
        let translucency_point_of_view = TranslucencyPointOfView::of(&camera_pos, section_node);
        if !mesh.is_different_point_of_view(&translucency_point_of_view)
            && !translucency_point_of_view.is_axis_aligned()
        {
            return;
        }

        let Some(index_buffer) = state.build_sorted_index_buffer(
            self.fixed_buffers.buffer(ChunkSectionLayer::TRANSLUCENT),
            &vertex_sorting,
        ) else {
            return;
        };
        // `addSectionBuffersToUberBuffer` with only the indices, which the next upload puts in
        // place of the mesh's.
        if self.mesh_order.contains_key(&key(mesh)) {
            self.chunk_uber_buffers[ChunkSectionLayer::TRANSLUCENT as usize]
                .index_buffer
                .add_allocation(key(mesh), index_buffer);
        }
        mesh.set_translucency_point_of_view(translucency_point_of_view);
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
