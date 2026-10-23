//! net/minecraft/world/phys/shapes/CollisionContext.java
//!
//! What collisions and blocks' shapes ask a context: `empty()`, `emptyWithFluidCollisions()`,
//! `of(Entity)` (which the entity makes, `EntityClass::collision_context`), `placementContext` and
//! `withPosition` without a player, and the questions. `MinecartCollisionContext` and
//! `positionContext` come with minecarts and with what places blocks.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::phys::shapes::entity_collision_context::EntityCollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

/// What a block's collision shape may depend on: who's colliding.
pub trait CollisionContext {
    fn is_descending(&self) -> bool;

    fn is_above(&self, shape: &VoxelShape, pos: &BlockPos, default_value: bool) -> bool;

    /// `isHoldingItem(Item)`: whether what's colliding holds the item, by its id.
    fn is_holding_item(&self, item: &ResourceKey) -> bool;

    fn always_collide_with_fluid(&self) -> bool;

    fn can_stand_on_fluid(&self, fluid_state_above: &FluidState, fluid: &FluidState) -> bool;

    fn get_collision_shape(
        &self,
        state: BlockState,
        collision_getter: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> VoxelShape;

    fn is_placement(&self) -> bool {
        false
    }

    /// `instanceof EntityCollisionContext`, which blocks asking about the entity check.
    fn as_entity_collision_context(&self) -> Option<&EntityCollisionContext> {
        None
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

/// `CollisionContext.placementContext(null)`: a block placed by no player.
pub fn placement_context_without_player() -> EntityCollisionContext {
    EntityCollisionContext::new(false, true, -f64::MAX, false)
}
