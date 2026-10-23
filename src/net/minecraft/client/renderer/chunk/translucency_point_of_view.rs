//! net/minecraft/client/renderer/chunk/TranslucencyPointOfView.java

use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// Which side of a section the camera is on, along each axis: -1, 0 (in its slab) or 1. A
/// section's translucent quads are sorted again when it changes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TranslucencyPointOfView {
    x: i32,
    y: i32,
    z: i32,
}

impl TranslucencyPointOfView {
    pub fn of(camera_pos: &Vec3, section_node: i64) -> TranslucencyPointOfView {
        let mut point_of_view = TranslucencyPointOfView::default();
        point_of_view.set(camera_pos, section_node);
        point_of_view
    }

    pub fn set(&mut self, camera_pos: &Vec3, section_pos: i64) -> &mut Self {
        self.x =
            TranslucencyPointOfView::get_coordinate(camera_pos.x, SectionPos::x_of(section_pos));
        self.y =
            TranslucencyPointOfView::get_coordinate(camera_pos.y, SectionPos::y_of(section_pos));
        self.z =
            TranslucencyPointOfView::get_coordinate(camera_pos.z, SectionPos::z_of(section_pos));
        self
    }

    fn get_coordinate(camera_coordinate: f64, section: i32) -> i32 {
        // SectionPos.blockToSectionCoord(double) floors first.
        let relative_section =
            SectionPos::block_to_section_coord(Mth::floor_f64(camera_coordinate)) - section;
        Mth::clamp_i32(relative_section, -1, 1)
    }

    pub fn is_axis_aligned(&self) -> bool {
        self.x == 0 || self.y == 0 || self.z == 0
    }
}
