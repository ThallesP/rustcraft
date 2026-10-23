//! net/minecraft/world/level/block/CocoaBlock.java
//!
//! Only what decides its blocks' states so far: its properties, its default state and which of its
//! states tick. Not ported yet: its behaviour.
//!
//! Of its behaviour, which states tick and its shape (bigger as it grows, hanging from the log it
//! faces) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::{
    self, HorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct CocoaBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(CocoaBlock::get_shape),
    is_randomly_ticking: Some(CocoaBlock::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.CocoaBlock",
        &horizontal_directional_block::CLASS,
    )
};

impl CocoaBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_2;

    pub fn new(properties: Properties) -> BlockData {
        CocoaBlock::new_with(
            properties,
            &CLASS,
            CocoaBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = HorizontalDirectionalBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(HorizontalDirectionalBlock::FACING, Direction::NORTH)
                .set_value(CocoaBlock::AGE, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[HorizontalDirectionalBlock::FACING, CocoaBlock::AGE]);
    }

    fn is_randomly_ticking(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(CocoaBlock::AGE) < 2
    }
}

static SHAPES: LazyLock<Vec<Vec<(Direction, VoxelShape)>>> = LazyLock::new(|| {
    (0..=2)
        .map(|i| {
            Shapes::rotate_horizontal(
                &Block::column(f64::from(4 + i * 2), f64::from(7 - i * 2), 12.0)
                    .move_xyz(0.0, 0.0, f64::from(i - 5) / 16.0)
                    .optimize(),
            )
        })
        .collect()
});

impl CocoaBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(
            &SHAPES[state.get_value(CocoaBlock::AGE) as usize],
            state.get_value(HorizontalDirectionalBlock::FACING),
        )
        .clone()
    }
}
