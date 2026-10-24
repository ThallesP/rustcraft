//! net/minecraft/world/level/block/sounds/AmbientLeavesBlockSoundPlayer.java
//!
//! Not ported yet: its `CODEC`, which no packet the client reads carries.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Plane;
use crate::net::minecraft::sounds::sound_event::SoundEvent;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::tags::block_tags::TagKey;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::BlockData;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

/// A leaves block's ambient sound: now and then, with enough of the blocks that make it, and
/// enough of the same leaves, beside it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmbientLeavesBlockSoundPlayer {
    ambient_sound: Option<SoundEvent>,
    chance: i32,
    satisfying_blocks: Option<TagKey>,
    nearby_satisfying_blocks_required: i32,
    nearby_same_leaves_required: i32,
}

impl AmbientLeavesBlockSoundPlayer {
    const AMBIENT_SOUND_CHANCE: i32 = 300;
    const NEARBY_SATISFYING_BLOCKS_REQUIRED: i32 = 1;
    const NEARBY_SAME_LEAVES_REQUIRED: i32 = 3;
    const NO_SOUND: AmbientLeavesBlockSoundPlayer = AmbientLeavesBlockSoundPlayer {
        ambient_sound: None,
        chance: 0,
        satisfying_blocks: None,
        nearby_satisfying_blocks_required: 0,
        nearby_same_leaves_required: 0,
    };

    /// `of(ambientSound, satisfyingBlocks)`: one in 300, with one of the tag's blocks and three
    /// of the same leaves.
    pub const fn of(
        ambient_sound: SoundEvent,
        satisfying_blocks: TagKey,
    ) -> AmbientLeavesBlockSoundPlayer {
        AmbientLeavesBlockSoundPlayer {
            ambient_sound: Some(ambient_sound),
            chance: AmbientLeavesBlockSoundPlayer::AMBIENT_SOUND_CHANCE,
            satisfying_blocks: Some(satisfying_blocks),
            nearby_satisfying_blocks_required:
                AmbientLeavesBlockSoundPlayer::NEARBY_SATISFYING_BLOCKS_REQUIRED,
            nearby_same_leaves_required: AmbientLeavesBlockSoundPlayer::NEARBY_SAME_LEAVES_REQUIRED,
        }
    }

    pub const fn no_ambient_sound() -> AmbientLeavesBlockSoundPlayer {
        AmbientLeavesBlockSoundPlayer::NO_SOUND
    }

    /// `playAmbientLeavesSounds`: the sound as soon as the neighbours counted so far (in the
    /// horizontal directions' order) are exactly what it needs.
    pub fn play_ambient_leaves_sounds(
        &self,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        block: &BlockData,
        random: &mut dyn RandomSource,
    ) {
        let Some(ambient_sound) = self.ambient_sound else {
            return;
        };
        if random.next_int_bounded(self.chance) == 0 {
            let mut nearby_logs = 0;
            let mut nearby_leaves = 0;

            for direction in Plane::HORIZONTAL.iterator() {
                let neighbor_pos = pos.relative(direction);
                let neighbor_state = level.get_block_state(&neighbor_pos);
                if let Some(satisfying_blocks) = self.satisfying_blocks
                    && neighbor_state.is_tag(satisfying_blocks)
                {
                    nearby_logs += 1;
                }

                if std::ptr::eq(&*neighbor_state.get_block(), block) {
                    nearby_leaves += 1;
                }

                if nearby_logs == self.nearby_satisfying_blocks_required
                    && nearby_leaves == self.nearby_same_leaves_required
                {
                    level.play_local_sound(
                        f64::from(pos.get_x()),
                        f64::from(pos.get_y()),
                        f64::from(pos.get_z()),
                        ambient_sound,
                        SoundSource::AMBIENT,
                        1.0,
                        1.0,
                        false,
                    );
                    return;
                }
            }
        }
    }
}
