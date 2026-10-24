//! net/minecraft/world/level/block/LeavesBlock.java
//!
//! What decides its blocks' states (its properties, its default state and which of its states tick)
//! and its support shape. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, `getLightDampening` and `skipRendering` (the faces between leaves, unless
//! they're drawn cut out), and its drips and sound (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::particle_options::ParticleOptions;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::particle_utils::ParticleUtils;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::sounds::ambient_leaves_block_sound_player::AmbientLeavesBlockSoundPlayer;
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

/// What its constructors take down the chain: the ambient sound
/// (`ambientLeavesBlockSoundPlayer`), and for falling leaves (`FallingParticlesLeavesBlock`) how
/// often one falls (`leafParticleChance`) and, for untinted ones, which particle
/// (`UntintedParticleLeavesBlock.leafParticle`; none for tinted leaves, whose colour is the
/// block's).
#[derive(Clone, Copy, Debug)]
pub struct LeavesBlockFields {
    pub ambient_leaves_block_sound_player: AmbientLeavesBlockSoundPlayer,
    pub leaf_particle_chance: f32,
    pub leaf_particle: Option<ParticleOptions>,
}

/// `cutoutLeaves`, which the options set: whether leaves draw their cutouts or as solid blocks
/// that hide the leaves next to them.
static CUTOUT_LEAVES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(LeavesBlock::animate_tick),
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

    /// `LeavesBlock(ambientLeavesBlockSoundPlayer, properties)`.
    pub fn new(
        ambient_leaves_block_sound_player: AmbientLeavesBlockSoundPlayer,
        properties: Properties,
    ) -> BlockData {
        LeavesBlock::new_with(
            properties,
            &CLASS,
            LeavesBlock::create_block_state_definition,
            ambient_leaves_block_sound_player,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
        ambient_leaves_block_sound_player: AmbientLeavesBlockSoundPlayer,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.set_fields(LeavesBlockFields {
            ambient_leaves_block_sound_player,
            leaf_particle_chance: 0.0,
            leaf_particle: None,
        });
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

impl LeavesBlock {
    /// `animateTick`: water dripping from under it in the rain, and its ambient sound (`AmbientLeavesBlockSoundPlayer`).
    fn animate_tick(
        block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        Block::super_animate_tick(block, &CLASS, state, level, pos, random);
        let below = pos.below();
        let below_state = level.get_block_state(&below);
        LeavesBlock::make_dripping_water_particles(level, pos, random, below_state, &below);
        block
            .fields::<LeavesBlockFields>()
            .ambient_leaves_block_sound_player
            .play_ambient_leaves_sounds(level, pos, block, random);
    }

    fn make_dripping_water_particles(
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
        below_state: BlockState,
        below: &BlockPos,
    ) {
        if level.is_raining_at(&pos.above())
            && random.next_int_bounded(15) == 1
            && (!below_state.can_occlude()
                || !below_state.is_face_sturdy(&**level, below, Direction::UP))
        {
            ParticleUtils::spawn_particle_below(
                level,
                pos,
                random,
                ParticleTypes::DRIPPING_WATER.into(),
            );
        }
    }
}
