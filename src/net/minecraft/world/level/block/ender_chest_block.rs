//! net/minecraft/world/level/block/EnderChestBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, `getFluidState` and its shape, and its portal particles (`animateTick`) are
//! ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::abstract_chest_block::{self, AbstractChestBlock};
use crate::net::minecraft::world::level::block::base_entity_block;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::entity::block_entity_ticker::BlockEntityTicker;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::entity::block_entity_types::BlockEntityTypes;
use crate::net::minecraft::world::level::block::entity::ender_chest_block_entity::EnderChestBlockEntity;
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
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

pub struct EnderChestBlock;

pub static CLASS: BlockClass = BlockClass {
    update_shape: Some(EnderChestBlock::update_shape),
    get_ticker: Some(EnderChestBlock::get_ticker),
    animate_tick: Some(EnderChestBlock::animate_tick),
    get_shape: Some(EnderChestBlock::get_shape),
    get_fluid_state: Some(EnderChestBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.EnderChestBlock",
        &abstract_chest_block::CLASS,
    )
};

impl EnderChestBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        EnderChestBlock::new_with(
            properties,
            &CLASS,
            EnderChestBlock::create_block_state_definition,
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
                .set_value(EnderChestBlock::FACING, Direction::NORTH)
                .set_value(EnderChestBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[EnderChestBlock::FACING, EnderChestBlock::WATERLOGGED]);
    }
}

impl EnderChestBlock {
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

impl EnderChestBlock {
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

impl EnderChestBlock {
    /// `animateTick`: three portal particles around it.
    fn animate_tick(
        _block: &BlockData,
        _state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        for _ in 0..3 {
            let flip_x = random.next_int_bounded(2) * 2 - 1;
            let flip_z = random.next_int_bounded(2) * 2 - 1;
            let x = f64::from(pos.get_x()) + 0.5 + 0.25 * f64::from(flip_x);
            let y = f64::from(pos.get_y() as f32 + random.next_float());
            let z = f64::from(pos.get_z()) + 0.5 + 0.25 * f64::from(flip_z);
            let xa = f64::from(random.next_float() * flip_x as f32);
            let ya = (f64::from(random.next_float()) - 0.5) * 0.125;
            let za = f64::from(random.next_float() * flip_z as f32);
            level.add_particle(ParticleTypes::PORTAL, x, y, z, xa, ya, za);
        }
    }
}

impl EnderChestBlock {
    /// `getTicker`: on the client, the lid's (`lidAnimateTick`).
    fn get_ticker(
        _block: &BlockData,
        _state: BlockState,
        r#type: BlockEntityType,
    ) -> Option<BlockEntityTicker> {
        base_entity_block::create_ticker_helper(
            r#type,
            BlockEntityTypes::ENDER_CHEST,
            EnderChestBlockEntity::lid_animate_tick,
        )
    }
}

impl EnderChestBlock {
    /// `updateShape`: its parent's. The water's tick it schedules when it's waterlogged goes
    /// nowhere on the client.
    #[allow(clippy::too_many_arguments)]
    fn update_shape(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction_to_neighbour: Direction,
        neighbour_pos: &BlockPos,
        neighbour_state: BlockState,
        random: &mut dyn RandomSource,
    ) -> BlockState {
        Block::super_update_shape(
            block,
            &CLASS,
            state,
            level,
            pos,
            direction_to_neighbour,
            neighbour_pos,
            neighbour_state,
            random,
        )
    }
}
