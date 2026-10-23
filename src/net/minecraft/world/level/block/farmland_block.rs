//! net/minecraft/world/level/block/FarmlandBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `baseBlock`.
//!
//! Of its behaviour, `useShapeForLightOcclusion` is ported, for the light engine.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct FarmlandBlock;

pub static CLASS: BlockClass = BlockClass {
    use_shape_for_light_occlusion: Some(FarmlandBlock::use_shape_for_light_occlusion),
    ..BlockClass::new(
        "net.minecraft.world.level.block.FarmlandBlock",
        &block::CLASS,
    )
};

impl FarmlandBlock {
    pub const MOISTURE: &'static IntegerProperty = BlockStateProperties::MOISTURE;

    pub fn new(properties: Properties) -> BlockData {
        FarmlandBlock::new_with(
            properties,
            &CLASS,
            FarmlandBlock::create_block_state_definition,
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
                .set_value(FarmlandBlock::MOISTURE, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[FarmlandBlock::MOISTURE]);
    }
}

impl FarmlandBlock {
    /// `useShapeForLightOcclusion`.
    fn use_shape_for_light_occlusion(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
