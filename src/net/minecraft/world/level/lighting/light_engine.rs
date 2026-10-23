//! net/minecraft/world/level/lighting/LightEngine.java
//!
//! What both light engines share: the blocks to check, the queues of decreases and increases,
//! and running them (`runLightUpdates`). Java's abstract class is generic in its storage; here
//! it's a struct the block light's and the sky light's engines hold and deref to, and its abstract
//! methods (`checkNode`, `propagateIncrease`, `propagateDecrease`) are the `LightEngineClass`
//! trait, which `run_light_updates` takes. The chunks the engine reads are a parameter of what
//! reads them (Java's `chunkSource` field). Its cache of the last two chunks it read isn't
//! ported: it only saves lookups, since chunks don't change while it runs.
//!
//! `blockNodesToCheck` is fastutil's `LongOpenHashSet`, ported, whose table's order the checks
//! run in: it decides the order of the queues, and so how many entries a run takes.

use std::collections::VecDeque;

use crate::it::unimi::dsi::fastutil::longs::long_open_hash_set::LongOpenHashSet;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::chunk::data_layer::DataLayer;
use crate::net::minecraft::world::level::chunk::light_chunk_getter::LightChunkGetter;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;
use crate::net::minecraft::world::level::lighting::data_layer_storage_map::SharedDataLayer;
use crate::net::minecraft::world::level::lighting::layer_light_section_storage::{
    self, LayerLightSectionStorageClass, SectionType,
};
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::ops::DerefMut;

/// What both engines keep: their storage, and the work queued for `runLightUpdates`.
pub struct LightEngine<S> {
    pub(super) storage: S,
    block_nodes_to_check: LongOpenHashSet,
    decrease_queue: VecDeque<i64>,
    increase_queue: VecDeque<i64>,
}

impl LightEngine<()> {
    pub const MAX_LEVEL: i32 = 15;
    pub(super) const MIN_OPACITY: i32 = 1;
    const MIN_QUEUE_SIZE: i32 = 512;
    pub(super) const PULL_LIGHT_IN_ENTRY: i64 = QueueEntry::decrease_all_directions(1);
    pub(super) const PROPAGATION_DIRECTIONS: [Direction; 6] = Direction::VALUES;

    /// Whether a block changing from one state to the other changes how light goes through it.
    pub fn has_different_light_properties(old_state: BlockState, new_state: BlockState) -> bool {
        if new_state == old_state {
            false
        } else {
            new_state.get_light_dampening() != old_state.get_light_dampening()
                || new_state.get_light_emission() != old_state.get_light_emission()
                || new_state.use_shape_for_light_occlusion()
                || old_state.use_shape_for_light_occlusion()
        }
    }

    /// The face of the state light sees going that way: none for a state light ignores the shape
    /// of.
    pub fn get_occlusion_shape(state: BlockState, direction: Direction) -> VoxelShape {
        if LightEngine::is_empty_shape(state) {
            Shapes::empty()
        } else {
            state.get_face_occlusion_shape(direction)
        }
    }

    pub(super) fn is_empty_shape(state: BlockState) -> bool {
        !state.can_occlude() || !state.use_shape_for_light_occlusion()
    }

    /// The block at a position, or bedrock in a chunk the source doesn't have.
    pub(super) fn get_state(chunk_source: &dyn LightChunkGetter, pos: &BlockPos) -> BlockState {
        let chunk_x = SectionPos::block_to_section_coord(pos.get_x());
        let chunk_z = SectionPos::block_to_section_coord(pos.get_z());
        match chunk_source.get_chunk_for_lighting(chunk_x, chunk_z) {
            None => Blocks::BEDROCK.default_block_state(),
            Some(chunk) => chunk.get_block_state(pos),
        }
    }

    pub(super) fn get_opacity(state: BlockState) -> i32 {
        LightEngine::MIN_OPACITY.max(state.get_light_dampening())
    }

    pub(super) fn shape_occludes(
        from_state: BlockState,
        to_state: BlockState,
        direction: Direction,
    ) -> bool {
        let from_shape = LightEngine::get_occlusion_shape(from_state, direction);
        let to_shape = LightEngine::get_occlusion_shape(to_state, direction.get_opposite());
        Shapes::face_shape_occludes(&from_shape, &to_shape)
    }
}

impl<S: LayerLightSectionStorageClass> LightEngine<S> {
    pub(super) fn new(storage: S) -> LightEngine<S> {
        LightEngine {
            storage,
            block_nodes_to_check: LongOpenHashSet::new(LightEngine::MIN_QUEUE_SIZE, 0.5),
            decrease_queue: VecDeque::new(),
            increase_queue: VecDeque::new(),
        }
    }

    pub fn check_block(&mut self, pos: &BlockPos) {
        self.block_nodes_to_check.add(pos.as_long());
    }

