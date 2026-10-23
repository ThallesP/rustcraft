//! net/minecraft/world/level/levelgen/structure/BoundingBox.java
//!
//! Only what the renderer's octree reads so far: the corners and the spans. The rest (the
//! structures' boxes, `CODEC`, the moves and intersections) comes with what calls it.

/// A box of whole blocks, both corners inclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BoundingBox {
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
}

impl BoundingBox {
    /// Inverted corners are swapped, where Java also logs (and pauses in an IDE).
    pub fn new(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Self {
        BoundingBox {
            min_x: min_x.min(max_x),
            min_y: min_y.min(max_y),
            min_z: min_z.min(max_z),
            max_x: min_x.max(max_x),
            max_y: min_y.max(max_y),
            max_z: min_z.max(max_z),
        }
    }

    pub fn get_x_span(&self) -> i32 {
        self.max_x - self.min_x + 1
    }

    pub fn get_y_span(&self) -> i32 {
        self.max_y - self.min_y + 1
    }

    pub fn get_z_span(&self) -> i32 {
        self.max_z - self.min_z + 1
    }

    pub fn min_x(&self) -> i32 {
        self.min_x
    }

    pub fn min_y(&self) -> i32 {
        self.min_y
    }

    pub fn min_z(&self) -> i32 {
        self.min_z
    }

    pub fn max_x(&self) -> i32 {
        self.max_x
    }

    pub fn max_y(&self) -> i32 {
        self.max_y
    }

    pub fn max_z(&self) -> i32 {
        self.max_z
    }
}

#[cfg(test)]
mod tests {
    use super::BoundingBox;
    use crate::golden::encode_fields;

    // [minX, minY, minZ, maxX, maxY, maxZ], made through the constructor, so in order.
    encode_fields!(BoundingBox {
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32
    });
}
