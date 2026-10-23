//! com/mojang/blaze3d/vertex/CompactVectorArray.java

use crate::org::joml::vector3f::Vector3f;

/// Vectors packed in one array of floats, three to a vector.
pub struct CompactVectorArray {
    contents: Vec<f32>,
}

impl CompactVectorArray {
    pub fn new(count: i32) -> CompactVectorArray {
        CompactVectorArray {
            contents: vec![0.0; 3 * count as usize],
        }
    }

    pub fn size(&self) -> usize {
        self.contents.len() / 3
    }

    pub fn set(&mut self, index: usize, x: f32, y: f32, z: f32) {
        self.contents[3 * index] = x;
        self.contents[3 * index + 1] = y;
        self.contents[3 * index + 2] = z;
    }

    pub fn get<'a>(&self, index: usize, output: &'a mut Vector3f) -> &'a Vector3f {
        output.set_xyz(
            self.contents[3 * index],
            self.contents[3 * index + 1],
            self.contents[3 * index + 2],
        )
    }

    pub fn get_x(&self, index: usize) -> f32 {
        self.contents[3 * index]
    }

    pub fn get_y(&self, index: usize) -> f32 {
        self.contents[3 * index + 1]
    }

    pub fn get_z(&self, index: usize) -> f32 {
        self.contents[3 * index + 2]
    }
}
