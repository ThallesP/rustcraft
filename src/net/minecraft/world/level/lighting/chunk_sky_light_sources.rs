//! net/minecraft/world/level/lighting/ChunkSkyLightSources.java
//!
//! For each column of a chunk, the lowest y the sky's light reaches undimmed: the top of the
//! highest block whose light dampening or occluding top face stops it. `fillFrom` takes the
//! client's `LevelChunk` (Java's `ChunkAccess`), and `update` reads blocks at the chunk's own
//! column coordinates, as `LevelChunk.setBlockState` asks it. Its mutable positions are plain y
//! coordinates here.
//!
//! `fillFrom` reads every column of the chunk down to its source, which is most of handling a
//! chunk: here it works out what `isEdgeOccluded` reads of each state of a section's palette once
//! (`EdgeFaces`), and decides from that where it can, which is where `isEdgeOccluded` decides
//! without joining shapes; the rest it asks `isEdgeOccluded` itself.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::util::bit_storage::BitStorage;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::simple_bit_storage::SimpleBitStorage;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::chunk::level_chunk::LevelChunk;
use crate::net::minecraft::world::level::level_height_accessor::LevelHeightAccessor;
use crate::net::minecraft::world::level::lighting::light_engine::LightEngine;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

const SIZE: i32 = 16;

/// What `isEdgeOccluded` reads of a state: whether it dampens light, and its occlusion shapes'
/// faces down (as the top of an edge) and up (as the bottom).
#[derive(Clone, Copy)]
struct EdgeFaces {
    state: BlockState,
    dampens: bool,
    down: FaceKind,
    up: FaceKind,
}

/// A face shape as `Shapes.faceShapeOccludes` first looks at it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum FaceKind {
    /// `Shapes.block()` itself.
    Block,
    /// Empty, and not the block.
    Empty,
    Other,
}

impl FaceKind {
    fn of(shape: &VoxelShape) -> FaceKind {
        if shape == Shapes::block_ref() {
            FaceKind::Block
        } else if shape.is_empty() {
            FaceKind::Empty
        } else {
            FaceKind::Other
        }
    }
}

impl EdgeFaces {
    fn of(state: BlockState) -> EdgeFaces {
        EdgeFaces {
            state,
            dampens: state.get_light_dampening() != 0,
            down: FaceKind::of(LightEngine::occlusion_shape_ref(state, Direction::DOWN)),
            up: FaceKind::of(LightEngine::occlusion_shape_ref(state, Direction::UP)),
        }
    }

    /// `isEdgeOccluded` of the two states: the bottom dampening, then `faceShapeOccludes`' first
    /// two answers from the faces' kinds, and else `isEdgeOccluded` itself.
    #[inline]
    fn is_edge_occluded(top: &EdgeFaces, bottom: &EdgeFaces) -> bool {
        if bottom.dampens || top.down == FaceKind::Block || bottom.up == FaceKind::Block {
            true
        } else if top.down == FaceKind::Empty && bottom.up == FaceKind::Empty {
            false
        } else {
            ChunkSkyLightSources::is_edge_occluded(top.state, bottom.state)
        }
    }
}

/// The lowest sky light source of each of a chunk's 256 columns.
#[derive(Clone, Debug)]
pub struct ChunkSkyLightSources {
    min_y: i32,
    heightmap: SimpleBitStorage,
}

impl ChunkSkyLightSources {
    pub const NEGATIVE_INFINITY: i32 = i32::MIN;

    /// Every column's source at the bottom, one below the level.
    pub fn new(level: &dyn LevelHeightAccessor) -> ChunkSkyLightSources {
        let min_y = level.get_min_y() - 1;
        let max_y = level.get_max_y() + 1;
        let bits = Mth::ceillog2(max_y - min_y + 1);
        ChunkSkyLightSources {
            min_y,
            heightmap: SimpleBitStorage::new(bits, 256),
        }
    }

    /// Each column's lowest source, from the chunk's blocks.
    pub fn fill_from(&mut self, chunk: &LevelChunk) {
        let max_section_index = chunk.get_highest_filled_section_index();
        if max_section_index == -1 {
            self.fill(self.min_y);
        } else {
            let mut sections = vec![None; max_section_index as usize + 1];
            for z in 0..SIZE {
                for x in 0..SIZE {
                    let initial_edge_y = self
                        .find_lowest_source_y(chunk, &mut sections, max_section_index, x, z)
                        .max(self.min_y);
                    self.set(ChunkSkyLightSources::index(x, z), initial_edge_y);
                }
            }
        }
    }

