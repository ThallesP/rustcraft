//! net/minecraft/core/Cursor3D.java

/// Walks every block of a box, x fastest and z slowest, telling inside blocks from the box's
/// faces, edges and corners.
#[derive(Clone, Debug)]
pub struct Cursor3D {
    origin_x: i32,
    origin_y: i32,
    origin_z: i32,
    width: i32,
    height: i32,
    depth: i32,
    end: i32,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
}

impl Cursor3D {
    pub const TYPE_INSIDE: i32 = 0;
    pub const TYPE_FACE: i32 = 1;
    pub const TYPE_EDGE: i32 = 2;
    pub const TYPE_CORNER: i32 = 3;

    pub fn new(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Cursor3D {
        let width = max_x.wrapping_sub(min_x).wrapping_add(1);
        let height = max_y.wrapping_sub(min_y).wrapping_add(1);
        let depth = max_z.wrapping_sub(min_z).wrapping_add(1);
        Cursor3D {
            origin_x: min_x,
            origin_y: min_y,
            origin_z: min_z,
            width,
            height,
            depth,
            end: width.wrapping_mul(height).wrapping_mul(depth),
            index: 0,
            x: 0,
            y: 0,
            z: 0,
        }
    }

    pub fn advance(&mut self) -> bool {
        if self.index == self.end {
            return false;
        }

        self.x = self.index % self.width;
        let slice = self.index / self.width;
        self.y = slice % self.height;
        self.z = slice / self.height;
        self.index += 1;
        true
    }

    pub fn next_x(&self) -> i32 {
        self.origin_x + self.x
    }

    pub fn next_y(&self) -> i32 {
        self.origin_y + self.y
    }

    pub fn next_z(&self) -> i32 {
        self.origin_z + self.z
    }

    /// How many of the box's sides the block is on: 0 inside, 1 on a face, 2 on an edge, 3 at a
    /// corner.
    pub fn get_next_type(&self) -> i32 {
        let mut r#type = 0;
        if self.x == 0 || self.x == self.width - 1 {
            r#type += 1;
        }

        if self.y == 0 || self.y == self.height - 1 {
            r#type += 1;
        }

        if self.z == 0 || self.z == self.depth - 1 {
            r#type += 1;
        }

        r#type
    }
}
