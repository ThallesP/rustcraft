//! net/minecraft/world/level/block/LiquidBlock.java
//!
//! Only what decides its blocks' states so far: its properties, its default state, its fluid and
//! which states tick. Not ported yet: the rest, including `stateCache` and `getFluidState`,
//! which wait for fluid states.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct LiquidBlock {
    pub fluid: Fluids,
}

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(LiquidBlock::is_randomly_ticking),
    ..BlockClass::new("net.minecraft.world.level.block.LiquidBlock", &block::CLASS)
};

impl LiquidBlock {
    pub const LEVEL: &'static IntegerProperty = BlockStateProperties::LEVEL;

    pub fn new(fluid: Fluids, properties: Properties) -> BlockData {
        LiquidBlock::new_with(
            fluid,
            properties,
            &CLASS,
            LiquidBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        fluid: Fluids,
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.set_fields(LiquidBlock { fluid });
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(LiquidBlock::LEVEL, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[LiquidBlock::LEVEL]);
    }

    /// `state.getFluidState().isRandomlyTicking()`: whatever its level, the state's fluid is its
    /// block's, flowing or still, which ticks when it's lava.
    fn is_randomly_ticking(block: &BlockData, _state: BlockState) -> bool {
        block.fields::<LiquidBlock>().fluid.is_randomly_ticking()
    }
}
