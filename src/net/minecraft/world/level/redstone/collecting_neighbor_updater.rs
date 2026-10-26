//! net/minecraft/world/level/redstone/CollectingNeighborUpdater.java
//!
//! The level's neighbour updates, run one at a time: an update that one running adds waits for it
//! to finish, then runs before those queued before it, its layer's in the order they came, so the
//! updates go depth first. Of the updates only `ShapeUpdate` is ported: the client's level sends
//! no other (`Level.neighborChanged` and `updateNeighborsAt` do nothing there). Nor is the debug
//! listener, which only the server's debug renderer sets.
//!
//! Java's updater holds its level; here each call is handed it, and the updater is the level's
//! (`ClientLevel::neighbor_updater`), its queues behind cells so an update can run with the level
//! borrowed mutably.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::redstone::neighbor_updater::NeighborUpdater;

pub struct CollectingNeighborUpdater {
    max_chained_neighbor_updates: i32,
    /// An `ArrayDeque` used as a stack: pushed onto, peeked and popped at its front.
    stack: RefCell<VecDeque<ShapeUpdate>>,
    added_this_layer: RefCell<Vec<ShapeUpdate>>,
    count: Cell<i32>,
}

impl CollectingNeighborUpdater {
    pub fn new(max_chained_neighbor_updates: i32) -> CollectingNeighborUpdater {
        CollectingNeighborUpdater {
            max_chained_neighbor_updates,
            stack: RefCell::new(VecDeque::new()),
            added_this_layer: RefCell::new(Vec::new()),
            count: Cell::new(0),
        }
    }

    /// `shapeUpdate(direction, neighborState, pos, neighborPos, updateFlags, updateLimit)`, on
    /// the level's updater.
    #[allow(clippy::too_many_arguments)]
    pub fn shape_update(
        level: &mut ClientLevel,
        direction: Direction,
        neighbor_state: BlockState,
        pos: &BlockPos,
        neighbor_pos: &BlockPos,
        update_flags: i32,
        update_limit: i32,
    ) {
        CollectingNeighborUpdater::add_and_run(
            level,
            pos,
            ShapeUpdate {
                direction,
                neighbor_state,
                pos: *pos,
                neighbor_pos: *neighbor_pos,
                update_flags,
                update_limit,
            },
        );
    }

    /// `addAndRun(pos, update)`: queued behind the running update if there's one, run now if not.
    /// Past `maxChainedNeighborUpdates` in one run the rest are dropped, the first said so.
    fn add_and_run(level: &mut ClientLevel, pos: &BlockPos, update: ShapeUpdate) {
        let updater = level.neighbor_updater();
        let count = updater.count.get();
        let running_already = count > 0;
        let too_many_updates = updater.max_chained_neighbor_updates >= 0
            && count >= updater.max_chained_neighbor_updates;
        updater.count.set(count + 1);
        if !too_many_updates {
            if running_already {
                updater.added_this_layer.borrow_mut().push(update);
            } else {
                updater.stack.borrow_mut().push_front(update);
            }
        } else if count == updater.max_chained_neighbor_updates {
            eprintln!(
                "Too many chained neighbor updates. Skipping the rest. First skipped position: {}",
                pos.to_short_string()
            );
        }

        if !running_already {
            CollectingNeighborUpdater::run_updates(level);
        }
    }

    /// `runUpdates()`: the queued updates, each one's own added ones first. A shape update runs
    /// once (`runNext` is false), so it's popped when it has run.
    fn run_updates(level: &mut ClientLevel) {
        loop {
            let next_updates = {
                let updater = level.neighbor_updater();
                let mut stack = updater.stack.borrow_mut();
                let mut added_this_layer = updater.added_this_layer.borrow_mut();
                if stack.is_empty() && added_this_layer.is_empty() {
                    break;
                }
                for update in added_this_layer.drain(..).rev() {
                    stack.push_front(update);
                }
                *stack.front().expect("the stack has an update")
            };
            next_updates.run_next(level);
            level.neighbor_updater().stack.borrow_mut().pop_front();
        }

        let updater = level.neighbor_updater();
        updater.stack.borrow_mut().clear();
        updater.added_this_layer.borrow_mut().clear();
        updater.count.set(0);
    }
}

/// `ShapeUpdate`: a block told its neighbour that way is now the neighbour's state.
#[derive(Clone, Copy)]
struct ShapeUpdate {
    direction: Direction,
    neighbor_state: BlockState,
    pos: BlockPos,
    neighbor_pos: BlockPos,
    update_flags: i32,
    update_limit: i32,
}

