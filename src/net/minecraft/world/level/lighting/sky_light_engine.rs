//! net/minecraft/world/level/lighting/SkyLightEngine.java
//!
//! The sky's light: full (15) from each column's lowest source up, spreading sideways and down
//! from there like the blocks' light. Not ported yet: `propagateLightSources`, which only the
//! server calls, when it lights a chunk it made (the client gets its light from the server).

use std::ops::{Deref, DerefMut};

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::chunk::light_chunk_getter::LightChunkGetter;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;
use crate::net::minecraft::world::level::level_height_accessor::Height;
use crate::net::minecraft::world::level::lighting::chunk_sky_light_sources::ChunkSkyLightSources;
use crate::net::minecraft::world::level::lighting::light_engine::{
    LightEngine, LightEngineClass, QueueEntry,
};
use crate::net::minecraft::world::level::lighting::sky_light_section_storage::SkyLightSectionStorage;

const REMOVE_TOP_SKY_SOURCE_ENTRY: i64 = QueueEntry::decrease_all_directions(15);
const REMOVE_SKY_SOURCE_ENTRY: i64 = QueueEntry::decrease_skip_one_direction(15, Direction::UP);
const ADD_SKY_SOURCE_ENTRY: i64 = QueueEntry::increase_skip_one_direction(15, false, Direction::UP);

/// The sky's light.
pub struct SkyLightEngine {
    base: LightEngine<SkyLightSectionStorage>,
    empty_chunk_sources: ChunkSkyLightSources,
}

impl SkyLightEngine {
    /// For a level of that height, which a chunk the source doesn't have takes its empty sources
    /// from.
    pub fn new(level_height: Height) -> SkyLightEngine {
        SkyLightEngine {
            base: LightEngine::new(SkyLightSectionStorage::new()),
            empty_chunk_sources: ChunkSkyLightSources::new(&level_height),
        }
    }

    fn is_source_level(value: i32) -> bool {
        value == 15
    }

    fn get_lowest_source_y(
        chunk_source: &dyn LightChunkGetter,
        x: i32,
        z: i32,
        default_value: i32,
    ) -> i32 {
        match SkyLightEngine::get_chunk_sources(
            chunk_source,
            SectionPos::block_to_section_coord(x),
            SectionPos::block_to_section_coord(z),
        ) {
            None => default_value,
            Some(sources) => sources.get_lowest_source_y(
                SectionPos::section_relative(x),
                SectionPos::section_relative(z),
            ),
        }
    }

