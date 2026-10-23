//! net/minecraft/world/level/block/HangingMossBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `propagatesSkylightDown` and its shape (shorter at the tip) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
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
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct HangingMossBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(HangingMossBlock::get_shape),
    propagates_skylight_down: Some(HangingMossBlock::propagates_skylight_down),
    ..BlockClass::new(
        "net.minecraft.world.level.block.HangingMossBlock",
        &block::CLASS,
    )
};

impl HangingMossBlock {
    pub const TIP: &'static BooleanProperty = BlockStateProperties::TIP;

    pub fn new(properties: Properties) -> BlockData {
        HangingMossBlock::new_with(
            properties,
            &CLASS,
            HangingMossBlock::create_block_state_definition,
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
                .set_value(HangingMossBlock::TIP, true),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[HangingMossBlock::TIP]);
    }
}

impl HangingMossBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}

static SHAPE_BASE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(14.0, 0.0, 16.0));
static SHAPE_TIP: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(14.0, 2.0, 16.0));

impl HangingMossBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        if state.get_value(HangingMossBlock::TIP) {
            SHAPE_TIP.clone()
        } else {
            SHAPE_BASE.clone()
        }
    }
}
