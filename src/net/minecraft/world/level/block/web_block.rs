//! net/minecraft/world/level/block/WebBlock.java
//!
//! Its blocks' states are its parent's. Of its behaviour, `entityInside`: an entity in a cobweb
//! is stuck, to a quarter of its speed sideways and a twentieth up and down (half and a quarter
//! with weaving, an effect, which isn't ported).

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::entity::inside_block_effect_applier::InsideBlockEffectApplier;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::level::Level;
use crate::net::minecraft::world::phys::vec3::Vec3;

pub struct WebBlock;

pub static CLASS: BlockClass = BlockClass {
    entity_inside: Some(WebBlock::entity_inside),
    ..BlockClass::new("net.minecraft.world.level.block.WebBlock", &block::CLASS)
};

impl WebBlock {
    pub fn new(properties: Properties) -> BlockData {
        WebBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }

    fn entity_inside(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn Level,
        _pos: &BlockPos,
        entity: &mut dyn EntityClass,
        _effect_applier: &mut dyn InsideBlockEffectApplier,
        _is_precise: bool,
    ) {
        // A living entity with weaving (`hasEffect(MobEffects.WEAVING)`, never: effects aren't
        // ported) would be stuck at (0.5, 0.25, 0.5).
        let speed_multiplier = Vec3::new(0.25, f64::from(0.05_f32), 0.25);
        entity.make_stuck_in_block(state, speed_multiplier);
    }
}
