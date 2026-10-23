//! net/minecraft/world/phys/shapes/CollisionContext.java
//!
//! Only what collisions ask a context so far: `empty()`, `of(Entity)` (which the entity makes,
//! `EntityClass::collision_context`), and `getCollisionShape`. `isHoldingItem`,
//! `canStandOnFluid` and the placement and position contexts come with items, fluids and
//! `MinecartCollisionContext`.

use crate::net::minecraft::core::block_pos::BlockPos;
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
}

/// `CollisionContext.empty()`.
pub fn empty() -> EntityCollisionContext {
    EntityCollisionContext::empty(false)
}

/// `CollisionContext.emptyWithFluidCollisions()`.
pub fn empty_with_fluid_collisions() -> EntityCollisionContext {
    EntityCollisionContext::empty(true)
}
