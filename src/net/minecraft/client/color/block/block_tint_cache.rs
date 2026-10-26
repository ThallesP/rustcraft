//! net/minecraft/client/color/block/BlockTintCache.java
//!
//! A level's tints of one resolver, worked out once per block: by chunk (the 256 most recent), by
//! y, then the column, -1 until worked out. Java's is shared by the threads that mesh sections,
//! with locks and the latest chunk per thread; the port's is the game thread's, so its state is in
//! cells, its latest chunk one for all. Its source, Java's function of the level, is a parameter
//! of `get_color`, since the level owns the cache. A chunk's layers are by y in a table that grows
//! to the ys asked for, where Java's array map searches its layers in turn: the same layers.
//!
//! A section compile on another thread reads a copy of its column's layers (`copy_layers`), taken
//! with the section's blocks, works out the tints the copy lacks the way `get_color` would, and
//! gives them back (`give_back`) unless the chunk's tints were dropped since the copy, which the
//! cache counts (`invalidations`): what a compile run when its copy was taken would have cached.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;

const MAX_CACHE_ENTRIES: usize = 256;
pub const BLOCKS_PER_LAYER: usize = 16 * 16;
/// How many chunks' last drops `invalidated_at` keeps before it forgets them all.
const MAX_INVALIDATED_CHUNKS: usize = 4096;

/// A chunk's layers at consecutive ys, by y from the first: none where the cache has none.
pub type TintLayers = Vec<Option<Box<[i32; BLOCKS_PER_LAYER]>>>;

/// The tints worked out so far.
#[derive(Default)]
pub struct BlockTintCache {
    latest_chunk: RefCell<LatestCacheInfo>,
    /// Java's `Long2ObjectLinkedOpenHashMap`, in the order its chunks came.
    cache: RefCell<HashMap<i64, Rc<CacheData>>>,
    order: RefCell<VecDeque<i64>>,
    /// How many times tints were dropped, when each chunk's last were (by that count), and when
    /// all of them last were: a chunk's copy is as of the count when it was taken.
    invalidations: Cell<u64>,
    invalidated_at: RefCell<HashMap<i64, u64>>,
    cleared_at: Cell<u64>,
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
        let invalidations = self.invalidations.get() + 1;
        self.invalidations.set(invalidations);
        let mut invalidated_at = self.invalidated_at.borrow_mut();
        if invalidated_at.len() >= MAX_INVALIDATED_CHUNKS {
            invalidated_at.clear();
            self.cleared_at.set(invalidations);
        }
        for offset_x in -1..=1 {
            for offset_z in -1..=1 {
                let key = ChunkPos::pack_xz(chunk_x + offset_x, chunk_z + offset_z);
                invalidated_at.insert(key, invalidations);
                if let Some(removed) = self.cache.borrow_mut().remove(&key) {
                    self.order.borrow_mut().retain(|&kept| kept != key);
                    removed.invalidate();
                }
            }
        }
    }

    pub fn invalidate_all(&self) {
        let invalidations = self.invalidations.get() + 1;
        self.invalidations.set(invalidations);
        self.invalidated_at.borrow_mut().clear();
        self.cleared_at.set(invalidations);
        for cache_data in self.cache.borrow().values() {
            cache_data.invalidate();
        }
        self.cache.borrow_mut().clear();
        self.order.borrow_mut().clear();
    }

    /// The count of drops a copy of the chunk's tints taken now is as of.
    fn generation(&self, key: i64) -> u64 {
        self.invalidated_at
            .borrow()
            .get(&key)
            .copied()
            .unwrap_or(0)
            .max(self.cleared_at.get())
    }

    /// The chunk's layers at `count` ys from `min_y`, as the cache has them, and the count of
    /// drops they're as of, for `give_back`. Makes no chunk or layer the cache hasn't got.
    pub fn copy_layers(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        min_y: i32,
        count: usize,
    ) -> (u64, TintLayers) {
        let key = ChunkPos::pack_xz(chunk_x, chunk_z);
        let layers = match self.cache.borrow().get(&key) {
            Some(cache_data) if !cache_data.is_invalidated() => {
                let cache = cache_data.cache.borrow();
                (0..count)
                    .map(|i| {
                        let index = min_y + i as i32 - cache.min_y;
                        (index >= 0)
                            .then(|| cache.by_y.get(index as usize).cloned().flatten())
                            .flatten()
                    })
                    .collect()
            }
            _ => vec![None; count],
        };
        (self.generation(key), layers)
    }

    /// Tints worked out over a copy of the chunk's layers (`copy_layers`'), put where the cache
    /// has none, unless the chunk's tints were dropped since the copy.
    pub fn give_back(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        min_y: i32,
        generation: u64,
        layers: TintLayers,
    ) {
        let key = ChunkPos::pack_xz(chunk_x, chunk_z);
        if self.generation(key) != generation || layers.iter().all(Option::is_none) {
            return;
        }

        let cache = self.find_or_create_chunk_cache(chunk_x, chunk_z);
        for (i, given) in layers.iter().enumerate() {
            let Some(given) = given else {
                continue;
            };
            cache.get_layer(min_y + i as i32, |layer| {
                for (cached, &given) in layer.iter_mut().zip(given.iter()) {
                    if *cached == -1 {
                        *cached = given;
                    }
                }
            });
        }
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
    cache: RefCell<Layers>,
    invalidated: Cell<bool>,
}

/// The layers made so far, by y from `min_y`.
#[derive(Default)]
struct Layers {
    min_y: i32,
    by_y: Vec<Option<Box<[i32; BLOCKS_PER_LAYER]>>>,
}

impl CacheData {
    /// The layer at y, made the first time, all -1.
    fn get_layer<R>(&self, y: i32, with: impl FnOnce(&mut [i32; BLOCKS_PER_LAYER]) -> R) -> R {
        let mut cache = self.cache.borrow_mut();
        if cache.by_y.is_empty() {
            cache.min_y = y;
        } else if y < cache.min_y {
            let below = (cache.min_y - y) as usize;
            cache
                .by_y
                .splice(0..0, std::iter::repeat_with(|| None).take(below));
            cache.min_y = y;
        }
        let index = (y - cache.min_y) as usize;
        if index >= cache.by_y.len() {
            cache.by_y.resize_with(index + 1, || None);
        }
        with(cache.by_y[index].get_or_insert_with(|| Box::new([-1; BLOCKS_PER_LAYER])))
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
