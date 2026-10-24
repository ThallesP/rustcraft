//! net/minecraft/world/level/block/SculkSensorBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, `getFluidState`, `useShapeForLightOcclusion` and its shape (half a block), and
//! its dust (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::dust_color_transition_options::DustColorTransitionOptions;
use crate::net::minecraft::util::random_source::RandomSource;
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
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::properties::sculk_sensor_phase::SculkSensorPhase;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct SculkSensorBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(SculkSensorBlock::animate_tick),
    get_shape: Some(SculkSensorBlock::get_shape),
    use_shape_for_light_occlusion: Some(SculkSensorBlock::use_shape_for_light_occlusion),
    get_fluid_state: Some(SculkSensorBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SculkSensorBlock",
        &base_entity_block::CLASS,
    )
};

impl SculkSensorBlock {
    pub const PHASE: &'static EnumProperty<SculkSensorPhase> =
        BlockStateProperties::SCULK_SENSOR_PHASE;
    pub const POWER: &'static IntegerProperty = BlockStateProperties::POWER;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        SculkSensorBlock::new_with(
            properties,
            &CLASS,
            SculkSensorBlock::create_block_state_definition,
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
                .set_value(SculkSensorBlock::PHASE, SculkSensorPhase::INACTIVE)
                .set_value(SculkSensorBlock::POWER, 0)
                .set_value(SculkSensorBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            SculkSensorBlock::PHASE,
            SculkSensorBlock::POWER,
            SculkSensorBlock::WATERLOGGED,
        ]);
    }

    pub fn get_phase(state: BlockState) -> SculkSensorPhase {
        state.get_value(SculkSensorBlock::PHASE)
    }
}

impl SculkSensorBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

impl SculkSensorBlock {
    /// `useShapeForLightOcclusion`.
    fn use_shape_for_light_occlusion(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 8.0));

impl SculkSensorBlock {
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

impl SculkSensorBlock {
    /// `animateTick`: active, now and then dust going from sculk to redstone at one of its sides.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if SculkSensorBlock::get_phase(state) == SculkSensorPhase::ACTIVE {
            let dir = Direction::get_random(random);
            if dir != Direction::UP && dir != Direction::DOWN {
                let x = f64::from(pos.get_x())
                    + 0.5
                    + if dir.get_step_x() == 0 {
                        0.5 - random.next_double()
                    } else {
                        f64::from(dir.get_step_x()) * 0.6
                    };
                let y = f64::from(pos.get_y()) + 0.25;
                let z = f64::from(pos.get_z())
                    + 0.5
                    + if dir.get_step_z() == 0 {
                        0.5 - random.next_double()
                    } else {
                        f64::from(dir.get_step_z()) * 0.6
                    };
                let ya = f64::from(random.next_float()) * 0.04;
                level.add_particle(
                    DustColorTransitionOptions::SCULK_TO_REDSTONE,
                    x,
                    y,
                    z,
                    0.0,
                    ya,
                    0.0,
                );
            }
        }
    }
}
