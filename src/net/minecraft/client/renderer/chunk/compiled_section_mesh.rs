//! net/minecraft/client/renderer/chunk/CompiledSectionMesh.java
//!
//! A section's compiled mesh: how each layer draws, what it sees through, and its translucent
//! quads' centroids for sorting them again. Its buffers live in the dispatcher's per-layer
//! buffers, which `SectionRenderDispatcher` fills, so the upload flags are set when they're
//! there. Block entities aren't ported.

use std::cell::{Cell, RefCell};

use crate::com::mojang::blaze3d::vertex::mesh_data::SortState;
use crate::net::minecraft::client::renderer::chunk::chunk_section_layer::ChunkSectionLayer;
use crate::net::minecraft::client::renderer::chunk::section_compiler::Results;
use crate::net::minecraft::client::renderer::chunk::section_mesh::SectionDraw;
use crate::net::minecraft::client::renderer::chunk::translucency_point_of_view::TranslucencyPointOfView;
use crate::net::minecraft::client::renderer::chunk::visibility_set::VisibilitySet;
use crate::net::minecraft::core::direction::Direction;

/// A layer's vertex bytes and sorted index bytes.
pub(crate) type PendingBuffers = (ChunkSectionLayer, Vec<u8>, Option<Vec<u8>>);

pub struct CompiledSectionMesh {
    visibility_set: VisibilitySet,
    transparency_state: Option<SortState>,
    translucency_point_of_view: Cell<Option<TranslucencyPointOfView>>,
    /// Each layer's draw, by the layer's ordinal (Java's `EnumMap`).
    draws: [Option<SectionDraw>; 3],
    vertex_buffer_uploaded: [Cell<bool>; 3],
    index_buffer_uploaded: [Cell<bool>; 3],
    compile_task_start_time_ns: i64,
    /// The layers' vertex and index bytes, until the dispatcher's buffers hold them.
    pub(crate) pending_buffers: RefCell<Vec<PendingBuffers>>,
}

impl CompiledSectionMesh {
    pub fn new(
        translucency_point_of_view: TranslucencyPointOfView,
        results: Results,
        compile_task_start_time_ns: i64,
    ) -> CompiledSectionMesh {
        let mut draws = [None; 3];
        let mut pending_buffers = Vec::new();
        for (layer, mesh) in results.rendered_layers {
            draws[layer as usize] = Some(SectionDraw {
                index_count: mesh.draw_state().index_count,
                index_type: mesh.draw_state().index_type,
                has_custom_index_buffer: mesh.index_buffer().is_some(),
            });
            pending_buffers.push((
                layer,
                mesh.vertex_buffer().to_vec(),
                mesh.index_buffer().map(<[u8]>::to_vec),
            ));
        }
        CompiledSectionMesh {
            visibility_set: results.visibility_set,
            transparency_state: results.transparency_state,
            translucency_point_of_view: Cell::new(Some(translucency_point_of_view)),
            draws,
            vertex_buffer_uploaded: Default::default(),
            index_buffer_uploaded: Default::default(),
            compile_task_start_time_ns,
            pending_buffers: RefCell::new(pending_buffers),
        }
    }

    pub fn set_translucency_point_of_view(
        &self,
        translucency_point_of_view: TranslucencyPointOfView,
    ) {
        self.translucency_point_of_view
            .set(Some(translucency_point_of_view));
    }

    pub fn is_different_point_of_view(&self, point_of_view: &TranslucencyPointOfView) -> bool {
        self.translucency_point_of_view.get() != Some(*point_of_view)
    }

    pub fn has_renderable_layers(&self) -> bool {
        self.draws.iter().any(Option::is_some)
    }

    pub fn is_empty(&self, layer: ChunkSectionLayer) -> bool {
        self.draws[layer as usize].is_none()
    }

    pub fn faces_can_see_eachother(&self, direction1: Direction, direction2: Direction) -> bool {
        self.visibility_set
            .visibility_between(direction1, direction2)
    }

    pub fn get_section_draw(&self, layer: ChunkSectionLayer) -> Option<SectionDraw> {
        self.draws[layer as usize]
    }

    pub fn get_compile_task_start_time(&self) -> i64 {
        self.compile_task_start_time_ns
    }

    pub fn is_vertex_buffer_uploaded(&self, layer: ChunkSectionLayer) -> bool {
        self.vertex_buffer_uploaded[layer as usize].get()
    }

    pub fn is_index_buffer_uploaded(&self, layer: ChunkSectionLayer) -> bool {
        self.index_buffer_uploaded[layer as usize].get()
    }

    pub fn set_vertex_buffer_uploaded(&self, layer: ChunkSectionLayer) {
        self.vertex_buffer_uploaded[layer as usize].set(true);
    }

    pub fn set_index_buffer_uploaded(&self, layer: ChunkSectionLayer) {
        self.index_buffer_uploaded[layer as usize].set(true);
    }

    pub fn has_translucent_geometry(&self) -> bool {
        self.draws[ChunkSectionLayer::TRANSLUCENT as usize].is_some()
    }

    pub fn get_transparency_state(&self) -> Option<&SortState> {
        self.transparency_state.as_ref()
    }
}
