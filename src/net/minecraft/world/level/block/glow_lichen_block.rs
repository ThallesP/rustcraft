//! net/minecraft/world/level/block/GlowLichenBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::multiface_block::MultifaceBlock;
use crate::net::minecraft::world::level::block::multiface_spreadeable_block::{
    self, MultifaceSpreadeableBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct GlowLichenBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.GlowLichenBlock",
    &multiface_spreadeable_block::CLASS,
);

impl GlowLichenBlock {
    pub fn new(properties: Properties) -> BlockData {
        GlowLichenBlock::new_with(
            properties,
            &CLASS,
            MultifaceBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        MultifaceSpreadeableBlock::new_with(properties, class, definition)
    }

    /// The light of lichen on any face.
    pub fn emission(light_emission: i32) -> impl Fn(BlockState) -> i32 + Send + Sync + 'static {
        move |state| {
            if MultifaceBlock::has_any_face(state) {
                light_emission
            } else {
                0
            }
        }
    }
}
