//! net/minecraft/world/level/block/BubbleColumnBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct BubbleColumnBlock;

pub static CLASS: BlockClass = BlockClass {
    get_fluid_state: Some(BubbleColumnBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BubbleColumnBlock",
        &block::CLASS,
    )
};

impl BubbleColumnBlock {
    pub const DRAG_DOWN: &'static BooleanProperty = BlockStateProperties::DRAG;

    pub fn new(properties: Properties) -> BlockData {
        BubbleColumnBlock::new_with(
            properties,
            &CLASS,
            BubbleColumnBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(BubbleColumnBlock::DRAG_DOWN, true),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[BubbleColumnBlock::DRAG_DOWN]);
    }
}

impl BubbleColumnBlock {
    /// `getFluidState`: always still water.
    fn get_fluid_state(_block: &BlockData, _state: BlockState) -> FluidState {
        Fluids::WATER.get_source(false)
    }
}