impl ShapeUpdate {
    /// `runNext(level)`: the update, once.
    fn run_next(self, level: &mut ClientLevel) {
        NeighborUpdater::execute_shape_update(
            level,
            self.direction,
            &self.pos,
            &self.neighbor_pos,
            self.neighbor_state,
            self.update_flags,
            self.update_limit,
        );
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::golden::{Decode, Golden};
    use crate::java::lang::r#enum::Enum;
    use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::direction::Direction;
    use crate::net::minecraft::util::random_source::RandomSource;
    use crate::net::minecraft::world::level::block::block::Block;
    use crate::net::minecraft::world::level::block::entity::ticking_block_entity::tests::{
        walk_chunks, walk_level,
    };
    use crate::net::minecraft::world::level::block_getter::BlockGetter;
    use crate::net::minecraft::world::level::levelgen::legacy_random_source::LegacyRandomSource;

    /// Where the golden's ops set blocks, and around it the blocks a change can reach.
    const MIN_X: i32 = -3;
    const MIN_Y: i32 = -61;
    const MIN_Z: i32 = -3;
    const SIZE_XZ: i32 = 7;
    const SIZE_Y: i32 = 5;
    const MARGIN: i32 = 2;

    /// Each block around where the ops set blocks, x, then z, then y, as the golden goes.
    fn around() -> Vec<BlockPos> {
        let mut positions = Vec::new();
        for y in MIN_Y - MARGIN..MIN_Y + SIZE_Y + MARGIN {
            for z in MIN_Z - MARGIN..MIN_Z + SIZE_XZ + MARGIN {
                for x in MIN_X - MARGIN..MIN_X + SIZE_XZ + MARGIN {
                    positions.push(BlockPos::new(x, y, z));
                }
            }
        }
        positions
    }

    fn states(level: &ClientLevel, positions: &[BlockPos]) -> Vec<u32> {
        positions
            .iter()
            .map(|pos| level.get_block_state(pos).id())
            .collect()
    }

    /// Blocks set on the walk's level with the flags and limits vanilla's were set with, each
    /// set's shape updates going around like vanilla's client: the blocks that changed after
    /// each, then each block's `updateShape` for each neighbour, and the level's random after
    /// (TESTING.md, "Shape updates").
    #[test]
    fn shape_updates_match_vanillas() {
        let golden = Golden::load("net.minecraft.world.level.redstone.CollectingNeighborUpdater");
        let (registries, packets) = walk_chunks();
        let positions = around();
        let programs = golden.cases("programs");
        assert_eq!(programs.len(), 64);
        for (number, (args, expected)) in programs.into_iter().enumerate() {
            let mut level = walk_level(&registries, &packets);
            *level.get_random() = LegacyRandomSource::new(i64::decode(&args[0]));
            let mut lines = Vec::new();
            for op in args[1].as_array().expect("ops") {
                let op = Vec::<i32>::decode(op);
                let before = states(&level, &positions);
                level.set_block(
                    &BlockPos::new(op[0], op[1], op[2]),
                    Block::state_by_id(op[3]),
                    op[4],
                    op[5],
                );
                let after = states(&level, &positions);
                let changes: Vec<Value> = positions
                    .iter()
                    .zip(before.iter().zip(&after))
                    .filter(|(_, (before, after))| before != after)
                    .map(|(pos, (_, after))| {
                        Value::from(vec![
                            Value::from(pos.get_x()),
                            Value::from(pos.get_y()),
                            Value::from(pos.get_z()),
                            Value::from(*after),
                        ])
                    })
                    .collect();
                lines.push(Value::from(changes));
            }

            let mut shapes = Vec::new();
            for pos in &positions {
                let state = level.get_block_state(pos);
                for direction in Direction::values() {
                    let neighbour_pos = pos.relative(direction);
                    let updated = state.update_shape(
                        &level,
                        pos,
                        direction,
                        &neighbour_pos,
                        level.get_block_state(&neighbour_pos),
                        &mut *level.get_random(),
                    );
                    if updated != state {
                        shapes.push(Value::from(vec![
                            Value::from(pos.get_x()),
                            Value::from(pos.get_y()),
                            Value::from(pos.get_z()),
                            Value::from(direction.ordinal()),
                            Value::from(updated.id()),
                        ]));
                    }
                }
            }
            lines.push(Value::from(shapes));
            lines.push(Value::from(level.get_random().next_long()));

            let expected = expected.as_array().expect("lines");
            assert_eq!(lines.len(), expected.len(), "program {number}");
            for (line, (actual, expected)) in lines.iter().zip(expected).enumerate() {
                assert_eq!(actual, expected, "program {number}, line {line}");
            }
        }
        golden.finish();
    }
}
