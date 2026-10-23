//! net/minecraft/world/phys/shapes/CollisionContext.java
//!
//! Only what collisions ask a context so far: `empty()`, `of(Entity)` (which the entity makes,
//! `EntityClass::collision_context`), `getCollisionShape`, and `isHoldingItem` for a block's item,
//! which is never true: no ported entity holds anything (inventories aren't ported).
//! `canStandOnFluid` and the placement and position contexts come with striders, items and
//! `MinecartCollisionContext`.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::entity_collision_context::EntityCollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

/// What a block's collision shape may depend on: who's colliding.
pub trait CollisionContext {
    fn is_descending(&self) -> bool;

    fn is_above(&self, shape: &VoxelShape, pos: &BlockPos, default_value: bool) -> bool;

    fn always_collide_with_fluid(&self) -> bool;

    fn get_collision_shape(
        &self,
        state: BlockState,
        collision_getter: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> VoxelShape;

    fn is_placement(&self) -> bool {
        false
    }

    /// `isHoldingItem(block.asItem())`: whether the entity holds the block's item in its main
    /// hand. No ported entity holds anything.
    fn is_holding_block_item(&self, _block: Block) -> bool {
        false
    }
}

/// `CollisionContext.empty()`.
pub fn empty() -> EntityCollisionContext {
    EntityCollisionContext::empty(false)
}

/// `CollisionContext.emptyWithFluidCollisions()`.
pub fn empty_with_fluid_collisions() -> EntityCollisionContext {
    EntityCollisionContext::empty(true)
}
