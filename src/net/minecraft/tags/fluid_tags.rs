//! net/minecraft/tags/FluidTags.java
//!
//! Only the tags an entity in a fluid asks about. Like `BlockTags`, a fluid's tags come from the
//! server in Java (`ClientboundUpdateTagsPacket`), which isn't ported: `Fluids::is_tag` answers
//! from vanilla's data pack (`data/minecraft/tags/fluid/`), which is what a vanilla server sends.

use crate::net::minecraft::tags::block_tags;
use crate::net::minecraft::world::level::material::fluids::Fluids;

/// `TagKey<Fluid>`: a fluid tag's name, under `minecraft:`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FluidTagKey(pub &'static str);

pub struct FluidTags;

impl FluidTags {
    pub const WATER: FluidTagKey = FluidTagKey("water");
    pub const LAVA: FluidTagKey = FluidTagKey("lava");
}

impl Fluids {
    /// `Fluid.is(TagKey<Fluid>)` (its holder's), from vanilla's data pack until the server's tags
    /// are ported.
    pub fn is_tag(&self, tag: FluidTagKey) -> bool {
        block_tags::tag_contains("fluid", tag.0, self.key())
    }
}
