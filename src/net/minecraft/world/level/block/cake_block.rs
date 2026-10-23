//! net/minecraft/world/level/block/CakeBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shape (shorter by two sixteenths a bite) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct CakeBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(CakeBlock::get_shape),
    ..BlockClass::new("net.minecraft.world.level.block.CakeBlock", &block::CLASS)
};

impl CakeBlock {
    pub const BITES: &'static IntegerProperty = BlockStateProperties::BITES;

    pub fn new(properties: Properties) -> BlockData {
        CakeBlock::new_with(properties, &CLASS, CakeBlock::create_block_state_definition)
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
                .set_value(CakeBlock::BITES, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[CakeBlock::BITES]);
    }
}

static SHAPES: LazyLock<Vec<VoxelShape>> = LazyLock::new(|| {
    Block::boxes(6, |bite| {
        Block::r#box(f64::from(1 + bite * 2), 0.0, 1.0, 15.0, 8.0, 15.0)
    })
});

impl CakeBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPES[state.get_value(CakeBlock::BITES) as usize].clone()
    }
}