    fn get_chunk_sources(
        chunk_source: &dyn LightChunkGetter,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Option<&ChunkSkyLightSources> {
        chunk_source
            .get_chunk_for_lighting(chunk_x, chunk_z)
            .map(|chunk| chunk.get_sky_light_sources())
    }

    fn update_sources_in_column(
        &mut self,
        chunk_source: &dyn LightChunkGetter,
        x: i32,
        z: i32,
        lowest_source_y: i32,
    ) {
        let world_bottom_y =
            SectionPos::section_to_block_coord(self.storage.get_bottom_section_y());
        self.remove_sources_below(x, z, lowest_source_y, world_bottom_y);
        self.add_sources_above(chunk_source, x, z, lowest_source_y, world_bottom_y);
    }

    fn remove_sources_below(&mut self, x: i32, z: i32, lowest_source_y: i32, world_bottom_y: i32) {
        if lowest_source_y > world_bottom_y {
            let section_x = SectionPos::block_to_section_coord(x);
            let section_z = SectionPos::block_to_section_coord(z);
            let start_y = lowest_source_y - 1;

            let mut section_y = SectionPos::block_to_section_coord(start_y);
            while self.storage.has_light_data_at_or_below(section_y) {
                if self
                    .storage
                    .storing_light_for_section(SectionPos::as_long_xyz(
                        section_x, section_y, section_z,
                    ))
                {
                    let section_bottom_y = SectionPos::section_to_block_coord(section_y);
                    let section_top_y = section_bottom_y + 15;

                    let mut y = section_top_y.min(start_y);
                    while y >= section_bottom_y {
                        let block_node = BlockPos::as_long_xyz(x, y, z);
                        if !SkyLightEngine::is_source_level(
                            self.storage.get_stored_level(block_node),
                        ) {
                            return;
                        }

                        self.storage.set_stored_level(block_node, 0);
                        self.enqueue_decrease(
                            block_node,
                            if y == lowest_source_y - 1 {
                                REMOVE_TOP_SKY_SOURCE_ENTRY
                            } else {
                                REMOVE_SKY_SOURCE_ENTRY
                            },
                        );
                        y -= 1;
                    }
                }
                section_y -= 1;
            }
        }
    }

    fn add_sources_above(
        &mut self,
        chunk_source: &dyn LightChunkGetter,
        x: i32,
        z: i32,
        lowest_source_y: i32,
        world_bottom_y: i32,
    ) {
        let section_x = SectionPos::block_to_section_coord(x);
        let section_z = SectionPos::block_to_section_coord(z);
        let neighbor_lowest_source_y =
            SkyLightEngine::get_lowest_source_y(chunk_source, x - 1, z, i32::MIN)
                .max(SkyLightEngine::get_lowest_source_y(
                    chunk_source,
                    x + 1,
                    z,
                    i32::MIN,
                ))
                .max(
                    SkyLightEngine::get_lowest_source_y(chunk_source, x, z - 1, i32::MIN).max(
                        SkyLightEngine::get_lowest_source_y(chunk_source, x, z + 1, i32::MIN),
                    ),
                );
        let start_y = lowest_source_y.max(world_bottom_y);

        let mut section_node = SectionPos::as_long_xyz(
            section_x,
            SectionPos::block_to_section_coord(start_y),
            section_z,
        );
        while !self.storage.is_above_data(section_node) {
            if self.storage.storing_light_for_section(section_node) {
                let section_bottom_y =
                    SectionPos::section_to_block_coord(SectionPos::y_of(section_node));
                let section_top_y = section_bottom_y + 15;

                for y in section_bottom_y.max(start_y)..=section_top_y {
                    let block_node = BlockPos::as_long_xyz(x, y, z);
                    if SkyLightEngine::is_source_level(self.storage.get_stored_level(block_node)) {
                        return;
                    }

                    self.storage.set_stored_level(block_node, 15);
                    if y < neighbor_lowest_source_y || y == lowest_source_y {
                        self.enqueue_increase(block_node, ADD_SKY_SOURCE_ENTRY);
                    }
                }
            }
            section_node = SectionPos::offset_i64(section_node, Direction::UP);
        }
    }

    /// How many sections below a block at the bottom edge of its section aren't stored, down to
    /// the lowest stored section: 0 for a block not at a section's bottom and side.
    fn count_empty_sections_below_if_at_border(&self, block_node: i64) -> i32 {
        let y = BlockPos::get_y_i64(block_node);
        let local_y = SectionPos::section_relative(y);
        if local_y != 0 {
            return 0;
        }

        let x = BlockPos::get_x_i64(block_node);
        let z = BlockPos::get_z_i64(block_node);
        let local_x = SectionPos::section_relative(x);
        let local_z = SectionPos::section_relative(z);
        if local_x != 0 && local_x != 15 && local_z != 0 && local_z != 15 {
            return 0;
        }

        let section_x = SectionPos::block_to_section_coord(x);
        let section_y = SectionPos::block_to_section_coord(y);
        let section_z = SectionPos::block_to_section_coord(z);
        let mut empty_sections_below = 0;

        while !self
            .storage
            .storing_light_for_section(SectionPos::as_long_xyz(
                section_x,
                section_y - empty_sections_below - 1,
                section_z,
            ))
            && self
                .storage
                .has_light_data_at_or_below(section_y - empty_sections_below - 1)
        {
            empty_sections_below += 1;
        }

        empty_sections_below
    }

    /// Light crossing a section's side above sections that aren't stored also reaches the
    /// stored sections below them in the column it went into.
    fn propagate_from_empty_sections(
        &mut self,
        to_node: i64,
        propagation_direction: Direction,
        to_level: i32,
        increase: bool,
        empty_sections_below: i32,
    ) {
        if empty_sections_below != 0 {
            let x = BlockPos::get_x_i64(to_node);
            let z = BlockPos::get_z_i64(to_node);
            if SkyLightEngine::crossed_section_edge(
                propagation_direction,
                SectionPos::section_relative(x),
                SectionPos::section_relative(z),
            ) {
                let y = BlockPos::get_y_i64(to_node);
                let section_x = SectionPos::block_to_section_coord(x);
                let section_z = SectionPos::block_to_section_coord(z);
                let mut section_y = SectionPos::block_to_section_coord(y) - 1;
                let bottom_section_y = section_y - empty_sections_below + 1;

                while section_y >= bottom_section_y {
                    if !self
                        .storage
                        .storing_light_for_section(SectionPos::as_long_xyz(
                            section_x, section_y, section_z,
                        ))
                    {
                        section_y -= 1;
                    } else {
                        let section_min_y = SectionPos::section_to_block_coord(section_y);

                        for local_y in (0..=15).rev() {
                            let block_node = BlockPos::as_long_xyz(x, section_min_y + local_y, z);
                            if increase {
                                self.storage.set_stored_level(block_node, to_level);
                                if to_level > 1 {
                                    self.enqueue_increase(
                                        block_node,
                                        QueueEntry::increase_skip_one_direction(
                                            to_level,
                                            true,
                                            propagation_direction.get_opposite(),
                                        ),
                                    );
                                }
                            } else {
                                self.storage.set_stored_level(block_node, 0);
                                self.enqueue_decrease(
                                    block_node,
                                    QueueEntry::decrease_skip_one_direction(
                                        to_level,
                                        propagation_direction.get_opposite(),
                                    ),
                                );
                            }
                        }

                        section_y -= 1;
                    }
                }
            }
        }
    }

    fn crossed_section_edge(propagation_direction: Direction, x: i32, z: i32) -> bool {
        match propagation_direction {
            Direction::NORTH => z == 15,
            Direction::SOUTH => z == 0,
            Direction::WEST => x == 15,
            Direction::EAST => x == 0,
            _ => false,
        }
    }

    /// `setLightEnabled`: turns the column's light on or off, and when on fills its empty layers
    /// above the chunk's sources with full light.
    pub fn set_light_enabled(
        &mut self,
        chunk_source: &dyn LightChunkGetter,
        pos: ChunkPos,
        enable: bool,
    ) {
        self.set_light_enabled_base(pos, enable);
        if enable {
            let sources = SkyLightEngine::get_chunk_sources(chunk_source, pos.x(), pos.z())
                .unwrap_or(&self.empty_chunk_sources);
            // `NEGATIVE_INFINITY - 1` wraps to the largest int, like Java's: no section is filled.
            let highest_non_source_y = sources.get_highest_lowest_source_y().wrapping_sub(1);
            let lowest_fully_source_section_y =
                SectionPos::block_to_section_coord(highest_non_source_y) + 1;
            let zero_node = SectionPos::get_zero_node_xz(pos.x(), pos.z());
            let top_section_y = self.storage.get_top_section_y(zero_node);
            let bottom_section_y = self
                .storage
                .get_bottom_section_y()
                .max(lowest_fully_source_section_y);

            let mut section_y = top_section_y - 1;
            while section_y >= bottom_section_y {
                if let Some(data_layer) = self
                    .storage
                    .get_data_layer_to_write(SectionPos::as_long_xyz(pos.x(), section_y, pos.z()))
                {
                    let mut data_layer = data_layer.borrow_mut();
                    if data_layer.is_empty() {
                        data_layer.fill(15);
                    }
                }
                section_y -= 1;
            }
        }
    }
}

impl LightEngineClass for SkyLightEngine {
    type Storage = SkyLightSectionStorage;

