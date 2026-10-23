//! net/minecraft/world/level/lighting/BlockLightEngine.java
//!
//! Not ported yet: `propagateLightSources`, which only the server calls, when it lights a chunk it
//! made (the client gets its light from the server).

use std::ops::{Deref, DerefMut};

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::chunk::light_chunk_getter::LightChunkGetter;
use crate::net::minecraft::world::level::lighting::block_light_section_storage::BlockLightSectionStorage;
use crate::net::minecraft::world::level::lighting::light_engine::{
    LightEngine, LightEngineClass, QueueEntry,
};

/// The blocks' light: from each light source out, one less a block, less through what dims it.
pub struct BlockLightEngine {
    base: LightEngine<BlockLightSectionStorage>,
}

impl BlockLightEngine {
    pub fn new() -> BlockLightEngine {
        BlockLightEngine {
            base: LightEngine::new(BlockLightSectionStorage::new()),
        }
    }

    /// The block's light, if its column's light is on.
    fn get_emission(&self, block_node: i64, state: BlockState) -> i32 {
        let emission = state.get_light_emission();
        if emission > 0
            && self
                .storage
                .light_on_in_section(SectionPos::block_to_section(block_node))
        {
            emission
        } else {
            0
        }
    }
}

impl Default for BlockLightEngine {
    fn default() -> BlockLightEngine {
        BlockLightEngine::new()
    }
}

impl LightEngineClass for BlockLightEngine {
    type Storage = BlockLightSectionStorage;

    fn check_node(&mut self, chunk_source: &dyn LightChunkGetter, block_node: i64) {
        let section_node = SectionPos::block_to_section(block_node);
        if self.storage.storing_light_for_section(section_node) {
            let state = LightEngine::get_state(chunk_source, &BlockPos::of(block_node));
            let light_emission = self.get_emission(block_node, state);
            let old_level = self.storage.get_stored_level(block_node);
            if light_emission < old_level {
                self.storage.set_stored_level(block_node, 0);
                self.enqueue_decrease(block_node, QueueEntry::decrease_all_directions(old_level));
            } else {
                self.enqueue_decrease(block_node, LightEngine::PULL_LIGHT_IN_ENTRY);
            }

            if light_emission > 0 {
                self.enqueue_increase(
                    block_node,
                    QueueEntry::increase_light_from_emission(
                        light_emission,
                        LightEngine::is_empty_shape(state),
                    ),
                );
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
                            }
                        }
                    }
                }
            }
        }
    }

    fn propagate_decrease(
        &mut self,
        chunk_source: &dyn LightChunkGetter,
        from_node: i64,
        decrease_data: i64,
    ) {
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
                            let to_state =
                                LightEngine::get_state(chunk_source, &BlockPos::of(to_node));
                            let to_emission = self.get_emission(to_node, to_state);
                            self.storage.set_stored_level(to_node, 0);
                            if to_emission < to_level {
                                self.enqueue_decrease(
                                    to_node,
                                    QueueEntry::decrease_skip_one_direction(
                                        to_level,
                                        propagation_direction.get_opposite(),
                                    ),
                                );
                            }

                            if to_emission > 0 {
                                self.enqueue_increase(
                                    to_node,
                                    QueueEntry::increase_light_from_emission(
                                        to_emission,
                                        LightEngine::is_empty_shape(to_state),
                                    ),
                                );
                            }
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

impl Deref for BlockLightEngine {
    type Target = LightEngine<BlockLightSectionStorage>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for BlockLightEngine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
