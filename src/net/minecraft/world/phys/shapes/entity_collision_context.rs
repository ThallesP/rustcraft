//! net/minecraft/world/phys/shapes/EntityCollisionContext.java, with its `Empty`.
//!
//! The entity is what it answers from, so the context holds those answers rather than the
//! entity: whether it's descending, where its bottom is, the item in its main hand (by its id),
//! and what blocks' shapes ask of the entity itself (`getEntity()`: its fall distance, whether
//! it's a falling block, walks on powder snow or stands on a fluid), none for a context without
//! one.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

/// A context for an entity, or for no entity (`Empty`).
#[derive(Clone, Debug)]
pub struct EntityCollisionContext {
    descending: bool,
    entity_bottom: f64,
    placement: bool,
    held_item: Option<ResourceKey>,
    always_collide_with_fluid: bool,
    entity: Option<CollidingEntity>,
    empty: bool,
}

/// What blocks' shapes ask of the entity a context is for.
#[derive(Clone, Debug)]
pub struct CollidingEntity {
    pub fall_distance: f64,
    /// `instanceof FallingBlockEntity`.
    pub is_falling_block: bool,
    /// `PowderSnowBlock.canEntityWalkOnPowderSnow`: a mob of the tag, or wearing leather boots.
    pub can_walk_on_powder_snow: bool,
    /// What a `LivingEntity` answers, none for another entity.
    pub living: Option<LivingCollision>,
}

/// A `LivingEntity`'s `getLiquidCollisionShape()` and `canStandOnFluid(FluidState)`.
#[derive(Clone, Debug)]
pub struct LivingCollision {
    pub liquid_collision_shape: VoxelShape,
    pub can_stand_on_fluid: fn(&FluidState) -> bool,
}

impl EntityCollisionContext {
    /// The protected constructor taking the entity's answers, with an empty hand and no entity.
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
            held_item: None,
            always_collide_with_fluid,
            entity: None,
            empty: false,
        }
    }

    /// The item in the entity's main hand, by its id.
    pub fn with_held_item(mut self, held_item: Option<ResourceKey>) -> EntityCollisionContext {
        self.held_item = held_item;
        self
    }

    /// The entity it's for, as blocks' shapes see it.
    pub fn with_entity(mut self, entity: CollidingEntity) -> EntityCollisionContext {
        self.entity = Some(entity);
        self
    }

    /// `new Empty(alwaysCollideWithFluid)`: no entity, its bottom at `-Double.MAX_VALUE`.
    pub fn empty(always_collide_with_fluid: bool) -> EntityCollisionContext {
        EntityCollisionContext {
            empty: true,
            ..EntityCollisionContext::new(false, false, -f64::MAX, always_collide_with_fluid)
        }
    }

    /// `getEntity()`.
    pub fn get_entity(&self) -> Option<&CollidingEntity> {
        self.entity.as_ref()
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

    fn is_holding_item(&self, item: &ResourceKey) -> bool {
        self.held_item.as_ref() == Some(item)
    }

    fn always_collide_with_fluid(&self) -> bool {
        self.always_collide_with_fluid
    }

    /// A living entity's own answer, unless the fluid above is the same fluid; false for
    /// anything else.
    fn can_stand_on_fluid(&self, fluid_state_above: &FluidState, fluid: &FluidState) -> bool {
        match self
            .entity
            .as_ref()
            .and_then(|entity| entity.living.as_ref())
        {
            Some(living) => {
                (living.can_stand_on_fluid)(fluid)
                    && !fluid_state_above.get_type().is_same(fluid.get_type())
            }
            None => false,
        }
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

    fn as_entity_collision_context(&self) -> Option<&EntityCollisionContext> {
        Some(self)
    }
}
