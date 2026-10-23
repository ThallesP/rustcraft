//! net/minecraft/world/level/BlockCollisions.java
//!
//! An iterator over the collision shapes of the blocks a box touches, each moved to its block,
//! which only reads blocks as it's asked for the next one, like Java's `AbstractIterator`. The
//! blocks on the box's faces only count when their shape reaches past their block, and those on
//! its edges only when they're moving pistons, and its corners never.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::cursor3d::Cursor3D;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::collision_getter::CollisionGetter;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::entity_collision_context::EntityCollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

/// The blocks' shapes a box collides with, made into `T` by the result provider.
pub struct BlockCollisions<'a, T> {
    r#box: AABB,
    context: EntityCollisionContext,
    cursor: Cursor3D,
    entity_shape: VoxelShape,
    collision_getter: &'a dyn CollisionGetter,
    only_suffocating_blocks: bool,
    cached_block_getter: Option<(i64, Option<&'a dyn BlockGetter>)>,
    result_provider: fn(BlockPos, VoxelShape) -> T,
}

impl<'a, T> BlockCollisions<'a, T> {
    pub fn new(
        collision_getter: &'a dyn CollisionGetter,
        context: EntityCollisionContext,
        r#box: AABB,
        only_suffocating_blocks: bool,
        result_provider: fn(BlockPos, VoxelShape) -> T,
    ) -> BlockCollisions<'a, T> {
        let x0 = Mth::floor_f64(r#box.min_x - 1.0E-7) - 1;
        let x1 = Mth::floor_f64(r#box.max_x + 1.0E-7) + 1;
        let y0 = Mth::floor_f64(r#box.min_y - 1.0E-7) - 1;
        let y1 = Mth::floor_f64(r#box.max_y + 1.0E-7) + 1;
        let z0 = Mth::floor_f64(r#box.min_z - 1.0E-7) - 1;
        let z1 = Mth::floor_f64(r#box.max_z + 1.0E-7) + 1;
        BlockCollisions {
            r#box,
            context,
            cursor: Cursor3D::new(x0, y0, z0, x1, y1, z1),
            entity_shape: Shapes::create_aabb(&r#box),
            collision_getter,
            only_suffocating_blocks,
            cached_block_getter: None,
            result_provider,
        }
    }

    /// The chunk the block is in, kept for the next block in the same chunk.
    fn get_chunk(&mut self, x: i32, z: i32) -> Option<&'a dyn BlockGetter> {
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;
        // ChunkPos.pack.
        let chunk_pos =
            (i64::from(chunk_x) & 0xFFFF_FFFF) | ((i64::from(chunk_z) & 0xFFFF_FFFF) << 32);
        if let Some((cached_pos, cached)) = self.cached_block_getter
            && cached.is_some()
            && cached_pos == chunk_pos
        {
            return cached;
        }

        let result = self
            .collision_getter
            .get_chunk_for_collisions(chunk_x, chunk_z);
        self.cached_block_getter = Some((chunk_pos, result));
        result
    }
}

impl<T> Iterator for BlockCollisions<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        while self.cursor.advance() {
            let x = self.cursor.next_x();
            let y = self.cursor.next_y();
            let z = self.cursor.next_z();
            let cursor_face_type = self.cursor.get_next_type();
            if cursor_face_type != 3
                && let Some(chunk) = self.get_chunk(x, z)
            {
                let pos = BlockPos::new(x, y, z);
                let block_state = chunk.get_block_state(&pos);
                // Java asks `isSuffocating` first; the checks are pure, and asking the face and
                // edge ones first keeps the default suffocation predicate, which needs block tags,
                // from being asked about blocks those rule out.
                if (cursor_face_type != 1 || block_state.has_large_collision_shape())
                    && (cursor_face_type != 2 || block_state.is(Blocks::MOVING_PISTON))
                    && (!self.only_suffocating_blocks || block_state.is_suffocating_at(chunk, &pos))
                {
                    let block_shape =
                        self.context
                            .get_collision_shape(block_state, self.collision_getter, &pos);
                    if block_shape == Shapes::block() {
                        if self.r#box.intersects_xyz(
                            f64::from(x),
                            f64::from(y),
                            f64::from(z),
                            f64::from(x) + 1.0,
                            f64::from(y) + 1.0,
                            f64::from(z) + 1.0,
                        ) {
                            return Some((self.result_provider)(pos, block_shape.move_vec3i(&pos)));
                        }
                    } else {
                        let shape = block_shape.move_vec3i(&pos);
                        if !shape.is_empty()
                            && Shapes::join_is_not_empty(&shape, &self.entity_shape, BooleanOp::AND)
                        {
                            return Some((self.result_provider)(pos, shape));
                        }
                    }
                }
            }
        }
        None
    }
}
