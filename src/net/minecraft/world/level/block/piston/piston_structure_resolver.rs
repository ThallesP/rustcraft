//! net/minecraft/world/level/block/piston/PistonStructureResolver.java
//!
//! What a piston's move takes with it: the blocks in a line in front of it, with the slime and
//! honey blocks' neighbours that stick to them, up to 12, and the blocks in the way that pop.

use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::piston::piston_base_block::PistonBaseBlock;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::push_reaction::PushReaction;

pub struct PistonStructureResolver<'a> {
    level: &'a ClientLevel,
    piston_pos: BlockPos,
    extending: bool,
    start_pos: BlockPos,
    push_direction: Direction,
    to_push: Vec<BlockPos>,
    to_destroy: Vec<BlockPos>,
    piston_direction: Direction,
}

impl<'a> PistonStructureResolver<'a> {
    pub const MAX_PUSH_DEPTH: usize = 12;

    pub fn new(
        level: &'a ClientLevel,
        piston_pos: BlockPos,
        direction: Direction,
        extending: bool,
    ) -> PistonStructureResolver<'a> {
        let (push_direction, start_pos) = if extending {
            (direction, piston_pos.relative(direction))
        } else {
            (
                direction.get_opposite(),
                piston_pos.relative_steps(direction, 2),
            )
        };
        PistonStructureResolver {
            level,
            piston_pos,
            extending,
            start_pos,
            push_direction,
            to_push: Vec::new(),
            to_destroy: Vec::new(),
            piston_direction: direction,
        }
    }

    /// `resolve()`: whether the piston can move, with what it moves and pops worked out.
    pub fn resolve(&mut self) -> bool {
        self.to_push.clear();
        self.to_destroy.clear();
        let next_state = self.level.get_block_state(&self.start_pos);
        if !PistonBaseBlock::is_pushable(
            next_state,
            self.level,
            &self.start_pos,
            self.push_direction,
            false,
            self.piston_direction,
        ) {
            if self.extending && next_state.get_piston_push_reaction() == PushReaction::POPPED {
                self.to_destroy.push(self.start_pos);
                return true;
            }
            return false;
        }

        if !self.add_block_line(self.start_pos, self.push_direction) {
            return false;
        }

        let mut i = 0;
        while i < self.to_push.len() {
            let pos = self.to_push[i];
            if PistonStructureResolver::is_sticky(self.level.get_block_state(&pos))
                && !self.add_branching_blocks(pos)
            {
                return false;
            }
            i += 1;
        }

        true
    }

    fn is_sticky(state: BlockState) -> bool {
        state.is(Blocks::SLIME_BLOCK) || state.is(Blocks::HONEY_BLOCK)
    }

    /// `canStickToEachOther`: a sticky block and another, but not slime and honey.
    fn can_stick_to_each_other(state1: BlockState, state2: BlockState) -> bool {
        let slime_and_honey = state1.is(Blocks::HONEY_BLOCK) && state2.is(Blocks::SLIME_BLOCK)
            || state1.is(Blocks::SLIME_BLOCK) && state2.is(Blocks::HONEY_BLOCK);
        !slime_and_honey
            && (PistonStructureResolver::is_sticky(state1)
                || PistonStructureResolver::is_sticky(state2))
    }

    /// `addBlockLine`: the line from `start` back along the push, sticky blocks taking the
    /// ones behind them, then the blocks it pushes in front; false when that's too many or
    /// something can't move.
    fn add_block_line(&mut self, start: BlockPos, direction: Direction) -> bool {
        let mut next_state = self.level.get_block_state(&start);
        if next_state.is_air() {
            return true;
        }

        if !PistonBaseBlock::is_pushable(
            next_state,
            self.level,
            &start,
            self.push_direction,
            false,
            direction,
        ) {
            return true;
        }

        if start == self.piston_pos {
            return true;
        }

        if self.to_push.contains(&start) {
            return true;
        }

        let mut block_count = 1;
        if block_count + self.to_push.len() > PistonStructureResolver::MAX_PUSH_DEPTH {
            return false;
        }

        while PistonStructureResolver::is_sticky(next_state) {
            let pos = start.relative_steps(self.push_direction.get_opposite(), block_count as i32);
            let previous_state = next_state;
            next_state = self.level.get_block_state(&pos);
            if next_state.is_air()
                || !PistonStructureResolver::can_stick_to_each_other(previous_state, next_state)
                || !PistonBaseBlock::is_pushable(
                    next_state,
                    self.level,
                    &pos,
                    self.push_direction,
                    false,
                    self.push_direction.get_opposite(),
                )
                || pos == self.piston_pos
            {
                break;
            }

            block_count += 1;
            if block_count + self.to_push.len() > PistonStructureResolver::MAX_PUSH_DEPTH {
                return false;
            }
        }

        let mut blocks_added = 0;
        for i in (0..block_count).rev() {
            self.to_push
                .push(start.relative_steps(self.push_direction.get_opposite(), i as i32));
            blocks_added += 1;
        }

        let mut i = 1;
        loop {
            let pos = start.relative_steps(self.push_direction, i);
            if let Some(collision_pos) = self.to_push.iter().position(|&pushed| pushed == pos) {
                self.reorder_list_at_collision(blocks_added, collision_pos);

                for j in 0..=collision_pos + blocks_added {
                    let block_pos = self.to_push[j];
                    if PistonStructureResolver::is_sticky(self.level.get_block_state(&block_pos))
                        && !self.add_branching_blocks(block_pos)
                    {
                        return false;
                    }
                }

                return true;
            }

            next_state = self.level.get_block_state(&pos);
            if next_state.is_air() {
                return true;
            }

            if !PistonBaseBlock::is_pushable(
                next_state,
                self.level,
                &pos,
                self.push_direction,
                true,
                self.push_direction,
            ) || pos == self.piston_pos
            {
                return false;
            }

            if next_state.get_piston_push_reaction() == PushReaction::POPPED {
                self.to_destroy.push(pos);
                return true;
            }

            if self.to_push.len() >= PistonStructureResolver::MAX_PUSH_DEPTH {
                return false;
            }

            self.to_push.push(pos);
            blocks_added += 1;
            i += 1;
        }
    }

    /// `reorderListAtCollision`: the line just added moved in front of the blocks it ran into.
    fn reorder_list_at_collision(&mut self, blocks_added: usize, collision_pos: usize) {
        let len = self.to_push.len();
        let head = self.to_push[..collision_pos].to_vec();
        let last_line_added = self.to_push[len - blocks_added..].to_vec();
        let collision_to_line = self.to_push[collision_pos..len - blocks_added].to_vec();
        self.to_push.clear();
        self.to_push.extend(head);
        self.to_push.extend(last_line_added);
        self.to_push.extend(collision_to_line);
    }

    /// `addBranchingBlocks`: the lines of the blocks sticking to a sticky block's sides.
    fn add_branching_blocks(&mut self, from_pos: BlockPos) -> bool {
        let from_state = self.level.get_block_state(&from_pos);
        for direction in Direction::values() {
            if direction.get_axis() != self.push_direction.get_axis() {
                let neighbour_pos = from_pos.relative(direction);
                let neighbour_state = self.level.get_block_state(&neighbour_pos);
                if PistonStructureResolver::can_stick_to_each_other(neighbour_state, from_state)
                    && !self.add_block_line(neighbour_pos, direction)
                {
                    return false;
                }
            }
        }

        true
    }

    pub fn get_push_direction(&self) -> Direction {
        self.push_direction
    }

    pub fn get_to_push(&self) -> &[BlockPos] {
        &self.to_push
    }

    pub fn get_to_destroy(&self) -> &[BlockPos] {
        &self.to_destroy
    }
}
