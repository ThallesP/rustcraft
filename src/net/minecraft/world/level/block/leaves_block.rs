//! net/minecraft/world/level/block/LeavesBlock.java
//!
//! What decides its blocks' states (its properties, its default state and which of its states
//! tick) and its support shape. Not ported yet: the rest of its behaviour, and its constructor's
//! `ambientLeavesBlockSoundPlayer`.
//!
//! Of its behaviour, `getLightDampening` and `skipRendering` (the faces between leaves, unless
//! they're drawn cut out) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct LeavesBlock;

/// `cutoutLeaves`, which the options set: whether leaves draw their cutouts or as solid blocks
/// that hide the leaves next to them.
static CUTOUT_LEAVES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

pub static CLASS: BlockClass = BlockClass {
    skip_rendering: Some(LeavesBlock::skip_rendering),
    get_light_dampening: Some(LeavesBlock::get_light_dampening),
    get_fluid_state: Some(LeavesBlock::get_fluid_state),
    is_randomly_ticking: Some(LeavesBlock::is_randomly_ticking),
    get_block_support_shape: Some(LeavesBlock::get_block_support_shape),
    ..BlockClass::new("net.minecraft.world.level.block.LeavesBlock", &block::CLASS)
};

impl LeavesBlock {
    pub const DISTANCE: &'static IntegerProperty = BlockStateProperties::DISTANCE;
    pub const PERSISTENT: &'static BooleanProperty = BlockStateProperties::PERSISTENT;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn set_cutout_leaves(cutout_leaves: bool) {
        CUTOUT_LEAVES.store(cutout_leaves, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn cutout_leaves() -> bool {
        CUTOUT_LEAVES.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn new(properties: Properties) -> BlockData {
        LeavesBlock::new_with(
            properties,
            &CLASS,
            LeavesBlock::create_block_state_definition,
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
                .set_value(LeavesBlock::DISTANCE, 7)
                .set_value(LeavesBlock::PERSISTENT, false)
                .set_value(LeavesBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            LeavesBlock::DISTANCE,
            LeavesBlock::PERSISTENT,
            LeavesBlock::WATERLOGGED,
        ]);
    }

    fn is_randomly_ticking(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(LeavesBlock::DISTANCE) == 7 && !state.get_value(LeavesBlock::PERSISTENT)
    }
}

impl LeavesBlock {
    /// `getBlockSupportShape`: none, so nothing is sturdy on leaves.
    fn get_block_support_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> VoxelShape {
        Shapes::empty()
    }

    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

impl LeavesBlock {
    /// `getLightDampening`.
    fn get_light_dampening(_block: &BlockData, _state: BlockState) -> i32 {
        1
    }
}

impl LeavesBlock {
    fn skip_rendering(
        _block: &BlockData,
        _state: BlockState,
        neighbor_state: BlockState,
        _direction: Direction,
    ) -> bool {
        !LeavesBlock::cutout_leaves() && neighbor_state.class_is_leaves()
    }
}
