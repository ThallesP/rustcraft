//! net/minecraft/client/renderer/chunk/SectionMesh.java
//!
//! A render section's mesh: not compiled yet, compiled with nothing to draw, or compiled
//! (`CompiledSectionMesh`). Java's interface is implemented by `CompiledSectionMesh` and its two
//! anonymous constants, so it's an enum of the three. Block entities aren't ported, so a mesh has
//! none to render.

use std::rc::Rc;

use crate::com::mojang::renderpearl::api::pipeline::index_type::IndexType;
use crate::net::minecraft::client::renderer::chunk::chunk_section_layer::ChunkSectionLayer;
use crate::net::minecraft::client::renderer::chunk::compiled_section_mesh::CompiledSectionMesh;
use crate::net::minecraft::client::renderer::chunk::translucency_point_of_view::TranslucencyPointOfView;
use crate::net::minecraft::core::direction::Direction;

#[derive(Clone)]
pub enum SectionMesh {
    /// `CompiledSectionMesh.UNCOMPILED`: sees nothing through itself.
    Uncompiled,
    /// `CompiledSectionMesh.EMPTY`: air, seen through every way.
    Empty,
    Compiled(Rc<CompiledSectionMesh>),
}

/// `SectionMesh.SectionDraw`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SectionDraw {
    pub index_count: i32,
    pub index_type: IndexType,
    pub has_custom_index_buffer: bool,
}

impl SectionMesh {
    pub fn is_different_point_of_view(&self, point_of_view: &TranslucencyPointOfView) -> bool {
        match self {
            SectionMesh::Compiled(mesh) => mesh.is_different_point_of_view(point_of_view),
            _ => false,
        }
    }

    pub fn has_renderable_layers(&self) -> bool {
        match self {
            SectionMesh::Compiled(mesh) => mesh.has_renderable_layers(),
            _ => false,
        }
    }

    pub fn has_translucent_geometry(&self) -> bool {
        match self {
            SectionMesh::Compiled(mesh) => mesh.has_translucent_geometry(),
            _ => false,
        }
    }

    pub fn is_empty(&self, layer: ChunkSectionLayer) -> bool {
        match self {
            SectionMesh::Compiled(mesh) => mesh.is_empty(layer),
            _ => true,
        }
    }

    pub fn faces_can_see_eachother(&self, direction1: Direction, direction2: Direction) -> bool {
        match self {
            SectionMesh::Uncompiled => false,
            SectionMesh::Empty => true,
            SectionMesh::Compiled(mesh) => mesh.faces_can_see_eachother(direction1, direction2),
        }
    }

    pub fn get_section_draw(&self, layer: ChunkSectionLayer) -> Option<SectionDraw> {
        match self {
            SectionMesh::Compiled(mesh) => mesh.get_section_draw(layer),
            _ => None,
        }
    }

    pub fn get_compile_task_start_time(&self) -> i64 {
        match self {
            SectionMesh::Compiled(mesh) => mesh.get_compile_task_start_time(),
            _ => 0,
        }
    }

    pub fn is_uncompiled(&self) -> bool {
        matches!(self, SectionMesh::Uncompiled)
    }

    /// Java compares meshes by reference.
    pub fn same(&self, other: &SectionMesh) -> bool {
        match (self, other) {
            (SectionMesh::Uncompiled, SectionMesh::Uncompiled) => true,
            (SectionMesh::Empty, SectionMesh::Empty) => true,
            (SectionMesh::Compiled(a), SectionMesh::Compiled(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
