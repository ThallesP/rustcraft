//! net/minecraft/world/level/block/TallSeagrassBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, `getFluidState` and its shape are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::double_plant_block::{self, DoublePlantBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::double_block_half::DoubleBlockHalf;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct TallSeagrassBlock;

pub static CLASS: BlockClass = BlockClass {
    may_place_on: Some(TallSeagrassBlock::may_place_on),
    get_shape: Some(TallSeagrassBlock::get_shape),
    get_fluid_state: Some(TallSeagrassBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.TallSeagrassBlock",
        &double_plant_block::CLASS,
    )
};

impl TallSeagrassBlock {
    pub const HALF: &'static EnumProperty<DoubleBlockHalf> = DoublePlantBlock::HALF;

    pub fn new(properties: Properties) -> BlockData {
        TallSeagrassBlock::new_with(
            properties,
            &CLASS,
            DoublePlantBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        DoublePlantBlock::new_with(properties, class, definition)
    }
}

impl TallSeagrassBlock {
    /// `getFluidState`: always still water.
    fn get_fluid_state(_block: &BlockData, _state: BlockState) -> FluidState {
        Fluids::WATER.get_source(false)
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(12.0, 0.0, 16.0));

impl TallSeagrassBlock {
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

impl TallSeagrassBlock {
    /// `mayPlaceOn`: on a sturdy top that may hold seagrass.
    fn may_place_on(
        _block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> bool {
        state.is_face_sturdy(level, pos, Direction::UP)
            && !state.is_tag(BlockTags::CANNOT_SUPPORT_SEAGRASS)
    }
}
