//! net/minecraft/world/level/block/entity/BellBlockEntity.java
//!
//! A bell's shaking, as the client sees it: the server's block event rings it from a side
//! (`triggerEvent`, which finds the living entities around it too), and the client's ticker shakes
//! it for 50 ticks (`clientTick`). A bell resonates when raiders are near; the level's other
//! entities aren't told apart as living entities yet (their classes aren't ported), so the ones it
//! finds are the player's alone, which is no raider, and it never resonates here: the resonance's
//! sound and its particles at the raiders (`showBellParticles`) aren't ported. Not ported yet
//! either: `onHit` (the server's ringing) and the raiders' glow (`serverTick`).

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntityData;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::phys::aabb::AABB;

#[derive(Clone, Debug, PartialEq)]
pub struct BellBlockEntity {
    last_ring_timestamp: i64,
    pub ticks: i32,
    pub shaking: bool,
    pub click_direction: Option<Direction>,
    /// `nearbyEntities`: how many it found, none before it first looked (Java's null).
    nearby_entities: Option<usize>,
    resonating: bool,
    resonation_ticks: i32,
}

impl BellBlockEntity {
    const DURATION: i32 = 50;
    const MIN_TICKS_BETWEEN_SEARCHES: i64 = 60;
    const MAX_RESONATION_TICKS: i32 = 40;
    const SEARCH_RADIUS: f64 = 48.0;

    pub fn new() -> BellBlockEntity {
        BellBlockEntity {
            last_ring_timestamp: 0,
            ticks: 0,
            shaking: false,
            click_direction: None,
            nearby_entities: None,
            resonating: false,
            resonation_ticks: 0,
        }
    }

    /// `triggerEvent`: rung from the side `b1` names, which starts it shaking over.
    pub fn trigger_event(
        &mut self,
        level: &LevelTicking<'_>,
        pos: &BlockPos,
        b0: i32,
        b1: i32,
    ) -> bool {
        if b0 == 1 {
            self.update_entities(level, pos);
            self.resonation_ticks = 0;
            self.click_direction = Some(Direction::from_3d_data_value(b1));
            self.ticks = 0;
            self.shaking = true;
            true
        } else {
            false
        }
    }

    /// `clientTick`: `tick` with the client's end of a resonance.
    pub fn client_tick(level: &mut LevelTicking<'_>, pos: &BlockPos, _state: BlockState) {
        let mut block_entity = level.block_entity(pos);
        let BlockEntityData::Bell(entity) = &mut block_entity.data else {
            return;
        };
        if entity.shaking {
            entity.ticks += 1;
        }

        if entity.ticks >= BellBlockEntity::DURATION {
            entity.shaking = false;
            entity.ticks = 0;
        }

        // `areRaidersNearby`: the entities it found hold no raider (above).
        if entity.resonating {
            if entity.resonation_ticks < BellBlockEntity::MAX_RESONATION_TICKS {
                entity.resonation_ticks += 1;
            } else {
                entity.resonating = false;
            }
        }
    }

    /// `updateEntities`: the living entities within 48 blocks, looked for again once 60 ticks
    /// have passed since it last looked. The client tells none of them of the ringing.
    fn update_entities(&mut self, level: &LevelTicking<'_>, pos: &BlockPos) {
        let game_time = level.level.get_game_time();
        if game_time
            > self
                .last_ring_timestamp
                .wrapping_add(BellBlockEntity::MIN_TICKS_BETWEEN_SEARCHES)
            || self.nearby_entities.is_none()
        {
            self.last_ring_timestamp = game_time;
            let aabb = AABB::from_block_pos(pos).inflate(BellBlockEntity::SEARCH_RADIUS);
            self.nearby_entities = Some(
                level
                    .player_bounding_box()
                    .filter(|player| player.intersects(&aabb))
                    .map_or(0, |_| 1),
            );
        }
    }

    pub fn last_ring_timestamp(&self) -> i64 {
        self.last_ring_timestamp
    }

    pub fn resonating(&self) -> bool {
        self.resonating
    }

    pub fn resonation_ticks(&self) -> i32 {
        self.resonation_ticks
    }

    pub fn nearby_entities(&self) -> Option<usize> {
        self.nearby_entities
    }
}

impl Default for BellBlockEntity {
    fn default() -> BellBlockEntity {
        BellBlockEntity::new()
    }
}
