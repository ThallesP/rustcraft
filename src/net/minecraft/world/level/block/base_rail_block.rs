//! net/minecraft/world/level/block/BaseRailBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `isStraight`.
//!
//! Of its behaviour, `getFluidState` and its shape (flat, or half a block on a slope) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyValue;
use crate::net::minecraft::world::level::block::state::properties::rail_shape::RailShape;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct BaseRailBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(BaseRailBlock::get_shape),
    get_fluid_state: Some(BaseRailBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BaseRailBlock",
        &block::CLASS,
    )
};

impl BaseRailBlock {
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

impl BaseRailBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

static SHAPE_FLAT: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 2.0));
static SHAPE_SLOPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 8.0));

impl BaseRailBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        if BaseRailBlock::get_rail_shape(state).is_slope() {
            SHAPE_SLOPE.clone()
        } else {
            SHAPE_FLAT.clone()
        }
    }

    /// The value of `getShapeProperty()`, each subclass's `shape` property.
    pub fn get_rail_shape(state: BlockState) -> RailShape {
        let property = state
            .get_properties()
            .iter()
            .find(|property| property.get_name() == "shape")
            .expect("a rail has a shape");
        RailShape::from_comparable(state.get_comparable(*property))
    }
}
