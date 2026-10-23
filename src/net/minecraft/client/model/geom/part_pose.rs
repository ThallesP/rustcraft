//! net/minecraft/client/model/geom/PartPose.java

/// A model part's offset (in sixteenths of a block), rotation (radians) and scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PartPose {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub x_rot: f32,
    pub y_rot: f32,
    pub z_rot: f32,
    pub x_scale: f32,
    pub y_scale: f32,
    pub z_scale: f32,
}

impl PartPose {
    pub const ZERO: PartPose = PartPose::offset_and_rotation(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

    pub const fn offset(x: f32, y: f32, z: f32) -> PartPose {
        PartPose::offset_and_rotation(x, y, z, 0.0, 0.0, 0.0)
    }

    pub const fn rotation(x: f32, y: f32, z: f32) -> PartPose {
        PartPose::offset_and_rotation(0.0, 0.0, 0.0, x, y, z)
    }

    pub const fn offset_and_rotation(
        offset_x: f32,
        offset_y: f32,
        offset_z: f32,
        rotation_x: f32,
        rotation_y: f32,
        rotation_z: f32,
    ) -> PartPose {
        PartPose {
            x: offset_x,
            y: offset_y,
            z: offset_z,
            x_rot: rotation_x,
            y_rot: rotation_y,
            z_rot: rotation_z,
            x_scale: 1.0,
            y_scale: 1.0,
            z_scale: 1.0,
        }
    }

    pub fn translated(&self, x: f32, y: f32, z: f32) -> PartPose {
        PartPose {
            x: self.x + x,
            y: self.y + y,
            z: self.z + z,
            ..*self
        }
    }

    pub fn with_scale(&self, scale: f32) -> PartPose {
        PartPose {
            x_scale: scale,
            y_scale: scale,
            z_scale: scale,
            ..*self
        }
    }

    pub fn scaled(&self, factor: f32) -> PartPose {
        if factor == 1.0 {
            *self
        } else {
            self.scaled_xyz(factor, factor, factor)
        }
    }

    pub fn scaled_xyz(&self, scale_x: f32, scale_y: f32, scale_z: f32) -> PartPose {
        PartPose {
            x: self.x * scale_x,
            y: self.y * scale_y,
            z: self.z * scale_z,
            x_rot: self.x_rot,
            y_rot: self.y_rot,
            z_rot: self.z_rot,
            x_scale: self.x_scale * scale_x,
            y_scale: self.y_scale * scale_y,
            z_scale: self.z_scale * scale_z,
        }
    }
}