    /// `findLowestSourceY`, reading each section's states through its palette's `EdgeFaces`
    /// (`sections`, by section index, worked out the first time a column reaches it), or one
    /// state at a time for a section with the global palette.
    fn find_lowest_source_y(
        &self,
        chunk: &LevelChunk,
        sections: &mut [Option<Option<Vec<EdgeFaces>>>],
        top_section_index: i32,
        x: i32,
        z: i32,
    ) -> i32 {
        let top_y = SectionPos::section_to_block_coord(
            chunk.get_section_y_from_section_index(top_section_index) + 1,
        );
        // Java's two mutable positions, at (x, top, z) and the one below.
        let mut top_pos_y = top_y;
        let mut bottom_pos_y = top_y - 1;
        let air = EdgeFaces::of(Blocks::AIR.default_block_state());
        let mut top_state = air;

        for section_index in (0..=top_section_index).rev() {
            let section = chunk.get_section(section_index);
            if section.has_only_air() {
                top_state = air;
                let section_y = chunk.get_section_y_from_section_index(section_index);
                top_pos_y = SectionPos::section_to_block_coord(section_y);
                bottom_pos_y = top_pos_y - 1;
            } else {
                let states = section.get_states();
                let palette = sections[section_index as usize].get_or_insert_with(|| {
                    states
                        .own_palette()
                        .map(|values| values.into_iter().map(EdgeFaces::of).collect())
                });
                for y in (0..=15).rev() {
                    let bottom_state = match palette {
                        Some(palette) => palette[states.id_at(x, y, z) as usize],
                        None => EdgeFaces::of(section.get_block_state(x, y, z)),
                    };
                    if EdgeFaces::is_edge_occluded(&top_state, &bottom_state) {
                        return top_pos_y;
                    }

                    top_state = bottom_state;
                    top_pos_y = bottom_pos_y;
                    bottom_pos_y -= 1;
                }
            }
        }

        self.min_y
    }

    /// The column's source after the block at (x, y, z) changed: whether it moved.
    pub fn update(&mut self, level: &dyn BlockGetter, x: i32, y: i32, z: i32) -> bool {
        let upper_edge_y = y + 1;
        let index = ChunkSkyLightSources::index(x, z);
        let current_lowest_source_y = self.get(index);
        if upper_edge_y < current_lowest_source_y {
            return false;
        }

        let top_pos = BlockPos::new(x, y + 1, z);
        let top_state = level.get_block_state(&top_pos);
        let middle_pos = BlockPos::new(x, y, z);
        let middle_state = level.get_block_state(&middle_pos);
        if self.update_edge(
            level,
            index,
            current_lowest_source_y,
            &top_pos,
            top_state,
            &middle_pos,
            middle_state,
        ) {
            return true;
        }

        let bottom_pos = BlockPos::new(x, y - 1, z);
        let bottom_state = level.get_block_state(&bottom_pos);
        self.update_edge(
            level,
            index,
            current_lowest_source_y,
            &middle_pos,
            middle_state,
            &bottom_pos,
            bottom_state,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn update_edge(
        &mut self,
        level: &dyn BlockGetter,
        index: i32,
        old_top_edge_y: i32,
        top_pos: &BlockPos,
        top_state: BlockState,
        bottom_pos: &BlockPos,
        bottom_state: BlockState,
    ) -> bool {
        let checked_edge_y = top_pos.get_y();
        if ChunkSkyLightSources::is_edge_occluded(top_state, bottom_state) {
            if checked_edge_y > old_top_edge_y {
                self.set(index, checked_edge_y);
                return true;
            }
        } else if checked_edge_y == old_top_edge_y {
            let lowest = self.find_lowest_source_below(level, bottom_pos, bottom_state);
            self.set(index, lowest);
            return true;
        }

        false
    }

    fn find_lowest_source_below(
        &self,
        level: &dyn BlockGetter,
        start_pos: &BlockPos,
        start_state: BlockState,
    ) -> i32 {
        let mut top_pos = *start_pos;
        let mut bottom_pos = start_pos.relative(Direction::DOWN);
        let mut top_state = start_state;

        while bottom_pos.get_y() >= self.min_y {
            let bottom_state = level.get_block_state(&bottom_pos);
            if ChunkSkyLightSources::is_edge_occluded(top_state, bottom_state) {
                return top_pos.get_y();
            }

            top_state = bottom_state;
            top_pos = bottom_pos;
            bottom_pos = bottom_pos.relative(Direction::DOWN);
        }

        self.min_y
    }

    /// Whether the sky's light stops between the two blocks: the lower one dims it, or their
    /// faces between them occlude.
    fn is_edge_occluded(top_state: BlockState, bottom_state: BlockState) -> bool {
        if bottom_state.get_light_dampening() != 0 {
            return true;
        }

        let top_shape = LightEngine::occlusion_shape_ref(top_state, Direction::DOWN);
        let bottom_shape = LightEngine::occlusion_shape_ref(bottom_state, Direction::UP);
        Shapes::face_shape_occludes(top_shape, bottom_shape)
    }

    /// The column's lowest source, or `NEGATIVE_INFINITY` when the sky reaches below the level.
    pub fn get_lowest_source_y(&self, x: i32, z: i32) -> i32 {
        let value = self.get(ChunkSkyLightSources::index(x, z));
        self.extend_sources_below_world(value)
    }

    pub fn get_highest_lowest_source_y(&self) -> i32 {
        let mut max_value = i32::MIN;

        for i in 0..self.heightmap.get_size() {
            let value = self.heightmap.get(i);
            if value > max_value {
                max_value = value;
            }
        }

        self.extend_sources_below_world(max_value + self.min_y)
    }

    fn fill(&mut self, lowest_source_y: i32) {
        let value = lowest_source_y - self.min_y;

        for i in 0..self.heightmap.get_size() {
            self.heightmap.set(i, value);
        }
    }

    fn set(&mut self, index: i32, value: i32) {
        self.heightmap.set(index, value - self.min_y);
    }

    fn get(&self, index: i32) -> i32 {
        self.heightmap.get(index) + self.min_y
    }

    fn extend_sources_below_world(&self, value: i32) -> i32 {
        if value == self.min_y { i32::MIN } else { value }
    }

    fn index(x: i32, z: i32) -> i32 {
        x + z * 16
    }
}
