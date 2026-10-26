//! net/minecraft/world/level/block/HugeMushroomBlock.java
//!
//! What decides its blocks' states (its properties and its default state), and `updateShape`: no
//! face toward the same mushroom block. Not ported yet: the rest of its behaviour.

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::pipe_block::PipeBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct HugeMushroomBlock;

pub static CLASS: BlockClass = BlockClass {
    update_shape: Some(HugeMushroomBlock::update_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.HugeMushroomBlock",
        &block::CLASS,
    )
};

impl HugeMushroomBlock {
    pub const NORTH: &'static BooleanProperty = PipeBlock::NORTH;
    pub const EAST: &'static BooleanProperty = PipeBlock::EAST;
    pub const SOUTH: &'static BooleanProperty = PipeBlock::SOUTH;
    pub const WEST: &'static BooleanProperty = PipeBlock::WEST;
    pub const UP: &'static BooleanProperty = PipeBlock::UP;
    pub const DOWN: &'static BooleanProperty = PipeBlock::DOWN;

    pub fn new(properties: Properties) -> BlockData {
        HugeMushroomBlock::new_with(
            properties,
            &CLASS,
            HugeMushroomBlock::create_block_state_definition,
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
                .set_value(HugeMushroomBlock::NORTH, true)
                .set_value(HugeMushroomBlock::EAST, true)
                .set_value(HugeMushroomBlock::SOUTH, true)
                .set_value(HugeMushroomBlock::WEST, true)
                .set_value(HugeMushroomBlock::UP, true)
                .set_value(HugeMushroomBlock::DOWN, true),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            HugeMushroomBlock::UP,
            HugeMushroomBlock::DOWN,
            HugeMushroomBlock::NORTH,
            HugeMushroomBlock::EAST,
            HugeMushroomBlock::SOUTH,
            HugeMushroomBlock::WEST,
        ]);
    }
}

impl HugeMushroomBlock {
    /// `updateShape`: no face toward the same mushroom block that way; its parent's otherwise.
    #[allow(clippy::too_many_arguments)]
    fn update_shape(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction_to_neighbour: Direction,
        neighbour_pos: &BlockPos,
        neighbour_state: BlockState,
        random: &mut dyn RandomSource,
    ) -> BlockState {
        if std::ptr::eq(&*neighbour_state.get_block(), block) {
            state.set_value(
                PipeBlock::PROPERTY_BY_DIRECTION[direction_to_neighbour.ordinal() as usize],
                false,
            )
        } else {
            Block::super_update_shape(
                block,
                &CLASS,
                state,
                level,
                pos,
                direction_to_neighbour,
                neighbour_pos,
                neighbour_state,
                random,
            )
        }
    }
}