    fn check_node(&mut self, chunk_source: &dyn LightChunkGetter, block_node: i64) {
        let x = BlockPos::get_x_i64(block_node);
        let y = BlockPos::get_y_i64(block_node);
        let z = BlockPos::get_z_i64(block_node);
        let section_node = SectionPos::block_to_section(block_node);
        let lowest_source_y = if self.storage.light_on_in_section(section_node) {
            SkyLightEngine::get_lowest_source_y(chunk_source, x, z, i32::MAX)
        } else {
            i32::MAX
        };
        if lowest_source_y != i32::MAX {
            self.update_sources_in_column(chunk_source, x, z, lowest_source_y);
        }

        if self.storage.storing_light_for_section(section_node) {
            let is_source = y >= lowest_source_y;
            if is_source {
                self.enqueue_decrease(block_node, REMOVE_SKY_SOURCE_ENTRY);
                self.enqueue_increase(block_node, ADD_SKY_SOURCE_ENTRY);
            } else {
                let old_level = self.storage.get_stored_level(block_node);
                if old_level > 0 {
                    self.storage.set_stored_level(block_node, 0);
                    self.enqueue_decrease(
                        block_node,
                        QueueEntry::decrease_all_directions(old_level),
                    );
                } else {
                    self.enqueue_decrease(block_node, LightEngine::PULL_LIGHT_IN_ENTRY);
                }
            }
        }
    }

