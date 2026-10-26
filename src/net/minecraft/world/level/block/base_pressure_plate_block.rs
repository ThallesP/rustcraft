//! net/minecraft/world/level/block/BasePressurePlateBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `type`.
//!
//! Of its behaviour, its shape (pressed down half a sixteenth while it gives a signal) is ported,
//! with each subclass's `getSignalForState`.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::pressure_plate_block::PressurePlateBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::weighted_pressure_plate_block::{
    self, WeightedPressurePlateBlock,
};
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct BasePressurePlateBlock;

pub static CLASS: BlockClass = BlockClass {
    is_signal_source: Some(BasePressurePlateBlock::is_signal_source),
    get_shape: Some(BasePressurePlateBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BasePressurePlateBlock",
        &block::CLASS,
    )
};

impl BasePressurePlateBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

static SHAPE_PRESSED: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(14.0, 0.0, 0.5));
static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(14.0, 0.0, 1.0));

impl BasePressurePlateBlock {
    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        if BasePressurePlateBlock::get_signal_for_state(block, state) > 0 {
            SHAPE_PRESSED.clone()
        } else {
            SHAPE.clone()
        }
    }

    /// `getSignalForState`, its subclasses' (abstract here): a weighted plate's power, or 15
    /// for a pressed plate.
    fn get_signal_for_state(block: &BlockData, state: BlockState) -> i32 {
        if std::ptr::eq(block.class(), &weighted_pressure_plate_block::CLASS) {
            state.get_value(WeightedPressurePlateBlock::POWER)
        } else if state.get_value(PressurePlateBlock::POWERED) {
            15
        } else {
            0
        }
    }
}

impl BasePressurePlateBlock {
    /// `isSignalSource`: it is one.
    fn is_signal_source(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
