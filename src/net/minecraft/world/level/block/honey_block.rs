//! net/minecraft/world/level/block/HoneyBlock.java
//!
//! Its blocks' states are its parent's. Of its behaviour, its collision shape (a sixteenth in and
//! a sixteenth short) and `entityInside`: an entity falling down one of its sides, off the ground
//! and below its top, slides, down at a fixed speed and slowed sideways when it fell fast, its fall
//! over. Its slow walking and low jumps are its `Properties`' speed and jump factors. What only
//! the server does (the slide's advancement and the fall's damage in `fallOn`) and what only
//! makes sounds and particles (the slide's, from the level's random source, and the fall's) isn't
//! ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::entity::inside_block_effect_applier::InsideBlockEffectApplier;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::half_transparent_block::{
    self, HalfTransparentBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::level::Level;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::net::minecraft::world::phys::vec3::Vec3;
use std::sync::LazyLock;

pub struct HoneyBlock;

pub static CLASS: BlockClass = BlockClass {
    get_collision_shape: Some(HoneyBlock::get_collision_shape),
    entity_inside: Some(HoneyBlock::entity_inside),
    ..BlockClass::new(
        "net.minecraft.world.level.block.HoneyBlock",
        &half_transparent_block::CLASS,
    )
};

impl HoneyBlock {
    pub fn new(properties: Properties) -> BlockData {
        HoneyBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        HalfTransparentBlock::new_with(properties, class, definition)
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(14.0, 0.0, 15.0));

impl HoneyBlock {
    fn get_collision_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE.clone()
    }

    /// `entityInside`, then `BlockBehaviour`'s, which does nothing. The slide's advancement
    /// (`maybeDoSlideAchievement`) is the server's, and its effects (`maybeDoSlideEffects`) only
    /// a sound and the server's particles, from the level's random source.
    fn entity_inside(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn Level,
        pos: &BlockPos,
        entity: &mut dyn EntityClass,
        _effect_applier: &mut dyn InsideBlockEffectApplier,
        _is_precise: bool,
    ) {
        if HoneyBlock::is_sliding_down(pos, entity) {
            HoneyBlock::do_slide_movement(entity);
        }
    }

    /// `getOldDeltaY(double)`: the vertical movement before the last tick's gravity and drag.
    fn get_old_delta_y(delta_y: f64) -> f64 {
        delta_y / f64::from(0.98_f32) + 0.08
    }

    /// `getNewDeltaY(double)`: the vertical movement after a tick's gravity and drag.
    fn get_new_delta_y(delta_y: f64) -> f64 {
        (delta_y - 0.08) * f64::from(0.98_f32)
    }

    /// `isSlidingDown`: off the ground, no higher than the block's top, falling, and against one
    /// of its sides.
    fn is_sliding_down(pos: &BlockPos, entity: &dyn EntityClass) -> bool {
        let entity = entity.entity();
        if entity.on_ground() {
            return false;
        }

        if entity.get_y() > f64::from(pos.get_y()) + 0.9375 - 1.0E-7 {
            return false;
        }

        if HoneyBlock::get_old_delta_y(entity.get_delta_movement().y) >= -0.08 {
            return false;
        }

        let dx = (f64::from(pos.get_x()) + 0.5 - entity.get_x()).abs();
        let dz = (f64::from(pos.get_z()) + 0.5 - entity.get_z()).abs();
        let overlap_distance = 0.4375 + f64::from(entity.get_bb_width() / 2.0);
        dx + 1.0E-7 > overlap_distance || dz + 1.0E-7 > overlap_distance
    }

    /// `doSlideMovement`: down at the slide's speed, slowed sideways by as much when it fell
    /// faster, and its fall over.
    fn do_slide_movement(entity: &mut dyn EntityClass) {
        let delta_movement = entity.entity().get_delta_movement();
        if HoneyBlock::get_old_delta_y(entity.entity().get_delta_movement().y) < -0.13 {
            let horizontal_reduction_factor =
                -0.05 / HoneyBlock::get_old_delta_y(entity.entity().get_delta_movement().y);
            entity.entity_mut().set_delta_movement(Vec3::new(
                delta_movement.x * horizontal_reduction_factor,
                HoneyBlock::get_new_delta_y(-0.05),
                delta_movement.z * horizontal_reduction_factor,
            ));
        } else {
            entity.entity_mut().set_delta_movement(Vec3::new(
                delta_movement.x,
                HoneyBlock::get_new_delta_y(-0.05),
                delta_movement.z,
            ));
        }

        entity.entity_mut().reset_fall_distance();
    }
}
