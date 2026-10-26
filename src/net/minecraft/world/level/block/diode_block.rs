//! net/minecraft/world/level/block/DiodeBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, its shape is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::{
    self, HorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::support_type::SupportType;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct DiodeBlock;

pub static CLASS: BlockClass = BlockClass {
    can_survive: Some(DiodeBlock::can_survive),
    is_signal_source: Some(DiodeBlock::is_signal_source),
    get_shape: Some(DiodeBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.DiodeBlock",
        &horizontal_directional_block::CLASS,
    )
};

impl DiodeBlock {
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        HorizontalDirectionalBlock::new_with(properties, class, definition)
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 2.0));

impl DiodeBlock {
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

impl DiodeBlock {
    /// `isSignalSource`: it is one.
    fn is_signal_source(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}

impl DiodeBlock {
    /// `canSurvive`: on what can hold it.
    fn can_survive(
        _block: &BlockData,
        _state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> bool {
        let below_pos = pos.below();
        DiodeBlock::can_survive_on(level, &below_pos, level.get_block_state(&below_pos))
    }

    /// `canSurviveOn(level, neighborPos, neighborState)`: a top rigid enough to hold it.
    pub fn can_survive_on(
        level: &dyn BlockGetter,
        neighbor_pos: &BlockPos,
        neighbor_state: BlockState,
    ) -> bool {
        neighbor_state.is_face_sturdy_for(level, neighbor_pos, Direction::UP, SupportType::RIGID)
    }
}
