//! net/minecraft/world/entity/EntityDimensions.java
//!
//! Not ported yet: the attachments (`EntityAttachments`), where passengers, name tags and leashes
//! go, which nothing ported reads.

use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// An entity's size: its width and height, where its eyes are, and whether scaling leaves it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntityDimensions {
    width: f32,
    height: f32,
    eye_height: f32,
    fixed: bool,
}

impl EntityDimensions {
    fn new(width: f32, height: f32, fixed: bool) -> EntityDimensions {
        EntityDimensions {
            width,
            height,
            eye_height: EntityDimensions::default_eye_height(height),
            fixed,
        }
    }

    fn default_eye_height(height: f32) -> f32 {
        height * 0.85
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn eye_height(&self) -> f32 {
        self.eye_height
    }

    pub fn fixed(&self) -> bool {
        self.fixed
    }

    pub fn make_bounding_box(&self, pos: &Vec3) -> AABB {
        self.make_bounding_box_xyz(pos.x, pos.y, pos.z)
    }

    /// Half the width either side of the feet, and the height above them, both floats added to
    /// the position's doubles.
    pub fn make_bounding_box_xyz(&self, x: f64, y: f64, z: f64) -> AABB {
        let w = self.width / 2.0;
        let h = self.height;
        AABB::new(
            x - f64::from(w),
            y,
            z - f64::from(w),
            x + f64::from(w),
            y + f64::from(h),
            z + f64::from(w),
        )
    }

    pub fn scale(&self, scale_factor: f32) -> EntityDimensions {
        self.scale_xy(scale_factor, scale_factor)
    }

    /// `scale(float, float)`: itself when fixed or when neither factor changes it.
    pub fn scale_xy(&self, width_scale_factor: f32, height_scale_factor: f32) -> EntityDimensions {
        if !self.fixed && (width_scale_factor != 1.0 || height_scale_factor != 1.0) {
            EntityDimensions {
                width: self.width * width_scale_factor,
                height: self.height * height_scale_factor,
                eye_height: self.eye_height * height_scale_factor,
                fixed: false,
            }
        } else {
            *self
        }
    }

    pub fn scalable(width: f32, height: f32) -> EntityDimensions {
        EntityDimensions::new(width, height, false)
    }

    pub fn fixed_size(width: f32, height: f32) -> EntityDimensions {
        EntityDimensions::new(width, height, true)
    }

    pub fn with_eye_height(&self, eye_height: f32) -> EntityDimensions {
        EntityDimensions {
            eye_height,
            ..*self
        }
    }
}
