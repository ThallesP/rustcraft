//! net/minecraft/world/level/block/ChestBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `blockEntityType`, `openSound` and `closeSound`.
//!
//! Of its behaviour, `getFluidState` and its shape (a sixteenth in from each side, and to the other
//! half of a double chest) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::abstract_chest_block::{self, AbstractChestBlock};
use crate::net::minecraft::world::level::block::base_entity_block;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::entity::block_entity_ticker::BlockEntityTicker;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::entity::block_entity_types::BY_BLOCK;
use crate::net::minecraft::world::level::block::entity::chest_block_entity::ChestBlockEntity;
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::chest_type::ChestType;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct ChestBlock;

pub static CLASS: BlockClass = BlockClass {
    get_ticker: Some(ChestBlock::get_ticker),
    get_shape: Some(ChestBlock::get_shape),
    get_fluid_state: Some(ChestBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.ChestBlock",
        &abstract_chest_block::CLASS,
    )
};

impl ChestBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;
    pub const TYPE: &'static EnumProperty<ChestType> = BlockStateProperties::CHEST_TYPE;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        ChestBlock::new_with(
            properties,
            &CLASS,
            ChestBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = AbstractChestBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(ChestBlock::FACING, Direction::NORTH)
                .set_value(ChestBlock::TYPE, ChestType::SINGLE)
                .set_value(ChestBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            ChestBlock::FACING,
            ChestBlock::TYPE,
            ChestBlock::WATERLOGGED,
        ]);
    }
}

impl ChestBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(14.0, 0.0, 14.0));
static HALF_SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal(&Block::box_z(14.0, 0.0, 14.0, 0.0, 15.0)));

impl ChestBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        match state.get_value(ChestBlock::TYPE) {
            ChestType::SINGLE => SHAPE.clone(),
            ChestType::LEFT | ChestType::RIGHT => {
                Shapes::get(&HALF_SHAPES, ChestBlock::get_connected_direction(state)).clone()
            }
        }
    }

    /// `getConnectedDirection`: toward the other half.
    pub fn get_connected_direction(state: BlockState) -> Direction {
        let facing = state.get_value(ChestBlock::FACING);
        if state.get_value(ChestBlock::TYPE) == ChestType::LEFT {
            facing.get_clock_wise()
        } else {
            facing.get_counter_clock_wise()
        }
    }
}

impl ChestBlock {
    /// `getTicker`: on the client, the lid's (`lidAnimateTick`) for the type of block entity its block makes
    /// (`blockEntityType()`).
    fn get_ticker(
        _block: &BlockData,
        state: BlockState,
        r#type: BlockEntityType,
    ) -> Option<BlockEntityTicker> {
        base_entity_block::create_ticker_helper(
            r#type,
            BY_BLOCK[state.get_block().0 as usize]?,
            ChestBlockEntity::lid_animate_tick,
        )
    }
}
