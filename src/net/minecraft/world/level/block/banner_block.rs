//! net/minecraft/world/level/block/BannerBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `color`.
//!
//! Of its behaviour, its shape (its pole) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::abstract_banner_block::{
    self, AbstractBannerBlock,
};
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
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

pub struct BannerBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(BannerBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BannerBlock",
        &abstract_banner_block::CLASS,
    )
};

impl BannerBlock {
    pub const ROTATION: &'static IntegerProperty = BlockStateProperties::ROTATION_16;

    pub fn new(properties: Properties) -> BlockData {
        BannerBlock::new_with(
            properties,
            &CLASS,
            BannerBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = AbstractBannerBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(BannerBlock::ROTATION, 8),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[BannerBlock::ROTATION]);
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(8.0, 0.0, 16.0));

impl BannerBlock {
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
