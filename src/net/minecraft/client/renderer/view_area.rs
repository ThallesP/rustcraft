//! net/minecraft/client/renderer/ViewArea.java
//!
//! The render sections of the columns within the render distance of the camera's, which move
//! with it. Java's view area invalidates the occlusion graph when the camera's section changes;
//! here `reposition_camera` says so and the level renderer does it.

use crate::net::minecraft::client::renderer::chunk::section_render_dispatcher::RenderSection;
use crate::net::minecraft::client::rotating_section_storage::RotatingSectionStorage;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;

pub struct ViewArea {
    pub sections: RotatingSectionStorage<RenderSection>,
    min_y: i32,
    max_y: i32,
}

impl ViewArea {
    pub fn new(
        min_y: i32,
        max_y: i32,
        min_section_y: i32,
        max_section_y: i32,
        render_distance: i32,
    ) -> ViewArea {
        ViewArea {
            sections: RotatingSectionStorage::new(
                render_distance,
                min_section_y,
                max_section_y,
                RenderSection::new,
            ),
            min_y,
            max_y,
        }
    }

    pub fn size(&self) -> usize {
        self.sections.size()
    }

    pub fn min_y(&self) -> i32 {
        self.min_y
    }

    pub fn max_y(&self) -> i32 {
        self.max_y
    }

    pub fn min_section_y(&self) -> i32 {
        self.sections.min_y()
    }

    pub fn max_section_y(&self) -> i32 {
        self.sections.max_y()
    }

    pub fn section_count(&self) -> i32 {
        self.sections.height()
    }

    pub fn get_view_distance(&self) -> i32 {
        self.sections.radius()
    }

    /// Moves the sections with the camera; true if its section changed, which invalidates the
    /// occlusion graph.
    pub fn reposition_camera(&mut self, camera_section_pos: SectionPos) -> bool {
        self.sections.reposition_center(camera_section_pos)
    }

    pub fn get_camera_section_pos(&self) -> SectionPos {
        self.sections.center_section_pos()
    }

    /// The index of the section at the position, if the view area has it.
    pub fn get_render_section_at(&self, pos: &BlockPos) -> Option<usize> {
        self.sections.get_index_at(pos)
    }

    pub fn get_render_section(&self, section_node: i64) -> Option<usize> {
        self.sections.get_index(section_node)
    }

    pub fn section(&self, index: usize) -> &RenderSection {
        self.sections.get(index)
    }
}
