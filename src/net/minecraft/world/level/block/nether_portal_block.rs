//! net/minecraft/world/level/block/NetherPortalBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, its shape (a pane along its axis), and its particles and sound (`animateTick`)
//! are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::sounds::sound_events::SoundEvents;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct NetherPortalBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(NetherPortalBlock::animate_tick),
    get_shape: Some(NetherPortalBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.NetherPortalBlock",
        &block::CLASS,
    )
};

impl NetherPortalBlock {
    pub const AXIS: &'static EnumProperty<Axis> = BlockStateProperties::HORIZONTAL_AXIS;

    pub fn new(properties: Properties) -> BlockData {
        NetherPortalBlock::new_with(
            properties,
            &CLASS,
            NetherPortalBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(NetherPortalBlock::AXIS, Axis::X),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[NetherPortalBlock::AXIS]);
    }
}

static SHAPES: LazyLock<Vec<(Axis, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal_axis(&Block::column_xz(4.0, 16.0, 0.0, 16.0)));

impl NetherPortalBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(
            &SHAPES,
            state.get_value(BlockStateProperties::HORIZONTAL_AXIS),
        )
        .clone()
    }
}

impl NetherPortalBlock {
    /// `animateTick`: its sound now and then, and four portal particles, along the portal's plane.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if random.next_int_bounded(100) == 0 {
            let pitch = random.next_float() * 0.4 + 0.8;
            level.play_local_sound(
                f64::from(pos.get_x()) + 0.5,
                f64::from(pos.get_y()) + 0.5,
                f64::from(pos.get_z()) + 0.5,
                SoundEvents::PORTAL_AMBIENT,
                SoundSource::BLOCKS,
                0.5,
                pitch,
                false,
            );
        }

        let this = state.get_block();
        for _ in 0..4 {
            let mut x = f64::from(pos.get_x()) + random.next_double();
            let y = f64::from(pos.get_y()) + random.next_double();
            let mut z = f64::from(pos.get_z()) + random.next_double();
            let mut xa = (f64::from(random.next_float()) - 0.5) * 0.5;
            let ya = (f64::from(random.next_float()) - 0.5) * 0.5;
            let mut za = (f64::from(random.next_float()) - 0.5) * 0.5;
            let flip = random.next_int_bounded(2) * 2 - 1;
            if !level.get_block_state(&pos.west()).is(this)
                && !level.get_block_state(&pos.east()).is(this)
            {
                x = f64::from(pos.get_x()) + 0.5 + 0.25 * f64::from(flip);
                xa = f64::from(random.next_float() * 2.0 * flip as f32);
            } else {
                z = f64::from(pos.get_z()) + 0.5 + 0.25 * f64::from(flip);
                za = f64::from(random.next_float() * 2.0 * flip as f32);
            }

            level.add_particle(ParticleTypes::PORTAL, x, y, z, xa, ya, za);
        }
    }
}
