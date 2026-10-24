//! net/minecraft/world/level/block/SporeBlossomBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, its shape, and its spores (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::block_pos::MutableBlockPos;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct SporeBlossomBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(SporeBlossomBlock::animate_tick),
    get_shape: Some(SporeBlossomBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SporeBlossomBlock",
        &block::CLASS,
    )
};

impl SporeBlossomBlock {
    pub fn new(properties: Properties) -> BlockData {
        SporeBlossomBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(12.0, 13.0, 16.0));

impl SporeBlossomBlock {
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

impl SporeBlossomBlock {
    /// `animateTick`: a spore falling from it, and spores in the air below and around it.
    fn animate_tick(
        _block: &BlockData,
        _state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        let plant_x = pos.get_x();
        let plant_y = pos.get_y();
        let plant_z = pos.get_z();
        let x_falling = f64::from(plant_x) + random.next_double();
        let y_falling = f64::from(plant_y) + 0.7;
        let z_falling = f64::from(plant_z) + random.next_double();
        level.add_particle(
            ParticleTypes::FALLING_SPORE_BLOSSOM,
            x_falling,
            y_falling,
            z_falling,
            0.0,
            0.0,
            0.0,
        );
        let mut ambient_pos = MutableBlockPos::new();

        for _ in 0..14 {
            let ax = plant_x + Mth::next_int(random, -10, 10);
            let ay = plant_y - random.next_int_bounded(10);
            let az = plant_z + Mth::next_int(random, -10, 10);
            ambient_pos.set_xyz(ax, ay, az);
            let particle_pos = ambient_pos.immutable();
            let particle_pos_state = level.get_block_state(&particle_pos);
            if !particle_pos_state.is_collision_shape_full_block(&**level, &particle_pos) {
                let x = f64::from(particle_pos.get_x()) + random.next_double();
                let y = f64::from(particle_pos.get_y()) + random.next_double();
                let z = f64::from(particle_pos.get_z()) + random.next_double();
                level.add_particle(ParticleTypes::SPORE_BLOSSOM_AIR, x, y, z, 0.0, 0.0, 0.0);
            }
        }
    }
}
