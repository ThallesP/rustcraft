//! net/minecraft/world/level/block/PointedDripstoneBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour, and its
//! constructor's `blockToGrowOn`.
//!
//! Of its behaviour, its drips (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Axis, AxisDirection, Direction};
use crate::net::minecraft::core::particles::particle_options::ParticleOptions;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::tags::fluid_tags::FluidTags;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::speleothem_block::{self, SpeleothemBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::speleothem_thickness::SpeleothemThickness;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::level_height_accessor::LevelHeightAccessor;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct PointedDripstoneBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(PointedDripstoneBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.PointedDripstoneBlock",
        &speleothem_block::CLASS,
    )
};

impl PointedDripstoneBlock {
    pub fn new(properties: Properties) -> BlockData {
        PointedDripstoneBlock::new_with(
            properties,
            &CLASS,
            SpeleothemBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        SpeleothemBlock::new_with(properties, class, definition)
    }
}

impl PointedDripstoneBlock {
    /// `animateTick`: a free hanging tip drips now and then: the fluid above its root, or rarely what the dimension drips (`DEFAULT_DRIPSTONE_PARTICLE`).
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if PointedDripstoneBlock::is_free_hanging_stalactite(state) {
            let random_value = random.next_float();
            if random_value <= 0.12
                && let Some((above_pos, fluid)) =
                    PointedDripstoneBlock::get_fluid_above_stalactite(level, pos, state)
                && (random_value < 0.02 || PointedDripstoneBlock::can_fill_cauldron(fluid))
            {
                PointedDripstoneBlock::spawn_drip_particle(level, pos, state, fluid, &above_pos);
            }
        }
    }

    /// `SpeleothemBlock.isFreeHangingStalactite`: a tip pointing down, not in water.
    fn is_free_hanging_stalactite(state: BlockState) -> bool {
        PointedDripstoneBlock::is_stalactite(state)
            && state.get_value(SpeleothemBlock::THICKNESS) == SpeleothemThickness::TIP
            && !state.get_value(SpeleothemBlock::WATERLOGGED)
    }

    /// `SpeleothemBlock.isStalactite`.
    fn is_stalactite(state: BlockState) -> bool {
        PointedDripstoneBlock::is_speleothem_with_direction(state, Direction::DOWN)
    }

    /// `SpeleothemBlock.isSpeleothemWithDirection`.
    fn is_speleothem_with_direction(block_state: BlockState, tip_direction: Direction) -> bool {
        block_state.is_tag(BlockTags::SPELEOTHEMS)
            && block_state.get_value(SpeleothemBlock::TIP_DIRECTION) == tip_direction
    }

    /// `getFluidAboveStalactite`: where the fluid above its root is, and which fluid (water over
    /// mud, where water doesn't evaporate), as Java's `FluidInfo`.
    fn get_fluid_above_stalactite(
        level: &LevelWithEffects<'_>,
        stalactite_pos: &BlockPos,
        stalactite_state: BlockState,
    ) -> Option<(BlockPos, Fluids)> {
        if !PointedDripstoneBlock::is_stalactite(stalactite_state) {
            return None;
        }
        let root_pos =
            PointedDripstoneBlock::find_root_block(level, stalactite_pos, stalactite_state, 11)?;
        let above_pos = root_pos.above();
        let above_state = level.get_block_state(&above_pos);
        let fluid = if above_state.is(Blocks::MUD)
            && !level.get_environment_value(&EnvironmentAttributes::WATER_EVAPORATES, &above_pos)
        {
            Fluids::WATER
        } else {
            level.get_fluid_state(&above_pos).get_type()
        };
        Some((above_pos, fluid))
    }

    fn can_fill_cauldron(fluid_above: Fluids) -> bool {
        fluid_above == Fluids::LAVA || fluid_above == Fluids::WATER
    }

    fn spawn_drip_particle(
        level: &LevelWithEffects<'_>,
        stalactite_tip_pos: &BlockPos,
        stalactite_tip_state: BlockState,
        fluid_above: Fluids,
        pos_above: &BlockPos,
    ) {
        let offset = stalactite_tip_state.get_offset(stalactite_tip_pos);
        let x = f64::from(stalactite_tip_pos.get_x()) + 0.5 + offset.x;
        let y = f64::from(stalactite_tip_pos.get_y())
            + speleothem_block::SHAPE_TIP_DOWN.min(Axis::Y)
            - 0.0625;
        let z = f64::from(stalactite_tip_pos.get_z()) + 0.5 + offset.z;
        let drip_particle = PointedDripstoneBlock::get_drip_particle(level, fluid_above, pos_above);
        level.add_particle(drip_particle, x, y, z, 0.0, 0.0, 0.0);
    }

    /// `findRootBlock`: the block past the dripstone it's part of, up to `max_search_length`.
    fn find_root_block(
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        drip_stone_state: BlockState,
        max_search_length: i32,
    ) -> Option<BlockPos> {
        let tip_direction = drip_stone_state.get_value(SpeleothemBlock::TIP_DIRECTION);
        let this = drip_stone_state.get_block();
        PointedDripstoneBlock::find_block_vertical(
            level,
            pos,
            tip_direction.get_opposite().get_axis_direction(),
            |_, state| {
                state.is(this) && state.get_value(SpeleothemBlock::TIP_DIRECTION) == tip_direction
            },
            |state| !state.is(this),
            max_search_length,
        )
    }

    /// `SpeleothemBlock.findBlockVertical`: the first block that way the target predicate takes,
    /// along blocks the path predicate takes, within `max_steps`.
    fn find_block_vertical(
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        axis_direction: AxisDirection,
        path_predicate: impl Fn(&BlockPos, BlockState) -> bool,
        target_predicate: impl Fn(BlockState) -> bool,
        max_steps: i32,
    ) -> Option<BlockPos> {
        let direction = Direction::get(axis_direction, Axis::Y);
        let mut mutable_pos = pos.mutable();

        for _ in 1..max_steps {
            mutable_pos.r#move(direction);
            let state = level.get_block_state(&mutable_pos);
            if target_predicate(state) {
                return Some(mutable_pos.immutable());
            }

            if level.is_outside_build_height(mutable_pos.get_y())
                || !path_predicate(&mutable_pos, state)
            {
                return None;
            }
        }

        None
    }

    /// `getDripParticle`: the dimension's drip where no fluid is above
    /// (`DEFAULT_DRIPSTONE_PARTICLE`, an attribute of a type not ported, so the value vanilla's
    /// dimension types set: the nether's lava, the others' water), and the fluid's otherwise.
    fn get_drip_particle(
        level: &LevelWithEffects<'_>,
        fluid_above: Fluids,
        _pos_above: &BlockPos,
    ) -> ParticleOptions {
        if fluid_above.is_empty() {
            if *level
                .dimension_type_registration()
                .unwrap_key()
                .expect("a registry's holder has its key")
                .identifier()
                == Identifier::with_default_namespace("the_nether")
            {
                ParticleTypes::DRIPPING_DRIPSTONE_LAVA.into()
            } else {
                ParticleTypes::DRIPPING_DRIPSTONE_WATER.into()
            }
        } else if fluid_above.is_tag(FluidTags::LAVA) {
            ParticleTypes::DRIPPING_DRIPSTONE_LAVA.into()
        } else {
            ParticleTypes::DRIPPING_DRIPSTONE_WATER.into()
        }
    }
}
