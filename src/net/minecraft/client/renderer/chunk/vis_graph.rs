//! net/minecraft/client/renderer/chunk/VisGraph.java
//!
//! Which faces of a section see each other through the blocks that don't fill it: a flood fill
//! from each edge block that isn't opaque. fastutil's `IntArrayFIFOQueue` is a `VecDeque`, and the
//! `EnumSet` of a fill's faces a bit per direction.

use std::collections::VecDeque;
use std::sync::LazyLock;

use crate::java::util::bit_set::BitSet;
use crate::net::minecraft::client::renderer::chunk::visibility_set::VisibilitySet;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;

pub struct VisGraph {
    bit_set: BitSet,
    empty: i32,
}

/// The index of every block on the section's sides, x, then y, then z.
static INDEX_OF_EDGES: LazyLock<Vec<i32>> = LazyLock::new(|| {
    let mut map = Vec::with_capacity(1352);
    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {
                if x == 0 || x == 15 || y == 0 || y == 15 || z == 0 || z == 15 {
                    map.push(VisGraph::get_index_xyz(x, y, z));
                }
            }
        }
    }
    map
});

impl Default for VisGraph {
    fn default() -> VisGraph {
        VisGraph::new()
    }
}

impl VisGraph {
    const DX: i32 = 1;
    const DZ: i32 = 16;
    const DY: i32 = 256;

    pub fn new() -> VisGraph {
        VisGraph {
            bit_set: BitSet::new(4096),
            empty: 4096,
        }
    }

    pub fn set_opaque(&mut self, pos: &BlockPos) {
        self.bit_set.set(VisGraph::get_index(pos));
        self.empty -= 1;
    }

    fn get_index(pos: &BlockPos) -> i32 {
        VisGraph::get_index_xyz(pos.get_x() & 15, pos.get_y() & 15, pos.get_z() & 15)
    }

    fn get_index_xyz(x: i32, y: i32, z: i32) -> i32 {
        x | y << 8 | z << 4
    }

    pub fn resolve(&mut self) -> VisibilitySet {
        let mut visibility_set = VisibilitySet::new();
        if 4096 - self.empty < 256 {
            visibility_set.set_all(true);
        } else if self.empty == 0 {
            visibility_set.set_all(false);
        } else {
            for &i in INDEX_OF_EDGES.iter() {
                if !self.bit_set.get(i) {
                    visibility_set.add(&self.flood_fill(i));
                }
            }
        }

        visibility_set
    }

    /// The faces the blocks reached from `start_index` touch, in `Direction`'s order.
    fn flood_fill(&mut self, start_index: i32) -> Vec<Direction> {
        let mut edges = 0u8;
        let mut queue = VecDeque::new();
        queue.push_back(start_index);
        self.bit_set.set(start_index);

        while let Some(index) = queue.pop_front() {
            VisGraph::add_edges(index, &mut edges);

            for direction in Direction::VALUES {
                let neighbor_index = VisGraph::get_neighbor_index_at_face(index, direction);
                if neighbor_index >= 0 && !self.bit_set.get(neighbor_index) {
                    self.bit_set.set(neighbor_index);
                    queue.push_back(neighbor_index);
                }
            }
        }

        Direction::VALUES
            .into_iter()
            .filter(|&direction| edges & 1 << direction as u32 != 0)
            .collect()
    }

    fn add_edges(index: i32, edges: &mut u8) {
        let mut add = |direction: Direction| *edges |= 1 << direction as u32;
        let x = index & 15;
        if x == 0 {
            add(Direction::WEST);
        } else if x == 15 {
            add(Direction::EAST);
        }

        let y = index >> 8 & 15;
        if y == 0 {
            add(Direction::DOWN);
        } else if y == 15 {
            add(Direction::UP);
        }

        let z = index >> 4 & 15;
        if z == 0 {
            add(Direction::NORTH);
        } else if z == 15 {
            add(Direction::SOUTH);
        }
    }

    fn get_neighbor_index_at_face(index: i32, direction: Direction) -> i32 {
        match direction {
            Direction::DOWN if index >> 8 & 15 == 0 => -1,
            Direction::DOWN => index - VisGraph::DY,
            Direction::UP if index >> 8 & 15 == 15 => -1,
            Direction::UP => index + VisGraph::DY,
            Direction::NORTH if index >> 4 & 15 == 0 => -1,
            Direction::NORTH => index - VisGraph::DZ,
            Direction::SOUTH if index >> 4 & 15 == 15 => -1,
            Direction::SOUTH => index + VisGraph::DZ,
            Direction::WEST if index & 15 == 0 => -1,
            Direction::WEST => index - VisGraph::DX,
            Direction::EAST if index & 15 == 15 => -1,
            Direction::EAST => index + VisGraph::DX,
        }
    }
}
