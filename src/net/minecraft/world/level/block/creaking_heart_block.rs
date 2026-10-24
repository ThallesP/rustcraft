//! net/minecraft/world/level/block/CreakingHeartBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, its sound (`animateTick`) is ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::sounds::sound_events::SoundEvents;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::creaking_heart_state::CreakingHeartState;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct CreakingHeartBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(CreakingHeartBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.CreakingHeartBlock",
        &base_entity_block::CLASS,
    )
};

impl CreakingHeartBlock {
    pub const AXIS: &'static EnumProperty<Axis> = BlockStateProperties::AXIS;
    pub const STATE: &'static EnumProperty<CreakingHeartState> =
        BlockStateProperties::CREAKING_HEART_STATE;
    pub const NATURAL: &'static BooleanProperty = BlockStateProperties::NATURAL;

    pub fn new(properties: Properties) -> BlockData {
        CreakingHeartBlock::new_with(
            properties,
            &CLASS,
            CreakingHeartBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseEntityBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .default_block_state()
                .set_value(CreakingHeartBlock::AXIS, Axis::Y)
                .set_value(CreakingHeartBlock::STATE, CreakingHeartState::UPROOTED)
                .set_value(CreakingHeartBlock::NATURAL, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            CreakingHeartBlock::AXIS,
            CreakingHeartBlock::STATE,
            CreakingHeartBlock::NATURAL,
        ]);
    }
}

impl CreakingHeartBlock {
    /// `animateTick`: while creakings are about, its sound now and then, when it's in pale oak logs.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if level.get_environment_value(&EnvironmentAttributes::CREAKING_ACTIVE, pos)
            && state.get_value(CreakingHeartBlock::STATE) != CreakingHeartState::UPROOTED
            && random.next_int_bounded(16) == 0
            && CreakingHeartBlock::is_surrounded_by_logs(level, pos)
        {
            level.play_local_sound(
                f64::from(pos.get_x()),
                f64::from(pos.get_y()),
                f64::from(pos.get_z()),
                SoundEvents::CREAKING_HEART_IDLE,
                SoundSource::BLOCKS,
                1.0,
                1.0,
                false,
            );
        }
    }

    fn is_surrounded_by_logs(level: &LevelWithEffects<'_>, pos: &BlockPos) -> bool {
        Direction::values().into_iter().all(|dir| {
            level
                .get_block_state(&pos.relative(dir))
                .is_tag(BlockTags::PALE_OAK_LOGS)
        })
    }
}
