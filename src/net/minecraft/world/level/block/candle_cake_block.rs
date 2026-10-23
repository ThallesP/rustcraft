//! net/minecraft/world/level/block/CandleCakeBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `block`.
//!
//! Of its behaviour, its shape (a candle on a cake) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::abstract_candle_block::{
    self, AbstractCandleBlock,
};
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct CandleCakeBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(CandleCakeBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.CandleCakeBlock",
        &abstract_candle_block::CLASS,
    )
};

impl CandleCakeBlock {
    pub const LIT: &'static BooleanProperty = AbstractCandleBlock::LIT;

    pub fn new(properties: Properties) -> BlockData {
        CandleCakeBlock::new_with(
            properties,
            &CLASS,
            CandleCakeBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = AbstractCandleBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(CandleCakeBlock::LIT, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[CandleCakeBlock::LIT]);
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| {
    Shapes::or(
        &Block::column(2.0, 8.0, 14.0),
        &Block::column(14.0, 0.0, 8.0),
    )
});

impl CandleCakeBlock {
    fn get_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE.clone()
    }
}
