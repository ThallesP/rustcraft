//! net/minecraft/world/level/ClipContext.java
//!
//! What a ray cast through the level (`BlockGetter.clip`) hits: the blocks' shapes it asks
//! (`Block`) and the fluids it picks (`Fluid`), from where to where, for whom. Only the shape
//! getters an entity's movement uses are ported: `FALLDAMAGE_RESETTING` (`Entity.move`'s check
//! that a fast fall passes something that ends it) and the collision and outline shapes; the
//! visual shape (`getVisualShape`) isn't, and panics. The entity is what its collision context and
//! `is(EntityTypes.PLAYER)` answer.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::tags::fluid_tags::FluidTags;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::phys::shapes::entity_collision_context::EntityCollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// A ray from `from` to `to`.
pub struct ClipContext {
    from: Vec3,
    to: Vec3,
    block: Block,
    fluid: Fluid,
    collision_context: EntityCollisionContext,
    /// `entity.is(EntityTypes.PLAYER)`, for `FALLDAMAGE_RESETTING`; false without an entity.
    entity_is_player: bool,
}

impl ClipContext {
    /// `new ClipContext(from, to, block, fluid, entity)`: the entity's collision context.
    pub fn new(
        from: Vec3,
        to: Vec3,
        block: Block,
        fluid: Fluid,
        collision_context: EntityCollisionContext,
        entity_is_player: bool,
    ) -> ClipContext {
        ClipContext {
            from,
            to,
            block,
            fluid,
            collision_context,
            entity_is_player,
        }
    }

    pub fn get_to(&self) -> Vec3 {
        self.to
    }

    pub fn get_from(&self) -> Vec3 {
        self.from
    }

    pub fn get_block_shape(
        &self,
        block_state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> VoxelShape {
        self.block.get(
            block_state,
            level,
            pos,
            &self.collision_context,
            self.entity_is_player,
        )
    }

    pub fn get_fluid_shape(
        &self,
        fluid_state: FluidState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> VoxelShape {
        if self.fluid.can_pick(fluid_state) {
            fluid_state.get_shape(level, pos)
        } else {
            Shapes::empty()
        }
    }
}

/// `ClipContext.Block`: which shape of a block the ray hits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Block {
    COLLIDER,
    OUTLINE,
    VISUAL,
    FALLDAMAGE_RESETTING,
}

impl Block {
    fn get(
        &self,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        collision_context: &EntityCollisionContext,
        entity_is_player: bool,
    ) -> VoxelShape {
        match self {
            Block::COLLIDER => state.get_collision_shape_in(level, pos, collision_context),
            Block::OUTLINE => state.get_shape_in(level, pos, collision_context),
            Block::VISUAL => panic!("getVisualShape isn't ported"),
            Block::FALLDAMAGE_RESETTING => {
                if state.is_tag(BlockTags::FALL_DAMAGE_RESETTING) {
                    return Shapes::block();
                }

                // A server's nether portal with no delay too, which the client's level isn't.
                if entity_is_player
                    && (state.is(Blocks::END_GATEWAY) || state.is(Blocks::END_PORTAL))
                {
                    return Shapes::block();
                }

                Shapes::empty()
            }
        }
    }
}

/// `ClipContext.Fluid`: which fluids the ray hits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fluid {
    NONE,
    SOURCE_ONLY,
    ANY,
    WATER,
}

impl Fluid {
    pub fn can_pick(&self, fluid_state: FluidState) -> bool {
        match self {
            Fluid::NONE => false,
            Fluid::SOURCE_ONLY => fluid_state.is_source(),
            Fluid::ANY => !fluid_state.is_empty(),
            Fluid::WATER => fluid_state.is_tag(FluidTags::WATER),
        }
    }
}
