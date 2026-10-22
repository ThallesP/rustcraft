//! net/minecraft/world/phys/BlockHitResult.java
//!
//! Not ported yet: `STREAM_CODEC`, which waits for the network buffers.

use std::ops::Deref;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::phys::hit_result::{HitResult, Type};
use crate::net::minecraft::world::phys::vec3::Vec3;

/// A pick that hit a block's side, or a miss that still says where it ended and which block
/// that was.
#[derive(Clone, Copy, Debug)]
pub struct BlockHitResult {
    hit_result: HitResult,
    direction: Direction,
    block_pos: BlockPos,
    miss: bool,
    inside: bool,
    world_border_hit: bool,
}

impl Deref for BlockHitResult {
    type Target = HitResult;

    fn deref(&self) -> &HitResult {
        &self.hit_result
    }
}

impl BlockHitResult {
    pub fn miss(location: &Vec3, direction: Direction, pos: &BlockPos) -> BlockHitResult {
        BlockHitResult::from_miss(true, location, direction, pos, false, false)
    }

    pub fn new(
        location: &Vec3,
        direction: Direction,
        pos: &BlockPos,
        inside: bool,
    ) -> BlockHitResult {
        BlockHitResult::from_miss(false, location, direction, pos, inside, false)
    }

    pub fn from_world_border_hit(
        location: &Vec3,
        direction: Direction,
        pos: &BlockPos,
        inside: bool,
        world_border_hit: bool,
    ) -> BlockHitResult {
        BlockHitResult::from_miss(false, location, direction, pos, inside, world_border_hit)
    }

    /// Java's private constructor, the only one that sets every field.
    fn from_miss(
        miss: bool,
        location: &Vec3,
        direction: Direction,
        block_pos: &BlockPos,
        inside: bool,
        world_border_hit: bool,
    ) -> BlockHitResult {
        BlockHitResult {
            hit_result: HitResult::new(*location),
            direction,
            block_pos: *block_pos,
            miss,
            inside,
            world_border_hit,
        }
    }

    pub fn with_direction(&self, direction: Direction) -> BlockHitResult {
        BlockHitResult::from_miss(
            self.miss,
            &self.location,
            direction,
            &self.block_pos,
            self.inside,
            self.world_border_hit,
        )
    }

    pub fn with_position(&self, block_pos: &BlockPos) -> BlockHitResult {
        BlockHitResult::from_miss(
            self.miss,
            &self.location,
            self.direction,
            block_pos,
            self.inside,
            self.world_border_hit,
        )
    }

    pub fn hit_border(&self) -> BlockHitResult {
        BlockHitResult::from_miss(
            self.miss,
            &self.location,
            self.direction,
            &self.block_pos,
            self.inside,
            true,
        )
    }

    pub fn get_block_pos(&self) -> BlockPos {
        self.block_pos
    }

    pub fn get_direction(&self) -> Direction {
        self.direction
    }

    pub fn get_type(&self) -> Type {
        if self.miss { Type::MISS } else { Type::BLOCK }
    }

    pub fn is_inside(&self) -> bool {
        self.inside
    }

    pub fn is_world_border_hit(&self) -> bool {
        self.world_border_hit
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::BlockHitResult;
    use crate::golden::{Decode, Encode, Golden, elements};
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::direction::Direction;
    use crate::net::minecraft::world::phys::hit_result::HitResult;
    use crate::net::minecraft::world::phys::vec3::Vec3;

    /// `[location, direction, blockPos, miss, inside, worldBorderHit]`, `HitResult`'s field first.
    impl Decode for BlockHitResult {
        fn decode(value: &Value) -> Self {
            let [
                location,
                direction,
                block_pos,
                miss,
                inside,
                world_border_hit,
            ] = elements(value);
            BlockHitResult {
                hit_result: HitResult::new(Vec3::decode(location)),
                direction: Direction::decode(direction),
                block_pos: BlockPos::decode(block_pos),
                miss: bool::decode(miss),
                inside: bool::decode(inside),
                world_border_hit: bool::decode(world_border_hit),
            }
        }
    }

    impl Encode for BlockHitResult {
        fn encode(&self) -> Value {
            Value::from(vec![
                self.location.encode(),
                self.direction.encode(),
                self.block_pos.encode(),
                self.miss.encode(),
                self.inside.encode(),
                self.world_border_hit.encode(),
            ])
        }

        fn matches(&self, expected: &Value) -> bool {
            match expected.as_array().map(Vec::as_slice) {
                Some(
                    [
                        location,
                        direction,
                        block_pos,
                        miss,
                        inside,
                        world_border_hit,
                    ],
                ) => {
                    self.location.matches(location)
                        && self.direction.matches(direction)
                        && self.block_pos.matches(block_pos)
                        && self.miss.matches(miss)
                        && self.inside.matches(inside)
                        && self.world_border_hit.matches(world_border_hit)
                }
                _ => false,
            }
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.phys.BlockHitResult");
        golden.pending("STREAM_CODEC", "waits for the network buffers");
        golden.check(
            "miss(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/core/Direction;Lnet/minecraft/core/BlockPos;)Lnet/minecraft/world/phys/BlockHitResult;",
            |location: Vec3, direction, pos: BlockPos| {
                BlockHitResult::miss(&location, direction, &pos)
            },
        );
        golden.check(
            "<init>(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/core/Direction;Lnet/minecraft/core/BlockPos;Z)V",
            |location: Vec3, direction, pos: BlockPos, inside| {
                BlockHitResult::new(&location, direction, &pos, inside)
            },
        );
        golden.check(
            "<init>(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/core/Direction;Lnet/minecraft/core/BlockPos;ZZ)V",
            |location: Vec3, direction, pos: BlockPos, inside, world_border_hit| {
                BlockHitResult::from_world_border_hit(
                    &location,
                    direction,
                    &pos,
                    inside,
                    world_border_hit,
                )
            },
        );
        golden.check_method(
            "withDirection(Lnet/minecraft/core/Direction;)Lnet/minecraft/world/phys/BlockHitResult;",
            |this: &mut BlockHitResult, direction| this.with_direction(direction),
        );
        golden.check_method(
            "withPosition(Lnet/minecraft/core/BlockPos;)Lnet/minecraft/world/phys/BlockHitResult;",
            |this: &mut BlockHitResult, block_pos: BlockPos| this.with_position(&block_pos),
        );
        golden.check_method(
            "hitBorder()Lnet/minecraft/world/phys/BlockHitResult;",
            |this: &mut BlockHitResult| this.hit_border(),
        );
        golden.check_method(
            "getBlockPos()Lnet/minecraft/core/BlockPos;",
            |this: &mut BlockHitResult| this.get_block_pos(),
        );
        golden.check_method(
            "getDirection()Lnet/minecraft/core/Direction;",
            |this: &mut BlockHitResult| this.get_direction(),
        );
        golden.check_method(
            "getType()Lnet/minecraft/world/phys/HitResult$Type;",
            |this: &mut BlockHitResult| this.get_type(),
        );
        golden.check_method("isInside()Z", |this: &mut BlockHitResult| this.is_inside());
        golden.check_method("isWorldBorderHit()Z", |this: &mut BlockHitResult| {
            this.is_world_border_hit()
        });
        golden.finish();
    }
}
