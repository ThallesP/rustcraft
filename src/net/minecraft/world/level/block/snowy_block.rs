//! net/minecraft/world/level/block/SnowyBlock.java
//!
//! Its properties and default state, and `updateShape`: snowy while snow is on it. Not ported
//! yet: the rest of its behaviour (`getStateForPlacement`).

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct SnowyBlock;

pub static CLASS: BlockClass = BlockClass {
    update_shape: Some(SnowyBlock::update_shape),
    ..BlockClass::new("net.minecraft.world.level.block.SnowyBlock", &block::CLASS)
};

impl SnowyBlock {
    pub const SNOWY: &'static BooleanProperty = BlockStateProperties::SNOWY;

    pub fn new(properties: Properties) -> BlockData {
        SnowyBlock::new_with(
            properties,
            &CLASS,
            SnowyBlock::create_block_state_definition,
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
                .set_value(SnowyBlock::SNOWY, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[SnowyBlock::SNOWY]);
    }
}

impl SnowyBlock {
    /// `updateShape`: snowy from what's above it.
    #[allow(clippy::too_many_arguments)]
    fn update_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        direction_to_neighbour: Direction,
        _neighbour_pos: &BlockPos,
        neighbour_state: BlockState,
        _random: &mut dyn RandomSource,
    ) -> BlockState {
        if direction_to_neighbour == Direction::UP {
            state.set_value(
                SnowyBlock::SNOWY,
                SnowyBlock::is_snowy_setting(neighbour_state),
            )
        } else {
            state
        }
    }

    /// `isSnowySetting`: snow, as the `snow` tag has it.
    fn is_snowy_setting(above_state: BlockState) -> bool {
        above_state.is_tag(BlockTags::SNOW)
    }
}
