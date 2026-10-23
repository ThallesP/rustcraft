//! net/minecraft/client/model/geom/builders/CubeDeformation.java

/// How far a cube grows past its box on each axis, in sixteenths of a block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubeDeformation {
    pub(crate) grow_x: f32,
    pub(crate) grow_y: f32,
    pub(crate) grow_z: f32,
}

impl CubeDeformation {
    pub const NONE: CubeDeformation = CubeDeformation::new(0.0);

    pub const fn new_xyz(grow_x: f32, grow_y: f32, grow_z: f32) -> CubeDeformation {
        CubeDeformation {
            grow_x,
            grow_y,
            grow_z,
        }
    }

    pub const fn new(grow: f32) -> CubeDeformation {
        CubeDeformation::new_xyz(grow, grow, grow)
    }

    pub fn extend(&self, factor: f32) -> CubeDeformation {
        CubeDeformation::new_xyz(
            self.grow_x + factor,
            self.grow_y + factor,
            self.grow_z + factor,
        )
    }

    pub fn extend_xyz(&self, factor_x: f32, factor_y: f32, factor_z: f32) -> CubeDeformation {
        CubeDeformation::new_xyz(
            self.grow_x + factor_x,
            self.grow_y + factor_y,
            self.grow_z + factor_z,
        )
    }
}
