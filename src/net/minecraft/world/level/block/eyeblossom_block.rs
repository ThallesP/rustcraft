//! net/minecraft/world/level/block/EyeblossomBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, its sound (`animateTick`) is ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::sounds::sound_events::SoundEvents;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{Block, BlockClass, BlockData};
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::flower_block::{self, FlowerBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct EyeblossomBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(EyeblossomBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.EyeblossomBlock",
        &flower_block::CLASS,
    )
};

impl EyeblossomBlock {
    /// `EyeblossomBlock(type, properties)`: open or closed, the block's field.
    pub fn new(r#type: EyeblossomType, properties: Properties) -> BlockData {
        let mut block =
            FlowerBlock::new_with(properties, &CLASS, Block::create_block_state_definition);
        block.set_fields(r#type);
        block
    }
}

impl EyeblossomBlock {
    /// `animateTick`: an open one's sound now and then, on pale moss.
    fn animate_tick(
        block: &BlockData,
        _state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if block.fields::<EyeblossomType>().emit_sounds() && random.next_int_bounded(700) == 0 {
            let below = level.get_block_state(&pos.below());
            if below.is(Blocks::PALE_MOSS_BLOCK) {
                level.play_local_sound(
                    f64::from(pos.get_x()),
                    f64::from(pos.get_y()),
                    f64::from(pos.get_z()),
                    SoundEvents::EYEBLOSSOM_IDLE,
                    SoundSource::AMBIENT,
                    1.0,
                    1.0,
                    false,
                );
            }
        }
    }
}

/// `EyeblossomBlock.Type`: open or closed. Not ported yet: what else each holds (its effect, the
/// particle's colour, the sounds of turning), which its ticks read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EyeblossomType {
    OPEN,
    CLOSED,
}

impl EyeblossomType {
    /// `emitSounds()`: an open one does.
    pub fn emit_sounds(self) -> bool {
        self == EyeblossomType::OPEN
    }
}
