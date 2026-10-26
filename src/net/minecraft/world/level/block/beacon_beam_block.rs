//! net/minecraft/world/level/block/BeaconBeamBlock.java
//!
//! The blocks a beacon's beam takes a colour from: stained glass and its panes, each of its colour,
//! and the beacon itself, white. Java's interface is implemented by their classes, whose `getColor`
//! is their constructor's colour (the beacon's own, `DyeColor.WHITE`); here the colour is the one
//! whose collection holds the block.

use crate::net::minecraft::world::item::dye_color::DyeColor;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::blocks::Blocks;

/// `instanceof BeaconBeamBlock`'s `getColor()`: none for a block that isn't one.
pub fn get_color(block: Block) -> Option<DyeColor> {
    if block == Blocks::BEACON {
        return Some(DyeColor::WHITE);
    }
    DyeColor::VALUES.into_iter().find(|&color| {
        *Blocks::STAINED_GLASS.pick(color) == block
            || *Blocks::STAINED_GLASS_PANE.pick(color) == block
    })
}
