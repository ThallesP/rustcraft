//! net/minecraft/world/phys/shapes/EntityCollisionContext.java, with its `Empty`.
//!
//! The entity is what it answers from, so the context holds those answers rather than the
//! entity: whether it's descending and where its bottom is. Not ported yet: the held item
//! (`isHoldingItem`) and `canStandOnFluid`, which come with items and fluids.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

/// A context for an entity, or for no entity (`Empty`).
#[derive(Clone, Debug)]
pub struct EntityCollisionContext {
    descending: bool,
    entity_bottom: f64,
    placement: bool,
    always_collide_with_fluid: bool,
    empty: bool,
}

impl EntityCollisionContext {
    /// The protected constructor taking the entity's answers.
    pub fn new(
        descending: bool,
        placement: bool,
        entity_bottom: f64,
        always_collide_with_fluid: bool,
    ) -> EntityCollisionContext {
        EntityCollisionContext {
            descending,
            entity_bottom,
            placement,
            always_collide_with_fluid,
            empty: false,
        }
    }

    /// `new Empty(alwaysCollideWithFluid)`: no entity, its bottom at `-Double.MAX_VALUE`.
    pub fn empty(always_collide_with_fluid: bool) -> EntityCollisionContext {
        EntityCollisionContext {
            descending: false,
            entity_bottom: -f64::MAX,
            placement: false,
            always_collide_with_fluid,
            empty: true,
        }
    }
}

impl CollisionContext for EntityCollisionContext {
    fn is_descending(&self) -> bool {
        self.descending
    }

    /// Whether the entity's bottom is above the shape's top at `pos`, less 1.0E-5F; an empty
    /// context answers the default.
    fn is_above(&self, shape: &VoxelShape, pos: &BlockPos, default_value: bool) -> bool {
        if self.empty {
            default_value
        } else {
            self.entity_bottom > f64::from(pos.get_y()) + shape.max(Axis::Y) - f64::from(1.0E-5_f32)
        }
    }

    fn always_collide_with_fluid(&self) -> bool {
        self.always_collide_with_fluid
    }

    fn get_collision_shape(
        &self,
        state: BlockState,
        collision_getter: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> VoxelShape {
        state.get_collision_shape_in(collision_getter, pos, self)
    }

    fn is_placement(&self) -> bool {
        self.placement
    }
}
