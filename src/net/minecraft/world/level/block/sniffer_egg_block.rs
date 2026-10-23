//! net/minecraft/world/level/block/SnifferEggBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shape is ported.

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

pub struct SnifferEggBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(SnifferEggBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SnifferEggBlock",
        &block::CLASS,
    )
};

impl SnifferEggBlock {
    pub const HATCH: &'static IntegerProperty = BlockStateProperties::HATCH;

    pub fn new(properties: Properties) -> BlockData {
        SnifferEggBlock::new_with(
            properties,
            &CLASS,
            SnifferEggBlock::create_block_state_definition,
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
                .set_value(SnifferEggBlock::HATCH, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[SnifferEggBlock::HATCH]);
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column_xz(14.0, 12.0, 0.0, 16.0));

impl SnifferEggBlock {
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
