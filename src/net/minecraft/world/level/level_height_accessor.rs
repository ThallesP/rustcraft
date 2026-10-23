//! net/minecraft/world/level/LevelHeightAccessor.java
//!
//! An interface whose defaults work out the rest from a height and a lowest y; `create` is
//! `LevelHeightAccessor::create`'s struct, `Height`.

use crate::net::minecraft::core::section_pos::SectionPos;

/// How high a level or chunk is, from its lowest block up.
pub trait LevelHeightAccessor {
    fn get_height(&self) -> i32;

    fn get_min_y(&self) -> i32;

    fn get_max_y(&self) -> i32 {
        self.get_min_y() + self.get_height() - 1
    }

    fn get_sections_count(&self) -> i32 {
        self.get_max_section_y() - self.get_min_section_y() + 1
    }

    fn get_min_section_y(&self) -> i32 {
        SectionPos::block_to_section_coord(self.get_min_y())
    }

    fn get_max_section_y(&self) -> i32 {
        SectionPos::block_to_section_coord(self.get_max_y())
    }

    fn is_inside_build_height(&self, block_y: i32) -> bool {
        block_y >= self.get_min_y() && block_y <= self.get_max_y()
    }

    fn is_outside_build_height(&self, block_y: i32) -> bool {
        block_y < self.get_min_y() || block_y > self.get_max_y()
    }

    fn get_section_index(&self, block_y: i32) -> i32 {
        self.get_section_index_from_section_y(SectionPos::block_to_section_coord(block_y))
    }

    fn get_section_index_from_section_y(&self, section_y: i32) -> i32 {
        section_y - self.get_min_section_y()
    }

    fn get_section_y_from_section_index(&self, section_index: i32) -> i32 {
        section_index + self.get_min_section_y()
    }
}

/// `LevelHeightAccessor.create(minY, height)`'s anonymous class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Height {
    pub min_y: i32,
    pub height: i32,
}

impl LevelHeightAccessor for Height {
    fn get_height(&self) -> i32 {
        self.height
    }

    fn get_min_y(&self) -> i32 {
        self.min_y
    }
}

pub fn create(min_y: i32, height: i32) -> Height {
    Height { min_y, height }
}
