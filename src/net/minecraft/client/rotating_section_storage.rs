//! net/minecraft/client/RotatingSectionStorage.java
//!
//! A value for each section of the columns around a center, kept in a grid that wraps around:
//! moving the center renumbers only the values whose section changed.

use crate::java::lang::math::Math;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;

/// `RotatingSectionStorage.Value`.
pub trait Value {
    fn set_section_node(&mut self, section_node: i64);

    fn get_section_node(&self) -> i64;
}

pub struct RotatingSectionStorage<T: Value> {
    nodes: Vec<T>,
    radius: i32,
    min_y: i32,
    max_y: i32,
    section_grid_size_y: i32,
    section_grid_size_xz: i32,
    center_section_pos: SectionPos,
}

impl<T: Value> RotatingSectionStorage<T> {
    pub fn new(
        radius: i32,
        min_y: i32,
        max_y: i32,
        mut value_creator: impl FnMut(usize, i64) -> T,
    ) -> RotatingSectionStorage<T> {
        let section_grid_size_y = max_y - min_y + 1;
        let section_grid_size_xz = radius * 2 + 1;
        let total_sections =
            (section_grid_size_xz * section_grid_size_xz * section_grid_size_y) as usize;
        let mut nodes: Vec<Option<T>> = (0..total_sections).map(|_| None).collect();
        for x in 0..section_grid_size_xz {
            for y in 0..section_grid_size_y {
                for z in 0..section_grid_size_xz {
                    let index = ((z * section_grid_size_y + y) * section_grid_size_xz + x) as usize;
                    let section_node = SectionPos::as_long_xyz(x, y + min_y, z);
                    nodes[index] = Some(value_creator(index, section_node));
                }
            }
        }
        RotatingSectionStorage {
            nodes: nodes
                .into_iter()
                .map(|node| node.expect("every node is made"))
                .collect(),
            radius,
            min_y,
            max_y,
            section_grid_size_y,
            section_grid_size_xz,
            center_section_pos: SectionPos::of(i32::MIN, i32::MIN, i32::MIN),
        }
    }

    /// Moves the center, giving each value whose section changed its new one. False if the
    /// center didn't move.
    pub fn reposition_center(&mut self, new_center_section_pos: SectionPos) -> bool {
        if new_center_section_pos == self.center_section_pos {
            return false;
        }

        let lowest_x = new_center_section_pos.x() - self.radius;
        let lowest_z = new_center_section_pos.z() - self.radius;
        for grid_x in 0..self.section_grid_size_xz {
            let new_section_x =
                lowest_x + Math::floor_mod_i32(grid_x - lowest_x, self.section_grid_size_xz);
            for grid_z in 0..self.section_grid_size_xz {
                let new_section_z =
                    lowest_z + Math::floor_mod_i32(grid_z - lowest_z, self.section_grid_size_xz);
                for grid_y in 0..self.section_grid_size_y {
                    let new_section_y = self.min_y + grid_y;
                    let index = self.get_section_index(grid_x, grid_y, grid_z);
                    let value = &mut self.nodes[index];
                    let section_node =
                        SectionPos::as_long_xyz(new_section_x, new_section_y, new_section_z);
                    if value.get_section_node() != section_node {
                        value.set_section_node(section_node);
                    }
                }
            }
        }

        self.center_section_pos = new_center_section_pos;
        true
    }

    pub fn radius(&self) -> i32 {
        self.radius
    }

    pub fn min_y(&self) -> i32 {
        self.min_y
    }

    pub fn max_y(&self) -> i32 {
        self.max_y
    }

    pub fn height(&self) -> i32 {
        self.section_grid_size_y
    }

    pub fn center_section_pos(&self) -> SectionPos {
        self.center_section_pos
    }

    pub fn get_index_at(&self, pos: &BlockPos) -> Option<usize> {
        self.get_index(SectionPos::as_long_xyz(
            SectionPos::block_to_section_coord(pos.get_x()),
            SectionPos::block_to_section_coord(pos.get_y()),
            SectionPos::block_to_section_coord(pos.get_z()),
        ))
    }

    /// The index of the value for the section, `getValue`'s, if the storage covers it.
    pub fn get_index(&self, section_node: i64) -> Option<usize> {
        let section_x = SectionPos::x_of(section_node);
        let section_y = SectionPos::y_of(section_node);
        let section_z = SectionPos::z_of(section_node);
        if !self.contains_section(section_x, section_y, section_z) {
            return None;
        }

        let y = section_y - self.min_y;
        let x = Math::floor_mod_i32(section_x, self.section_grid_size_xz);
        let z = Math::floor_mod_i32(section_z, self.section_grid_size_xz);
        Some(self.get_section_index(x, y, z))
    }

    pub fn get_value(&self, section_node: i64) -> Option<&T> {
        self.get_index(section_node).map(|index| &self.nodes[index])
    }

    pub fn get_value_mut(&mut self, section_node: i64) -> Option<&mut T> {
        self.get_index(section_node)
            .map(move |index| &mut self.nodes[index])
    }

    fn contains_section(&self, section_x: i32, section_y: i32, section_z: i32) -> bool {
        if section_y >= self.min_y && section_y <= self.max_y {
            !(section_x < self.center_section_pos.x() - self.radius
                || section_x > self.center_section_pos.x() + self.radius)
                && section_z >= self.center_section_pos.z() - self.radius
                && section_z <= self.center_section_pos.z() + self.radius
        } else {
            false
        }
    }

    fn get_section_index(&self, x: i32, y: i32, z: i32) -> usize {
        ((z * self.section_grid_size_y + y) * self.section_grid_size_xz + x) as usize
    }

    pub fn get(&self, index: usize) -> &T {
        &self.nodes[index]
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        &mut self.nodes[index]
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.nodes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.nodes.iter_mut()
    }

    pub fn size(&self) -> usize {
        self.nodes.len()
    }
}
