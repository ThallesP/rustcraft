//! net/minecraft/world/level/block/entity/BeaconBlockEntity.java
//!
//! A beacon's beam and its pyramid's levels, as its ticker works them out on both sides (`tick`):
//! ten blocks up the column a tick to the world surface, a section of a colour for each run of
//! stained glass, ended by a block that blocks the light, then every 80 ticks the levels of the
//! pyramid under it. Its effects and sounds are the server's: on the client `applyEffects` does
//! nothing and its sounds have no listener (`playSound` with no player). Not ported yet: its
//! effects and lock (`primary_effect`, `secondary_effect`, `lock`), its name, the menu's data and
//! `validateEffects`.

use crate::net::minecraft::client::multiplayer::client_level::{ClientLevel, LevelTicking};
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::world::level::block::beacon_beam_block;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::entity::beacon_beam_owner::Section;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntityData;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::level_height_accessor::LevelHeightAccessor;
use crate::net::minecraft::world::level::levelgen::heightmap::Types;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct BeaconBlockEntity {
    beam_sections: Vec<Section>,
    checking_beam_sections: Vec<Section>,
    levels: i32,
    last_check_y: i32,
}

impl BeaconBlockEntity {
    const BLOCKS_CHECK_PER_TICK: i32 = 10;

    pub fn new() -> BeaconBlockEntity {
        BeaconBlockEntity::default()
    }

    /// `tick`. Java hands the checked sections over to the beam (`beamSections =
    /// checkingBeamSections`), and makes the next check a new list before anything changes
    /// either, so a copy is the same.
    pub fn tick(level: &mut LevelTicking<'_>, pos: &BlockPos, _self_state: BlockState) {
        let level = &*level.level;
        let mut block_entity = level
            .get_block_entity_mut(pos)
            .expect("a ticker's block entity is the level's");
        let BlockEntityData::Beacon(entity) = &mut block_entity.data else {
            return;
        };
        let x = pos.get_x();
        let y = pos.get_y();
        let z = pos.get_z();
        let mut check_pos;
        if entity.last_check_y < y {
            check_pos = *pos;
            entity.checking_beam_sections = Vec::new();
            entity.last_check_y = check_pos.get_y() - 1;
        } else {
            check_pos = BlockPos::new(x, entity.last_check_y + 1, z);
        }

        // The last section, by its index in the checking sections; past the end once they're
        // cleared, which ends the check.
        let mut last_beam_section = entity.checking_beam_sections.len().checked_sub(1);
        let last_set_block = level.get_height_at(Types::WORLD_SURFACE, x, z);

        let mut i = 0;
        while i < BeaconBlockEntity::BLOCKS_CHECK_PER_TICK && check_pos.get_y() <= last_set_block {
            let state = level.get_block_state(&check_pos);
            if let Some(color) = beacon_beam_block::get_color(state.get_block()) {
                let color = color.get_texture_diffuse_color();
                if entity.checking_beam_sections.len() <= 1 {
                    entity.checking_beam_sections.push(Section::new(color));
                    last_beam_section = Some(entity.checking_beam_sections.len() - 1);
                } else if let Some(last) = last_beam_section {
                    let last_color = entity.checking_beam_sections[last].get_color();
                    if color == last_color {
                        entity.checking_beam_sections[last].increase_height();
                    } else {
                        entity
                            .checking_beam_sections
                            .push(Section::new(ARGB::average(last_color, color)));
                        last_beam_section = Some(entity.checking_beam_sections.len() - 1);
                    }
                }
            } else {
                match last_beam_section {
                    Some(last)
                        if !(state.get_light_dampening() >= 15 && !state.is(Blocks::BEDROCK)) =>
                    {
                        entity.checking_beam_sections[last].increase_height();
                    }
                    _ => {
                        entity.checking_beam_sections.clear();
                        entity.last_check_y = last_set_block;
                        break;
                    }
                }
            }

            check_pos = check_pos.above();
            entity.last_check_y += 1;
            i += 1;
        }

        if level.get_game_time() % 80 == 0 && !entity.beam_sections.is_empty() {
            entity.levels = BeaconBlockEntity::update_base(level, x, y, z);
        }

        if entity.last_check_y >= last_set_block {
            entity.last_check_y = LevelHeightAccessor::get_min_y(level) - 1;
            entity.beam_sections = entity.checking_beam_sections.clone();
        }
    }

    /// `updateBase`: how many layers of the pyramid under it are whole, up to 4.
    fn update_base(level: &ClientLevel, x: i32, y: i32, z: i32) -> i32 {
        let mut levels = 0;
        let mut step = 1;
        while step <= 4 {
            let ly = y - step;
            if ly < LevelHeightAccessor::get_min_y(level) {
                break;
            }

            let mut is_ok = true;
            let mut lx = x - step;
            while lx <= x + step && is_ok {
                for lz in z - step..=z + step {
                    if !level
                        .get_block_state(&BlockPos::new(lx, ly, lz))
                        .is_tag(BlockTags::BEACON_BASE_BLOCKS)
                    {
                        is_ok = false;
                        break;
                    }
                }
                lx += 1;
            }

            if !is_ok {
                break;
            }
            levels = step;
            step += 1;
        }

        levels
    }

    /// `getBeamSections()`: none without a pyramid under it.
    pub fn get_beam_sections(&self) -> &[Section] {
        if self.levels == 0 {
            &[]
        } else {
            &self.beam_sections
        }
    }

    /// `setLevel`: the beam checked from the bottom of the level up.
    pub fn set_level(&mut self, min_y: i32) {
        self.last_check_y = min_y - 1;
    }

    pub fn checking_beam_sections(&self) -> &[Section] {
        &self.checking_beam_sections
    }

    pub fn levels(&self) -> i32 {
        self.levels
    }

    pub fn last_check_y(&self) -> i32 {
        self.last_check_y
    }
}
