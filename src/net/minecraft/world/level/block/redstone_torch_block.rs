//! net/minecraft/world/level/block/RedstoneTorchBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, its specks (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::dust_particle_options::DustParticleOptions;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::base_torch_block::{self, BaseTorchBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct RedstoneTorchBlock;

pub static CLASS: BlockClass = BlockClass {
    is_signal_source: Some(RedstoneTorchBlock::is_signal_source),
    animate_tick: Some(RedstoneTorchBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.RedstoneTorchBlock",
        &base_torch_block::CLASS,
    )
};

impl RedstoneTorchBlock {
    pub const LIT: &'static BooleanProperty = BlockStateProperties::LIT;

    pub fn new(properties: Properties) -> BlockData {
        RedstoneTorchBlock::new_with(
            properties,
            &CLASS,
            RedstoneTorchBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseTorchBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(RedstoneTorchBlock::LIT, true),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[RedstoneTorchBlock::LIT]);
    }
}

impl RedstoneTorchBlock {
    /// `animateTick`: while lit, a redstone speck at its tip.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if state.get_value(RedstoneTorchBlock::LIT) {
            let x = f64::from(pos.get_x()) + 0.5 + (random.next_double() - 0.5) * 0.2;
            let y = f64::from(pos.get_y()) + 0.7 + (random.next_double() - 0.5) * 0.2;
            let z = f64::from(pos.get_z()) + 0.5 + (random.next_double() - 0.5) * 0.2;
            level.add_particle(DustParticleOptions::REDSTONE, x, y, z, 0.0, 0.0, 0.0);
        }
    }
}

impl RedstoneTorchBlock {
    /// `isSignalSource`: it is one.
    fn is_signal_source(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
