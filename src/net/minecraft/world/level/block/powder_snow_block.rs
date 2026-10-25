//! net/minecraft/world/level/block/PowderSnowBlock.java
//!
//! Its blocks' states are its parent's. Of its behaviour, its shapes (to collide with only for
//! falling blocks, what walks on it from above, and anything falling fast; nothing to see; the
//! collision shape, or the whole block when that's empty, to be inside it),
//! `canEntityWalkOnPowderSnow`, `skipRendering` (against itself) and `entityInside`: what isn't
//! living, or a living entity whose feet are in it, is stuck (a tenth slower sideways, half again
//! as fast up and down), and everything inside freezes and has its fire put out. The snowflakes it
//! sends and the block a burning entity melts are the server's, and so is the fall's damage
//! (`fallOn`, whose sound isn't ported). Not ported yet: picking it up with a bucket.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::tags::entity_type_tags::EntityTypeTags;
use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::entity::inside_block_effect_applier::InsideBlockEffectApplier;
use crate::net::minecraft::world::entity::inside_block_effect_type::InsideBlockEffectType;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::{self, Properties};
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::level::Level;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::net::minecraft::world::phys::vec3::Vec3;
use std::sync::LazyLock;

pub struct PowderSnowBlock;

pub static CLASS: BlockClass = BlockClass {
    get_collision_shape: Some(PowderSnowBlock::get_collision_shape),
    get_visual_shape: Some(PowderSnowBlock::get_visual_shape),
    skip_rendering: Some(PowderSnowBlock::skip_rendering),
    get_entity_inside_collision_shape: Some(PowderSnowBlock::get_entity_inside_collision_shape),
    entity_inside: Some(PowderSnowBlock::entity_inside),
    ..BlockClass::new(
        "net.minecraft.world.level.block.PowderSnowBlock",
        &block::CLASS,
    )
};

impl PowderSnowBlock {
    pub fn new(properties: Properties) -> BlockData {
        PowderSnowBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

static FALLING_COLLISION_SHAPE: LazyLock<VoxelShape> =
    LazyLock::new(|| Shapes::r#box(0.0, 0.0, 0.0, 1.0, f64::from(0.9_f32), 1.0));

impl PowderSnowBlock {
    fn skip_rendering(
        _block: &BlockData,
        state: BlockState,
        neighbor_state: BlockState,
        _direction: Direction,
    ) -> bool {
        neighbor_state.is(state.get_block())
    }

    /// `getCollisionShape`: for an entity (not placing a block), a falling block's shape when it
    /// falls more than two and a half blocks, and its own when it's a falling block or walks on
    /// powder snow from above without descending; nothing for anything else.
    fn get_collision_shape(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        context: &dyn CollisionContext,
    ) -> VoxelShape {
        if !context.is_placement()
            && let Some(entity) = context
                .as_entity_collision_context()
                .and_then(|context| context.get_entity())
        {
            if entity.fall_distance > 2.5 {
                return FALLING_COLLISION_SHAPE.clone();
            }
            if entity.is_falling_block
                || entity.can_walk_on_powder_snow
                    && context.is_above(&Shapes::block(), pos, false)
                    && !context.is_descending()
            {
                return block_behaviour::get_collision_shape(block, state, level, pos, context);
            }
        }
        Shapes::empty()
    }

    fn get_visual_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::empty()
    }

    fn entity_inside(
        _block: &BlockData,
        state: BlockState,
        level: &dyn Level,
        _pos: &BlockPos,
        entity: &mut dyn EntityClass,
        effect_applier: &mut dyn InsideBlockEffectApplier,
        _is_precise: bool,
    ) {
        if !entity.is_living_entity()
            || entity
                .entity_mut()
                .get_in_block_state(level)
                .is(state.get_block())
        {
            entity.make_stuck_in_block(
                state,
                Vec3::new(f64::from(0.9_f32), 1.5, f64::from(0.9_f32)),
            );
            // The snowflakes of an entity moving in it are the server's.
        }

        // A burning entity melts the block, on the server.
        effect_applier.run_before(InsideBlockEffectType::EXTINGUISH, Box::new(|_, _| {}));
        effect_applier.apply(InsideBlockEffectType::FREEZE);
        effect_applier.apply(InsideBlockEffectType::EXTINGUISH);
    }

    /// `getEntityInsideCollisionShape`: its collision shape for the entity, or the whole block
    /// when that's empty.
    fn get_entity_inside_collision_shape(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        entity: &dyn EntityClass,
    ) -> VoxelShape {
        let collision_shape = PowderSnowBlock::get_collision_shape(
            block,
            state,
            level,
            pos,
            &entity.collision_context(),
        );
        if collision_shape.is_empty() {
            Shapes::block()
        } else {
            collision_shape
        }
    }

    /// `canEntityWalkOnPowderSnow(Entity)`: a mob of the tag, or one in leather boots, which none
    /// is: equipment isn't ported.
    pub fn can_entity_walk_on_powder_snow(entity: &(impl EntityClass + ?Sized)) -> bool {
        entity
            .entity()
            .get_type()
            .is_tag(EntityTypeTags::POWDER_SNOW_WALKABLE_MOBS)
    }
}
