//! net/minecraft/client/color/block/BlockTintCache.java
//!
//! A level's tints of one resolver, worked out once per block: by chunk (the 256 most recent), by
//! y, then the column, -1 until worked out. Java's is shared by the threads that mesh sections,
//! with locks and the latest chunk per thread; the port's is the game thread's, so its state is in
//! cells, its latest chunk one for all. Its source, Java's function of the level, is a parameter
//! of `get_color`, since the level owns the cache.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;

const MAX_CACHE_ENTRIES: usize = 256;
const BLOCKS_PER_LAYER: usize = 16 * 16;

/// The tints worked out so far.
#[derive(Default)]
pub struct BlockTintCache {
    latest_chunk: RefCell<LatestCacheInfo>,
    /// Java's `Long2ObjectLinkedOpenHashMap`, in the order its chunks came.
    cache: RefCell<HashMap<i64, Rc<CacheData>>>,
    order: RefCell<VecDeque<i64>>,
}

impl BlockTintCache {
    pub fn new() -> BlockTintCache {
        BlockTintCache::default()
    }

    /// The block's tint, worked out with `source` the first time.
    pub fn get_color(&self, pos: &BlockPos, source: impl FnOnce(&BlockPos) -> i32) -> i32 {
        let chunk_x = SectionPos::block_to_section_coord(pos.get_x());
        let chunk_z = SectionPos::block_to_section_coord(pos.get_z());
        let cache = {
            let mut chunk_info = self.latest_chunk.borrow_mut();
            let stale = match &chunk_info.cache {
                None => true,
                Some(cache) => {
                    chunk_info.x != chunk_x || chunk_info.z != chunk_z || cache.is_invalidated()
                }
            };
            if stale {
                chunk_info.x = chunk_x;
                chunk_info.z = chunk_z;
                chunk_info.cache = Some(self.find_or_create_chunk_cache(chunk_x, chunk_z));
            }
            Rc::clone(chunk_info.cache.as_ref().expect("set above"))
        };

        let x = pos.get_x() & 15;
        let z = pos.get_z() & 15;
        let index = (z << 4 | x) as usize;
        let cached = cache.get_layer(pos.get_y(), |layer| layer[index]);
        if cached != -1 {
            return cached;
        }

        let calculated = source(pos);
        cache.get_layer(pos.get_y(), |layer| layer[index] = calculated);
        calculated
    }

    /// Drops the chunk's tints and its neighbours', whose blends reach into it.
    pub fn invalidate_for_chunk(&self, chunk_x: i32, chunk_z: i32) {
        for offset_x in -1..=1 {
            for offset_z in -1..=1 {
                let key = ChunkPos::pack_xz(chunk_x + offset_x, chunk_z + offset_z);
                if let Some(removed) = self.cache.borrow_mut().remove(&key) {
                    self.order.borrow_mut().retain(|&kept| kept != key);
                    removed.invalidate();
                }
            }
        }
    }

    pub fn invalidate_all(&self) {
        for cache_data in self.cache.borrow().values() {
            cache_data.invalidate();
        }
        self.cache.borrow_mut().clear();
        self.order.borrow_mut().clear();
    }

    fn find_or_create_chunk_cache(&self, x: i32, z: i32) -> Rc<CacheData> {
        let key = ChunkPos::pack_xz(x, z);
        if let Some(existing) = self.cache.borrow().get(&key) {
            return Rc::clone(existing);
        }

        let new_cache = Rc::new(CacheData::default());
        if self.cache.borrow().len() >= MAX_CACHE_ENTRIES
            && let Some(first) = self.order.borrow_mut().pop_front()
            && let Some(cache_data) = self.cache.borrow_mut().remove(&first)
        {
            cache_data.invalidate();
        }

        self.cache.borrow_mut().insert(key, Rc::clone(&new_cache));
        self.order.borrow_mut().push_back(key);
        new_cache
    }
}

/// `BlockTintCache.CacheData`: a chunk's layers, by y.
#[derive(Default)]
struct CacheData {
    cache: RefCell<HashMap<i32, Box<[i32; BLOCKS_PER_LAYER]>>>,
    invalidated: Cell<bool>,
}

impl CacheData {
    /// The layer at y, made the first time, all -1.
    fn get_layer<R>(&self, y: i32, with: impl FnOnce(&mut [i32; BLOCKS_PER_LAYER]) -> R) -> R {
        let mut cache = self.cache.borrow_mut();
        with(
            cache
                .entry(y)
                .or_insert_with(|| Box::new([-1; BLOCKS_PER_LAYER])),
        )
    }

    fn is_invalidated(&self) -> bool {
        self.invalidated.get()
    }

    fn invalidate(&self) {
        self.invalidated.set(true);
    }
}

/// `BlockTintCache.LatestCacheInfo`.
struct LatestCacheInfo {
    x: i32,
    z: i32,
    cache: Option<Rc<CacheData>>,
}

impl Default for LatestCacheInfo {
    fn default() -> LatestCacheInfo {
        LatestCacheInfo {
            x: i32::MIN,
            z: i32::MIN,
            cache: None,
        }
    }
}
