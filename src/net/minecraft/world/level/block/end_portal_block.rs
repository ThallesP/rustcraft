//! net/minecraft/world/level/block/EndPortalBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, its shape (a slab from 6 to 12 sixteenths up), which is also what an entity
//! has to touch to be in it, its render shape (invisible: its block entity draws it), and its smoke
//! (`animateTick`) are ported. What it does to an entity inside (`entityInside`, a portal) isn't.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_behaviour::RenderShape;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct EndPortalBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(EndPortalBlock::animate_tick),
    get_shape: Some(EndPortalBlock::get_shape),
    get_entity_inside_collision_shape: Some(EndPortalBlock::get_entity_inside_collision_shape),
    get_render_shape: Some(EndPortalBlock::get_render_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.EndPortalBlock",
        &base_entity_block::CLASS,
    )
};

impl EndPortalBlock {
    pub fn new(properties: Properties) -> BlockData {
        EndPortalBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        BaseEntityBlock::new_with(properties, class, definition)
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 6.0, 12.0));

impl EndPortalBlock {
    fn get_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE.clone()
    }

    fn get_render_shape(_block: &BlockData, _state: BlockState) -> RenderShape {
        RenderShape::INVISIBLE
    }

    /// `getEntityInsideCollisionShape`: its shape.
    fn get_entity_inside_collision_shape(
        _block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        _entity: &dyn EntityClass,
    ) -> VoxelShape {
        state.get_shape(level, pos)
    }
}

impl EndPortalBlock {
    /// `animateTick`: smoke over it.
    fn animate_tick(
        _block: &BlockData,
        _state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        let x = f64::from(pos.get_x()) + random.next_double();
        let y = f64::from(pos.get_y()) + 0.8;
        let z = f64::from(pos.get_z()) + random.next_double();
        level.add_particle(ParticleTypes::SMOKE, x, y, z, 0.0, 0.0, 0.0);
    }
}
