//! net/minecraft/world/level/Level.java, as far as entities read it, and its block entities'
//! tickers.
//!
//! Java's `Level` is the world an entity lives in; the client's is `ClientLevel`, which the world
//! track brings with chunk storage. Until then this is what an entity's movement asks of it
//! besides collisions, and tests implement it over a few blocks. Entities hold their level in
//! Java; here it's a parameter of what reads it.
//!
//! The level's block entity tickers (`blockEntityTickers`, `pendingSharedLevelState` and
//! `tickingBlockEntities`) are a struct the level shares with its chunks, which add their block
//! entities' tickers to it (`addBlockEntityTicker`, called through Java's back-reference from a
//! chunk to its level); `tickBlockEntities` is `ClientLevel`'s.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::chunk::level_chunk::RebindableTickingBlockEntityWrapper;
use crate::net::minecraft::world::level::collision_getter::CollisionGetter;
use crate::net::minecraft::world::phys::aabb::AABB;

/// What a level shares with its chunks: its block entity tickers, those it ticks in the order
/// they came and those that came while it ticked them, which it ticks from its next tick on; and
/// its game time.
#[derive(Default)]
pub struct SharedLevelState {
    pub(crate) block_entity_tickers: RefCell<Vec<Rc<RebindableTickingBlockEntityWrapper>>>,
    pub(crate) pending_block_entity_tickers: RefCell<Vec<Rc<RebindableTickingBlockEntityWrapper>>>,
    pub(crate) ticking_block_entities: Cell<bool>,
    /// `levelData`'s game time.
    pub(crate) game_time: Cell<i64>,
}

impl SharedLevelState {
    pub fn new() -> SharedLevelState {
        SharedLevelState::default()
    }

    /// `addBlockEntityTicker`: to the tickers, or to the pending ones while the level ticks them.
    pub fn add_block_entity_ticker(&self, ticker: Rc<RebindableTickingBlockEntityWrapper>) {
        if self.ticking_block_entities.get() {
            self.pending_block_entity_tickers.borrow_mut().push(ticker);
        } else {
            self.block_entity_tickers.borrow_mut().push(ticker);
        }
    }

    pub fn get_game_time(&self) -> i64 {
        self.game_time.get()
    }

    /// The tickers in the order the level ticks them, without the pending ones.
    pub fn tickers(&self) -> Vec<Rc<RebindableTickingBlockEntityWrapper>> {
        self.block_entity_tickers.borrow().clone()
    }
}

/// A level, as an entity's tick reads it.
pub trait Level: CollisionGetter {
    fn is_client_side(&self) -> bool;

    /// `getMinY()`, the lowest block's height.
    fn get_min_y(&self) -> i32;

    /// `hasChunkAt(BlockPos)`: whether the block's chunk is loaded.
    fn has_chunk_at(&self, pos: &BlockPos) -> bool;

    /// Whether any section from `section_y0` to `section_y1` of the chunk holds a fluid, `None`
    /// when the chunk isn't loaded: `getChunk(x, z, ChunkStatus.FULL, false)` and its sections'
    /// `hasFluid()`, which `EntityFluidInteraction` asks before it reads fluid states.
    fn chunk_has_fluid(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        section_y0: i32,
        section_y1: i32,
    ) -> Option<bool>;

    /// `environmentAttributes().getDimensionValue(EnvironmentAttributes.FAST_LAVA)`: whether lava
    /// flows fast here, and pushes harder.
    fn has_fast_lava(&self) -> bool;
}

/// `LevelReader.containsAnyLiquid(AABB)`: whether a block from the box's lower corner's to its
/// upper corner's holds a fluid. Java scans by section, skipping those without one, which finds
/// the same.
pub fn contains_any_liquid(level: &dyn Level, r#box: &AABB) -> bool {
    let from = BlockPos::containing_xyz(r#box.min_x, r#box.min_y, r#box.min_z);
    let to = BlockPos::containing_xyz(r#box.max_x, r#box.max_y, r#box.max_z);
    for x in from.get_x().min(to.get_x())..=from.get_x().max(to.get_x()) {
        for y in from.get_y().min(to.get_y())..=from.get_y().max(to.get_y()) {
            for z in from.get_z().min(to.get_z())..=from.get_z().max(to.get_z()) {
                if !level.get_fluid_state(&BlockPos::new(x, y, z)).is_empty() {
                    return true;
                }
            }
        }
    }
    false
}

/// The level as the collisions see it.
pub fn collisions(level: &dyn Level) -> &dyn CollisionGetter {
    level
}