    fn propagate_increase(
        &mut self,
        chunk_source: &dyn LightChunkGetter,
        from_node: i64,
        increase_data: i64,
        from_level: i32,
    ) {
        let mut from_state = None;
        let empty_sections_below = self.count_empty_sections_below_if_at_border(from_node);

        for propagation_direction in LightEngine::PROPAGATION_DIRECTIONS {
            if QueueEntry::should_propagate_in_direction(increase_data, propagation_direction) {
                let to_node = BlockPos::offset_i64(from_node, propagation_direction);
                if self
                    .storage
                    .storing_light_for_section(SectionPos::block_to_section(to_node))
                {
                    let to_level = self.storage.get_stored_level(to_node);
                    let max_possible_new_to_level = from_level - 1;
                    if max_possible_new_to_level > to_level {
                        let to_state = LightEngine::get_state(chunk_source, &BlockPos::of(to_node));
                        let new_to_level = from_level - LightEngine::get_opacity(to_state);
                        if new_to_level > to_level {
                            let from_state = *from_state.get_or_insert_with(|| {
                                if QueueEntry::is_from_empty_shape(increase_data) {
                                    Blocks::AIR.default_block_state()
                                } else {
                                    LightEngine::get_state(chunk_source, &BlockPos::of(from_node))
                                }
                            });

                            if !LightEngine::shape_occludes(
                                from_state,
                                to_state,
                                propagation_direction,
                            ) {
                                self.storage.set_stored_level(to_node, new_to_level);
                                if new_to_level > 1 {
                                    self.enqueue_increase(
                                        to_node,
                                        QueueEntry::increase_skip_one_direction(
                                            new_to_level,
                                            LightEngine::is_empty_shape(to_state),
                                            propagation_direction.get_opposite(),
                                        ),
                                    );
                                }

                                self.propagate_from_empty_sections(
                                    to_node,
                                    propagation_direction,
                                    new_to_level,
                                    true,
                                    empty_sections_below,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn propagate_decrease(
        &mut self,
        _chunk_source: &dyn LightChunkGetter,
        from_node: i64,
        decrease_data: i64,
    ) {
        let empty_sections_below = self.count_empty_sections_below_if_at_border(from_node);
        let old_from_level = QueueEntry::get_from_level(decrease_data);

        for propagation_direction in LightEngine::PROPAGATION_DIRECTIONS {
            if QueueEntry::should_propagate_in_direction(decrease_data, propagation_direction) {
                let to_node = BlockPos::offset_i64(from_node, propagation_direction);
                if self
                    .storage
                    .storing_light_for_section(SectionPos::block_to_section(to_node))
                {
                    let to_level = self.storage.get_stored_level(to_node);
                    if to_level != 0 {
                        if to_level < old_from_level {
                            self.storage.set_stored_level(to_node, 0);
                            self.enqueue_decrease(
                                to_node,
                                QueueEntry::decrease_skip_one_direction(
                                    to_level,
                                    propagation_direction.get_opposite(),
                                ),
                            );
                            self.propagate_from_empty_sections(
                                to_node,
                                propagation_direction,
                                to_level,
                                false,
                                empty_sections_below,
                            );
                        } else {
                            self.enqueue_increase(
                                to_node,
                                QueueEntry::increase_only_one_direction(
                                    to_level,
                                    false,
                                    propagation_direction.get_opposite(),
                                ),
                            );
                        }
                    }
                }
            }
        }
    }
}

impl Deref for SkyLightEngine {
    type Target = LightEngine<SkyLightSectionStorage>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for SkyLightEngine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
