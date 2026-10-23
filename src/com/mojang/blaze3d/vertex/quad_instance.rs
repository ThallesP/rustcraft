//! com/mojang/blaze3d/vertex/QuadInstance.java
//!
//! A quad's per-vertex colours and light, and its overlay, as a renderer prepares them before
//! `putBlockBakedQuad` writes the quad. Java throws `IndexOutOfBoundsException` for a vertex past
//! 3, where this panics.

use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::light_coords_util::LightCoordsUtil;

/// `OverlayTexture.NO_OVERLAY`, `pack(0, 10)`.
pub const NO_OVERLAY: i32 = 10 << 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuadInstance {
    colors: [i32; 4],
    light_coords: [i32; 4],
    overlay_coords: i32,
}

impl Default for QuadInstance {
    fn default() -> QuadInstance {
        QuadInstance::new()
    }
}

impl QuadInstance {
    pub fn new() -> QuadInstance {
        QuadInstance {
            colors: [-1; 4],
            light_coords: [LightCoordsUtil::FULL_BRIGHT; 4],
            overlay_coords: NO_OVERLAY,
        }
    }

    pub fn get_color(&self, vertex: usize) -> i32 {
        self.colors[vertex]
    }

    pub fn get_light_coords(&self, vertex: usize) -> i32 {
        self.light_coords[vertex]
    }

    pub fn get_light_coords_with_emission(&self, vertex: usize, light_emission: i32) -> i32 {
        LightCoordsUtil::light_coords_with_emission(self.get_light_coords(vertex), light_emission)
    }

    pub fn overlay_coords(&self) -> i32 {
        self.overlay_coords
    }

    pub fn set_color_at(&mut self, vertex: usize, color: i32) {
        self.colors[vertex] = color;
    }

    pub fn set_light_coords_at(&mut self, vertex: usize, light_coords: i32) {
        self.light_coords[vertex] = light_coords;
    }

    pub fn set_color(&mut self, color: i32) {
        self.colors = [color; 4];
    }

    pub fn set_light_coords(&mut self, light_coords: i32) {
        self.light_coords = [light_coords; 4];
    }

    pub fn set_overlay_coords(&mut self, overlay_coords: i32) {
        self.overlay_coords = overlay_coords;
    }

    pub fn multiply_color(&mut self, color: i32) {
        for vertex_color in &mut self.colors {
            *vertex_color = ARGB::multiply(*vertex_color, color);
        }
    }

    pub fn scale_color(&mut self, scale: f32) {
        for vertex_color in &mut self.colors {
            *vertex_color = ARGB::scale_rgb(*vertex_color, scale);
        }
    }
}
