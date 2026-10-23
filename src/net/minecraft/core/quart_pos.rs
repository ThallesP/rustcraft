//! net/minecraft/core/QuartPos.java
//!
//! A biome's coordinates, a quarter of a section's: 4 blocks.

/// Conversions between blocks, quarts and sections.
pub struct QuartPos;

impl QuartPos {
    pub const BITS: i32 = 2;
    pub const SIZE: i32 = 4;
    pub const MASK: i32 = 3;
    const SECTION_TO_QUARTS_BITS: i32 = 2;

    pub fn from_block(block: i32) -> i32 {
        block >> QuartPos::BITS
    }

    pub fn quart_local(quart: i32) -> i32 {
        quart & QuartPos::MASK
    }

    pub fn to_block(quart: i32) -> i32 {
        quart << QuartPos::BITS
    }

    pub fn from_section(section: i32) -> i32 {
        section << QuartPos::SECTION_TO_QUARTS_BITS
    }

    pub fn to_section(quart: i32) -> i32 {
        quart >> QuartPos::SECTION_TO_QUARTS_BITS
    }
}
