//! net/minecraft/world/level/block/entity/EnchantingTableBlockEntity.java
//!
//! An enchanting table's book, as the client's ticker turns it towards the nearest player within
//! three blocks, opens it and flips its pages (`bookAnimationTick`), from the class's own random
//! (`RANDOM`, a static made once with a unique seed: a thread's here, the game thread's). Not
//! ported yet: its name.

use std::cell::RefCell;

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntityData;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::levelgen::legacy_random_source::LegacyRandomSource;
use crate::net::minecraft::world::level::levelgen::random_support::RandomSupport;

thread_local! {
    /// `RANDOM`: `RandomSource.create()`.
    static RANDOM: RefCell<LegacyRandomSource> =
        RefCell::new(LegacyRandomSource::new(RandomSupport::generate_unique_seed()));
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct EnchantingTableBlockEntity {
    pub time: i32,
    pub flip: f32,
    pub o_flip: f32,
    pub flip_t: f32,
    pub flip_a: f32,
    pub open: f32,
    pub o_open: f32,
    pub rot: f32,
    pub o_rot: f32,
    pub t_rot: f32,
}

impl EnchantingTableBlockEntity {
    const PI: f32 = std::f64::consts::PI as f32;
    const TWO_PI: f32 = (std::f64::consts::PI * 2.0) as f32;

    pub fn new() -> EnchantingTableBlockEntity {
        EnchantingTableBlockEntity::default()
    }

    /// `bookAnimationTick`, the client's ticker.
    pub fn book_animation_tick(
        level: &mut LevelTicking<'_>,
        world_position: &BlockPos,
        _state: BlockState,
    ) {
        let x = f64::from(world_position.get_x()) + 0.5;
        let y = f64::from(world_position.get_y()) + 0.5;
        let z = f64::from(world_position.get_z()) + 0.5;
        let player = level.get_nearest_player(x, y, z, 3.0);
        let mut block_entity = level.block_entity(world_position);
        let BlockEntityData::EnchantingTable(entity) = &mut block_entity.data else {
            return;
        };
        entity.o_open = entity.open;
        entity.o_rot = entity.rot;
        match player {
            Some(player) => {
                let xd = player.x - x;
                let zd = player.z - z;
                entity.t_rot = Mth::atan2(zd, xd) as f32;
                entity.open += 0.1;
                RANDOM.with_borrow_mut(|random| {
                    if entity.open < 0.5 || random.next_int_bounded(40) == 0 {
                        let old = entity.flip_t;
                        loop {
                            entity.flip_t +=
                                (random.next_int_bounded(4) - random.next_int_bounded(4)) as f32;
                            if old != entity.flip_t {
                                break;
                            }
                        }
                    }
                });
            }
            None => {
                entity.t_rot += 0.02;
                entity.open -= 0.1;
            }
        }

        while entity.rot >= EnchantingTableBlockEntity::PI {
            entity.rot -= EnchantingTableBlockEntity::TWO_PI;
        }

        while entity.rot < -EnchantingTableBlockEntity::PI {
            entity.rot += EnchantingTableBlockEntity::TWO_PI;
        }

        while entity.t_rot >= EnchantingTableBlockEntity::PI {
            entity.t_rot -= EnchantingTableBlockEntity::TWO_PI;
        }

        while entity.t_rot < -EnchantingTableBlockEntity::PI {
            entity.t_rot += EnchantingTableBlockEntity::TWO_PI;
        }

        let mut rot_dir = entity.t_rot - entity.rot;

        while rot_dir >= EnchantingTableBlockEntity::PI {
            rot_dir -= EnchantingTableBlockEntity::TWO_PI;
        }

        while rot_dir < -EnchantingTableBlockEntity::PI {
            rot_dir += EnchantingTableBlockEntity::TWO_PI;
        }

        entity.rot += rot_dir * 0.4;
        entity.open = Mth::clamp_f32(entity.open, 0.0, 1.0);
        entity.time = entity.time.wrapping_add(1);
        entity.o_flip = entity.flip;
        let diff = (entity.flip_t - entity.flip) * 0.4;
        let diff = Mth::clamp_f32(diff, -0.2, 0.2);
        entity.flip_a += (diff - entity.flip_a) * 0.9;
        entity.flip += entity.flip_a;
    }

    /// `RANDOM` seeded, for the golden programs, whose vanilla side seeds it too.
    #[cfg(test)]
    pub(crate) fn seed_random(seed: i64) {
        RANDOM.with_borrow_mut(|random| random.set_seed(seed));
    }

    /// `RANDOM`'s next long, which the golden programs end with.
    #[cfg(test)]
    pub(crate) fn next_random_long() -> i64 {
        RANDOM.with_borrow_mut(|random| random.next_long())
    }
}
