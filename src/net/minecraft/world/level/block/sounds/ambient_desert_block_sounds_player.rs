//! net/minecraft/world/level/block/sounds/AmbientDesertBlockSoundsPlayer.java

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::{BlockPos, MutableBlockPos};
use crate::net::minecraft::core::direction::{Direction, Plane};
use crate::net::minecraft::sounds::sound_events::SoundEvents;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::levelgen::heightmap::Types;

/// The desert's ambient sounds: sand in the dunes, dry grass and dead bushes.
pub struct AmbientDesertBlockSoundsPlayer;

impl AmbientDesertBlockSoundsPlayer {
    const IDLE_SOUND_CHANCE: i32 = 2100;
    const DRY_GRASS_SOUND_CHANCE: i32 = 200;
    const DEAD_BUSH_SOUND_CHANCE: i32 = 130;
    const DEAD_BUSH_SOUND_BADLANDS_DECREASED_CHANCE: i32 = 3;
    const SURROUNDING_BLOCKS_PLAY_SOUND_THRESHOLD: i32 = 3;
    const SURROUNDING_BLOCKS_DISTANCE_HORIZONTAL_CHECK: i32 = 8;
    const SURROUNDING_BLOCKS_DISTANCE_VERTICAL_CHECK: i32 = 5;
    const HORIZONTAL_DIRECTIONS: i32 = 4;

    /// `playAmbientSandSounds`: rarely, under open air among more sand around it.
    pub fn play_ambient_sand_sounds(
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if level.get_block_state(&pos.above()).is(Blocks::AIR)
            && random.next_int_bounded(AmbientDesertBlockSoundsPlayer::IDLE_SOUND_CHANCE) == 0
            && AmbientDesertBlockSoundsPlayer::should_play_ambient_sand_sound(level, pos)
        {
            level.play_local_sound(
                f64::from(pos.get_x()),
                f64::from(pos.get_y()),
                f64::from(pos.get_z()),
                SoundEvents::SAND_IDLE,
                SoundSource::AMBIENT,
                1.0,
                1.0,
                false,
            );
        }
    }

    /// `playAmbientDryGrassSounds`: now and then, on two blocks that make it, the player's.
    pub fn play_ambient_dry_grass_sounds(
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if random.next_int_bounded(AmbientDesertBlockSoundsPlayer::DRY_GRASS_SOUND_CHANCE) == 0
            && AmbientDesertBlockSoundsPlayer::should_play_desert_dry_vegetation_block_sounds(
                level,
                &pos.below(),
            )
        {
            level.play_player_sound(SoundEvents::DRY_GRASS, SoundSource::AMBIENT, 1.0, 1.0);
        }
    }

    /// `playAmbientDeadBushSounds`: now and then, on two blocks that make it, less often on red
    /// sand or terracotta.
    pub fn play_ambient_dead_bush_sounds(
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if random.next_int_bounded(AmbientDesertBlockSoundsPlayer::DEAD_BUSH_SOUND_CHANCE) == 0 {
            let below_pos = level.get_block_state(&pos.below());
            if (below_pos.is(Blocks::RED_SAND) || below_pos.is_tag(BlockTags::TERRACOTTA))
                && random.next_int_bounded(
                    AmbientDesertBlockSoundsPlayer::DEAD_BUSH_SOUND_BADLANDS_DECREASED_CHANCE,
                ) != 0
            {
                return;
            }

            if AmbientDesertBlockSoundsPlayer::should_play_desert_dry_vegetation_block_sounds(
                level,
                &pos.below(),
            ) {
                level.play_local_sound(
                    f64::from(pos.get_x()),
                    f64::from(pos.get_y()),
                    f64::from(pos.get_z()),
                    SoundEvents::DEAD_BUSH_IDLE,
                    SoundSource::AMBIENT,
                    1.0,
                    1.0,
                    false,
                );
            }
        }
    }

