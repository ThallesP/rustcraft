//! net/minecraft/world/level/block/PowderSnowBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, its shapes (to collide with only for falling blocks, what walks on it from
//! above, and anything falling fast; nothing to see) and `skipRendering` (against itself) are
//! ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::{self, Properties};
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct PowderSnowBlock;

pub static CLASS: BlockClass = BlockClass {
    get_collision_shape: Some(PowderSnowBlock::get_collision_shape),
    get_visual_shape: Some(PowderSnowBlock::get_visual_shape),
    skip_rendering: Some(PowderSnowBlock::skip_rendering),
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
}