    pub fn queue_section_data(&mut self, pos: i64, data: Option<SharedDataLayer>) {
        self.storage.queue_section_data(pos, data);
    }

    pub fn retain_data(&mut self, pos: ChunkPos, retain: bool) {
        self.storage
            .retain_data(SectionPos::get_zero_node_xz(pos.x(), pos.z()), retain);
    }

    pub fn update_section_status(&mut self, pos: SectionPos, section_empty: bool) {
        layer_light_section_storage::update_section_status(
            &mut self.storage,
            pos.as_long(),
            section_empty,
        );
    }

    /// `LightEngine.setLightEnabled`, which the sky's engine adds to.
    pub(super) fn set_light_enabled_base(&mut self, pos: ChunkPos, enable: bool) {
        self.storage
            .set_light_enabled(SectionPos::get_zero_node_xz(pos.x(), pos.z()), enable);
    }

    pub(super) fn enqueue_decrease(&mut self, from_node: i64, decrease_data: i64) {
        self.decrease_queue.push_back(from_node);
        self.decrease_queue.push_back(decrease_data);
    }

    pub(super) fn enqueue_increase(&mut self, from_node: i64, increase_data: i64) {
        self.increase_queue.push_back(from_node);
        self.increase_queue.push_back(increase_data);
    }

    pub fn has_light_work(&self) -> bool {
        self.storage.has_inconsistencies()
            || !self.block_nodes_to_check.is_empty()
            || !self.decrease_queue.is_empty()
            || !self.increase_queue.is_empty()
    }

    pub fn get_data_layer_data(&self, pos: &SectionPos) -> Option<DataLayer> {
        self.storage.get_data_layer_data(pos.as_long())
    }

    pub fn get_light_value(&self, pos: &BlockPos) -> i32 {
        self.storage.get_light_value(pos.as_long())
    }

    pub fn get_debug_data(&self, section_node: i64) -> String {
        self.get_debug_section_type(section_node)
            .display()
            .to_owned()
    }

    pub fn get_debug_section_type(&self, section_node: i64) -> SectionType {
        self.storage.get_debug_section_type(section_node)
    }
}

/// The abstract methods of an engine.
pub trait LightEngineClass: DerefMut<Target = LightEngine<Self::Storage>> {
    type Storage: LayerLightSectionStorageClass;

    fn check_node(&mut self, chunk_source: &dyn LightChunkGetter, block_node: i64);

    fn propagate_increase(
        &mut self,
        chunk_source: &dyn LightChunkGetter,
        from_node: i64,
        increase_data: i64,
        from_level: i32,
    );

    fn propagate_decrease(
        &mut self,
        chunk_source: &dyn LightChunkGetter,
        from_node: i64,
        decrease_data: i64,
    );
}

/// Checks the blocks queued, runs the decreases and then the increases they queued, and swaps
/// in what came out. How many entries ran.
pub fn run_light_updates(
    this: &mut impl LightEngineClass,
    chunk_source: &dyn LightChunkGetter,
) -> i32 {
    // A check never queues another, so the set doesn't change while it's iterated.
    let block_nodes_to_check: Vec<i64> = this.block_nodes_to_check.iter().collect();
    for block_node in block_nodes_to_check {
        this.check_node(chunk_source, block_node);
    }

    this.block_nodes_to_check.clear();
    this.block_nodes_to_check.trim(LightEngine::MIN_QUEUE_SIZE);

    let mut count = 0;
    count += propagate_decreases(this, chunk_source);
    count += propagate_increases(this, chunk_source);
    layer_light_section_storage::mark_new_inconsistencies(&mut this.storage);
    this.storage.swap_section_map(chunk_source);
    count
}

fn propagate_increases(
    this: &mut impl LightEngineClass,
    chunk_source: &dyn LightChunkGetter,
) -> i32 {
    let mut count = 0;
    while let Some(from_node) = this.increase_queue.pop_front() {
        let increase_data = this
            .increase_queue
            .pop_front()
            .expect("increases are queued in pairs");
        let mut from_level = this.storage.get_stored_level(from_node);
        let from_target_level = QueueEntry::get_from_level(increase_data);
        if QueueEntry::is_increase_from_emission(increase_data) && from_level < from_target_level {
            this.storage.set_stored_level(from_node, from_target_level);
            from_level = from_target_level;
        }

        if from_level == from_target_level {
            this.propagate_increase(chunk_source, from_node, increase_data, from_level);
        }
        count += 1;
    }

    count
}

fn propagate_decreases(
    this: &mut impl LightEngineClass,
    chunk_source: &dyn LightChunkGetter,
) -> i32 {
    let mut count = 0;
    while let Some(from_node) = this.decrease_queue.pop_front() {
        let decrease_data = this
            .decrease_queue
            .pop_front()
            .expect("decreases are queued in pairs");
        this.propagate_decrease(chunk_source, from_node, decrease_data);
        count += 1;
    }

    count
}