    /// `shouldPlayDesertDryVegetationBlockSounds`: the block and the one under it make it.
    pub fn should_play_desert_dry_vegetation_block_sounds(
        level: &LevelWithEffects<'_>,
        below_pos: &BlockPos,
    ) -> bool {
        level
            .get_block_state(below_pos)
            .is_tag(BlockTags::TRIGGERS_AMBIENT_DESERT_DRY_VEGETATION_BLOCK_SOUNDS)
            && level
                .get_block_state(&below_pos.below())
                .is_tag(BlockTags::TRIGGERS_AMBIENT_DESERT_DRY_VEGETATION_BLOCK_SOUNDS)
    }

    /// `shouldPlayAmbientSandSound`: sand open to the air in three of the columns 8 blocks away.
    fn should_play_ambient_sand_sound(level: &LevelWithEffects<'_>, pos: &BlockPos) -> bool {
        let mut matching_blocks_found = 0;
        let mut sides_checked = 0;
        let mut mutable_pos = pos.mutable();

        for dir in Plane::HORIZONTAL.iterator() {
            mutable_pos.set(pos).move_steps(
                dir,
                AmbientDesertBlockSoundsPlayer::SURROUNDING_BLOCKS_DISTANCE_HORIZONTAL_CHECK,
            );
            if AmbientDesertBlockSoundsPlayer::column_contains_triggering_block(
                level,
                &mut mutable_pos,
            ) {
                let found = matching_blocks_found;
                matching_blocks_found += 1;
                if found >= AmbientDesertBlockSoundsPlayer::SURROUNDING_BLOCKS_PLAY_SOUND_THRESHOLD
                {
                    return true;
                }
            }

            sides_checked += 1;
            let remaining_sides =
                AmbientDesertBlockSoundsPlayer::HORIZONTAL_DIRECTIONS - sides_checked;
            let potential_matches = remaining_sides + matching_blocks_found;
            let can_still_find_required_sound_trigger_blocks = potential_matches
                >= AmbientDesertBlockSoundsPlayer::SURROUNDING_BLOCKS_PLAY_SOUND_THRESHOLD;
            if !can_still_find_required_sound_trigger_blocks {
                return false;
            }
        }

        false
    }

    fn column_contains_triggering_block(
        level: &LevelWithEffects<'_>,
        mutable_pos: &mut MutableBlockPos,
    ) -> bool {
        let surface_y = level.get_height_at(
            Types::WORLD_SURFACE,
            mutable_pos.get_x(),
            mutable_pos.get_z(),
        ) - 1;
        if (surface_y - mutable_pos.get_y()).abs()
            > AmbientDesertBlockSoundsPlayer::SURROUNDING_BLOCKS_DISTANCE_VERTICAL_CHECK
        {
            mutable_pos.move_steps(Direction::UP, 6);
            let mut above_block_state = level.get_block_state(mutable_pos);
            mutable_pos.r#move(Direction::DOWN);

            for _ in 0..10 {
                let current_block_state = level.get_block_state(mutable_pos);
                if above_block_state.is_air()
                    && AmbientDesertBlockSoundsPlayer::can_trigger_ambient_desert_sand_sounds(
                        current_block_state,
                    )
                {
                    return true;
                }

                above_block_state = current_block_state;
                mutable_pos.r#move(Direction::DOWN);
            }

            false
        } else {
            let has_air_above = level
                .get_block_state(mutable_pos.set_y(surface_y + 1))
                .is_air();
            has_air_above
                && AmbientDesertBlockSoundsPlayer::can_trigger_ambient_desert_sand_sounds(
                    level.get_block_state(mutable_pos.set_y(surface_y)),
                )
        }
    }

    fn can_trigger_ambient_desert_sand_sounds(block_state: BlockState) -> bool {
        block_state.is_tag(BlockTags::TRIGGERS_AMBIENT_DESERT_SAND_BLOCK_SOUNDS)
    }
}
