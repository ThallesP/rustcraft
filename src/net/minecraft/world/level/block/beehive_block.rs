//! net/minecraft/world/level/block/BeehiveBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, its honey's drips (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct BeehiveBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(BeehiveBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BeehiveBlock",
        &base_entity_block::CLASS,
    )
};

impl BeehiveBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;
    pub const HONEY_LEVEL: &'static IntegerProperty = BlockStateProperties::LEVEL_HONEY;

    pub fn new(properties: Properties) -> BlockData {
        BeehiveBlock::new_with(
            properties,
            &CLASS,
            BeehiveBlock::create_block_state_definition,
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
                .set_value(BeehiveBlock::HONEY_LEVEL, 0)
                .set_value(BeehiveBlock::FACING, Direction::NORTH),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[BeehiveBlock::HONEY_LEVEL, BeehiveBlock::FACING]);
    }
}

impl BeehiveBlock {
    /// `animateTick`: honey dripping from a full hive.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if state.get_value(BeehiveBlock::HONEY_LEVEL) >= 5 {
            // Java's loop draws its bound again each time round.
            let mut i = 0;
            while i < random.next_int_bounded(1) + 1 {
                BeehiveBlock::try_spawn_drip_particles(level, pos, state);
                i += 1;
            }
        }
    }

    fn try_spawn_drip_particles(level: &LevelWithEffects<'_>, pos: &BlockPos, state: BlockState) {
        if state.get_fluid_state().is_empty() && level.get_random().next_float() >= 0.3 {
            let collision_shape = state.get_collision_shape(&**level, pos);
            let top_side_height = collision_shape.max(Axis::Y);
            if top_side_height >= 1.0 && !state.is_tag(BlockTags::IMPERMEABLE) {
                let bottom_side_height = collision_shape.min(Axis::Y);
                if bottom_side_height > 0.0 {
                    BeehiveBlock::spawn_particle(
                        level,
                        pos,
                        &collision_shape,
                        f64::from(pos.get_y()) + bottom_side_height - 0.05,
                    );
                } else {
                    let below = pos.below();
                    let below_state = level.get_block_state(&below);
                    let below_shape = below_state.get_collision_shape(&**level, &below);
                    let below_top_side_height = below_shape.max(Axis::Y);
                    if (below_top_side_height < 1.0
                        || !below_state.is_collision_shape_full_block(&**level, &below))
                        && below_state.get_fluid_state().is_empty()
                    {
                        BeehiveBlock::spawn_particle(
                            level,
                            pos,
                            &collision_shape,
                            f64::from(pos.get_y()) - 0.05,
                        );
                    }
                }
            }
        }
    }

    fn spawn_particle(
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        drip_shape: &VoxelShape,
        height: f64,
    ) {
        BeehiveBlock::spawn_fluid_particle(
            level,
            f64::from(pos.get_x()) + drip_shape.min(Axis::X),
            f64::from(pos.get_x()) + drip_shape.max(Axis::X),
            f64::from(pos.get_z()) + drip_shape.min(Axis::Z),
            f64::from(pos.get_z()) + drip_shape.max(Axis::Z),
            height,
        );
    }

    fn spawn_fluid_particle(
        level: &LevelWithEffects<'_>,
        x1: f64,
        x2: f64,
        z1: f64,
        z2: f64,
        y: f64,
    ) {
        let x = Mth::lerp_f64(level.get_random().next_double(), x1, x2);
        let z = Mth::lerp_f64(level.get_random().next_double(), z1, z2);
        level.add_particle(ParticleTypes::DRIPPING_HONEY, x, y, z, 0.0, 0.0, 0.0);
    }
}
