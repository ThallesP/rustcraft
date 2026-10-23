//! net/minecraft/world/level/CollisionGetter.java
//!
//! What collisions ask a level: its blocks' shapes (`BlockCollisions`), its entities' (which the
//! port has none of yet), and its border. A source entity is a `&dyn EntityClass`, whose collision
//! context and position the queries read. Not ported yet: `isUnobstructed`, the pre-move and
//! liquid collisions, `clipIncludingBorder` and `findFreePosition`, which come with what calls
//! them.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::level::block_collisions::BlockCollisions;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::border::world_border::WorldBorder;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
use crate::net::minecraft::world::phys::shapes::collision_context;
use crate::net::minecraft::world::phys::shapes::entity_collision_context::EntityCollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

/// A level, as far as collisions go.
pub trait CollisionGetter: BlockGetter {
    fn get_world_border(&self) -> &WorldBorder;

    /// The chunk at the chunk coordinates, if it's loaded.
    fn get_chunk_for_collisions(&self, chunk_x: i32, chunk_z: i32) -> Option<&dyn BlockGetter>;

    /// Other entities' shapes in the area that `source` collides with.
    fn get_entity_collisions(
        &self,
        source: Option<&dyn EntityClass>,
        test_area: &AABB,
    ) -> Vec<VoxelShape>;
}

impl dyn CollisionGetter + '_ {
    /// `noCollision(Entity)`: whether nothing is in the source's box.
    pub fn no_collision_entity(&self, source: &dyn EntityClass) -> bool {
        self.no_collision(Some(source), &source.entity().get_bounding_box())
    }

    /// `noCollision(Entity, AABB)`: no block, entity or border in the box.
    pub fn no_collision(&self, entity: Option<&dyn EntityClass>, aabb: &AABB) -> bool {
        self.no_block_collision(entity, aabb)
            && self.no_entity_collision(entity, aabb)
            && self.no_border_collision(entity, aabb)
    }

    pub fn no_block_collision(&self, entity: Option<&dyn EntityClass>, aabb: &AABB) -> bool {
        self.get_block_collisions(entity, aabb)
            .all(|block_collision| block_collision.is_empty())
    }

    pub fn no_entity_collision(&self, entity: Option<&dyn EntityClass>, aabb: &AABB) -> bool {
        self.get_entity_collisions(entity, aabb).is_empty()
    }

    pub fn no_border_collision(&self, entity: Option<&dyn EntityClass>, aabb: &AABB) -> bool {
        let Some(entity) = entity else {
            return true;
        };
        match self.border_collision(entity, aabb) {
            None => true,
            Some(border_shape) => !Shapes::join_is_not_empty(
                &border_shape,
                &Shapes::create_aabb(aabb),
                BooleanOp::AND,
            ),
        }
    }

    /// The blocks' collision shapes the box touches, made as they're asked for.
    pub fn get_block_collisions(
        &self,
        source: Option<&dyn EntityClass>,
        r#box: &AABB,
    ) -> BlockCollisions<'_, VoxelShape> {
        let context = match source {
            None => collision_context::empty(),
            Some(source) => source.collision_context(),
        };
        self.get_block_collisions_from_context(context, r#box)
    }

    pub fn get_block_collisions_from_context(
        &self,
        source: EntityCollisionContext,
        r#box: &AABB,
    ) -> BlockCollisions<'_, VoxelShape> {
        BlockCollisions::new(self, source, *r#box, false, |_, shape| shape)
    }

    fn border_collision(&self, source: &dyn EntityClass, r#box: &AABB) -> Option<VoxelShape> {
        let world_border = self.get_world_border();
        let entity = source.entity();
        world_border
            .is_inside_close_to_border(entity.get_x(), entity.get_z(), r#box)
            .then(|| world_border.get_collision_shape())
    }

    /// Whether a suffocating block's shape is in the box.
    pub fn collides_with_suffocating_block(
        &self,
        source: Option<&dyn EntityClass>,
        r#box: &AABB,
    ) -> bool {
        let context = match source {
            None => collision_context::empty(),
            Some(source) => source.collision_context(),
        };
        BlockCollisions::new(self, context, *r#box, true, |_, shape| shape)
            .any(|shape| !shape.is_empty())
    }

    /// The block under the box the source stands on: the one whose center is nearest the source,
    /// and on a tie the greatest by `compareTo`.
    pub fn find_supporting_block(
        &self,
        source: &dyn EntityClass,
        r#box: &AABB,
    ) -> Option<BlockPos> {
        let mut main_support: Option<BlockPos> = None;
        let mut main_support_distance = f64::MAX;
        let position = source.entity().position();
        for pos in
            BlockCollisions::new(self, source.collision_context(), *r#box, false, |pos, _| {
                pos
            })
        {
            let distance = pos.dist_to_center_sqr(&position);
            if distance < main_support_distance
                || distance == main_support_distance
                    && main_support.is_none_or(|main_support| main_support.compare_to(&pos) < 0)
            {
                main_support = Some(pos);
                main_support_distance = distance;
            }
        }
        main_support
    }
}