/// `LightEngine.QueueEntry`: a queued entry's level (4 bits), the directions it goes (6 bits, by
/// ordinal), whether it came from a block light ignores the shape of, and whether it's a light
/// source's own increase.
pub struct QueueEntry;

impl QueueEntry {
    const FROM_LEVEL_BITS: i32 = 4;
    const DIRECTION_BITS: i32 = 6;
    const LEVEL_MASK: i64 = 15;
    const DIRECTIONS_MASK: i64 = 1008;
    const FLAG_FROM_EMPTY_SHAPE: i64 = 1024;
    const FLAG_INCREASE_FROM_EMISSION: i64 = 2048;

    pub const fn decrease_skip_one_direction(
        old_from_level: i32,
        skip_direction: Direction,
    ) -> i64 {
        let decrease_data =
            QueueEntry::without_direction(QueueEntry::DIRECTIONS_MASK, skip_direction);
        QueueEntry::with_level(decrease_data, old_from_level)
    }

    pub const fn decrease_all_directions(old_from_level: i32) -> i64 {
        QueueEntry::with_level(QueueEntry::DIRECTIONS_MASK, old_from_level)
    }

    pub const fn increase_light_from_emission(new_from_level: i32, from_empty_shape: bool) -> i64 {
        let mut increase_data = QueueEntry::DIRECTIONS_MASK;
        increase_data |= QueueEntry::FLAG_INCREASE_FROM_EMISSION;
        if from_empty_shape {
            increase_data |= QueueEntry::FLAG_FROM_EMPTY_SHAPE;
        }

        QueueEntry::with_level(increase_data, new_from_level)
    }

    pub const fn increase_skip_one_direction(
        new_from_level: i32,
        from_empty_shape: bool,
        skip_direction: Direction,
    ) -> i64 {
        let mut increase_data =
            QueueEntry::without_direction(QueueEntry::DIRECTIONS_MASK, skip_direction);
        if from_empty_shape {
            increase_data |= QueueEntry::FLAG_FROM_EMPTY_SHAPE;
        }

        QueueEntry::with_level(increase_data, new_from_level)
    }

    pub const fn increase_only_one_direction(
        new_from_level: i32,
        from_empty_shape: bool,
        direction: Direction,
    ) -> i64 {
        let mut increase_data = 0;
        if from_empty_shape {
            increase_data |= QueueEntry::FLAG_FROM_EMPTY_SHAPE;
        }

        increase_data = QueueEntry::with_direction(increase_data, direction);
        QueueEntry::with_level(increase_data, new_from_level)
    }

    pub const fn increase_sky_source_in_directions(
        down: bool,
        north: bool,
        south: bool,
        west: bool,
        east: bool,
    ) -> i64 {
        let mut increase_data = QueueEntry::with_level(0, 15);
        if down {
            increase_data = QueueEntry::with_direction(increase_data, Direction::DOWN);
        }

        if north {
            increase_data = QueueEntry::with_direction(increase_data, Direction::NORTH);
        }

        if south {
            increase_data = QueueEntry::with_direction(increase_data, Direction::SOUTH);
        }

        if west {
            increase_data = QueueEntry::with_direction(increase_data, Direction::WEST);
        }

        if east {
            increase_data = QueueEntry::with_direction(increase_data, Direction::EAST);
        }

        increase_data
    }

    pub const fn get_from_level(entry: i64) -> i32 {
        (entry & QueueEntry::LEVEL_MASK) as i32
    }

    pub const fn is_from_empty_shape(entry: i64) -> bool {
        entry & QueueEntry::FLAG_FROM_EMPTY_SHAPE != 0
    }

    pub const fn is_increase_from_emission(entry: i64) -> bool {
        entry & QueueEntry::FLAG_INCREASE_FROM_EMISSION != 0
    }

    pub const fn should_propagate_in_direction(entry: i64, direction: Direction) -> bool {
        entry & 1 << (direction as i32 + QueueEntry::FROM_LEVEL_BITS) != 0
    }

    const fn with_level(entry: i64, level: i32) -> i64 {
        entry & -16 | (level as i64 & QueueEntry::LEVEL_MASK)
    }

    const fn with_direction(entry: i64, direction: Direction) -> i64 {
        entry | 1 << (direction as i32 + QueueEntry::FROM_LEVEL_BITS)
    }

    const fn without_direction(entry: i64, direction: Direction) -> i64 {
        entry & !(1 << (direction as i32 + QueueEntry::FROM_LEVEL_BITS))
    }
}

// `DIRECTION_BITS` only documents the layout in Java too.
const _: i32 = QueueEntry::DIRECTION_BITS;
