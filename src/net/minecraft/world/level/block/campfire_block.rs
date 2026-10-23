//! net/minecraft/world/level/block/CampfireBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `spawnParticles` and `fireDamage`.
//!
//! Of its behaviour, `getFluidState` and its shape are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct CampfireBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(CampfireBlock::get_shape),
    get_fluid_state: Some(CampfireBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.CampfireBlock",
        &base_entity_block::CLASS,
    )
};

impl CampfireBlock {
    pub const LIT: &'static BooleanProperty = BlockStateProperties::LIT;
    pub const SIGNAL_FIRE: &'static BooleanProperty = BlockStateProperties::SIGNAL_FIRE;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::HORIZONTAL_FACING;

    pub fn new(properties: Properties) -> BlockData {
        CampfireBlock::new_with(
            properties,
            &CLASS,
            CampfireBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseEntityBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(CampfireBlock::LIT, true)
                .set_value(CampfireBlock::SIGNAL_FIRE, false)
                .set_value(CampfireBlock::WATERLOGGED, false)
                .set_value(CampfireBlock::FACING, Direction::NORTH),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            CampfireBlock::LIT,
            CampfireBlock::SIGNAL_FIRE,
            CampfireBlock::WATERLOGGED,
            CampfireBlock::FACING,
        ]);
    }
}

impl CampfireBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 7.0));

impl CampfireBlock {
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
