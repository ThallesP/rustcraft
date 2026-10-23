//! net/minecraft/client/resources/model/geometry/QuadCollection.java
//!
//! Java's lists are views into one list of every quad, the unculled ones first, then the ones
//! culled toward north, south, east, west, up and down; here they're ranges of it. Java works out
//! the material flags on first use, without a lock; here in a `OnceLock`.

use std::ops::Range;
use std::sync::OnceLock;

use crate::net::minecraft::client::resources::model::geometry::baked_quad::BakedQuad;
use crate::net::minecraft::core::direction::Direction;

#[derive(Default)]
pub struct QuadCollection {
    all: Vec<BakedQuad>,
    unculled: Range<usize>,
    north: Range<usize>,
    south: Range<usize>,
    east: Range<usize>,
    west: Range<usize>,
    up: Range<usize>,
    down: Range<usize>,
    material_flags: OnceLock<i32>,
}

/// `QuadCollection.Builder`.
#[derive(Default)]
pub struct Builder {
    unculled_faces: Vec<BakedQuad>,
    // Java's `ArrayListMultimap`, by direction.
    culled_faces: [Vec<BakedQuad>; 6],
}

impl QuadCollection {
    /// `EMPTY`.
    pub fn empty() -> QuadCollection {
        QuadCollection::default()
    }

    fn compute_material_flags(quads: &[BakedQuad]) -> i32 {
        quads
            .iter()
            .fold(0, |flags, quad| flags | quad.material_info.flags())
    }

    pub fn get_quads(&self, direction: Option<Direction>) -> &[BakedQuad] {
        let range = match direction {
            None => &self.unculled,
            Some(Direction::NORTH) => &self.north,
            Some(Direction::SOUTH) => &self.south,
            Some(Direction::EAST) => &self.east,
            Some(Direction::WEST) => &self.west,
            Some(Direction::UP) => &self.up,
            Some(Direction::DOWN) => &self.down,
        };
        &self.all[range.clone()]
    }

    pub fn get_all(&self) -> &[BakedQuad] {
        &self.all
    }

    pub fn material_flags(&self) -> i32 {
        *self
            .material_flags
            .get_or_init(|| QuadCollection::compute_material_flags(&self.all))
    }

    pub fn has_material_flag(&self, flag: i32) -> bool {
        (self.material_flags() & flag) != 0
    }
}

impl Builder {
    pub fn new() -> Builder {
        Builder::default()
    }

    pub fn add_culled_face(&mut self, direction: Direction, quad: BakedQuad) -> &mut Builder {
        self.culled_faces[direction as usize].push(quad);
        self
    }

    pub fn add_unculled_face(&mut self, quad: BakedQuad) -> &mut Builder {
        self.unculled_faces.push(quad);
        self
    }

    pub fn add_all(&mut self, quad_collection: &QuadCollection) -> &mut Builder {
        for direction in [
            Direction::UP,
            Direction::DOWN,
            Direction::NORTH,
            Direction::SOUTH,
            Direction::EAST,
            Direction::WEST,
        ] {
            self.culled_faces[direction as usize]
                .extend_from_slice(quad_collection.get_quads(Some(direction)));
        }
        self.unculled_faces
            .extend_from_slice(quad_collection.get_quads(None));
        self
    }

    pub fn build(self) -> QuadCollection {
        let mut all = self.unculled_faces;
        let unculled = 0..all.len();
        let [down, up, north, south, west, east] = self.culled_faces;
        let mut sublist = |quads: Vec<BakedQuad>| {
            let start = all.len();
            all.extend(quads);
            start..all.len()
        };
        let north = sublist(north);
        let south = sublist(south);
        let east = sublist(east);
        let west = sublist(west);
        let up = sublist(up);
        let down = sublist(down);
        QuadCollection {
            all,
            unculled,
            north,
            south,
            east,
            west,
            up,
            down,
            material_flags: OnceLock::new(),
        }
    }
}
