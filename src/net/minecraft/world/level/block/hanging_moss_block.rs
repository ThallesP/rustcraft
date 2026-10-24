//! net/minecraft/world/level/block/HangingMossBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, `propagatesSkylightDown` and its shape (shorter at the tip), and its sound
//! (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::sounds::sound_events::SoundEvents;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct HangingMossBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(HangingMossBlock::animate_tick),
    get_shape: Some(HangingMossBlock::get_shape),
    propagates_skylight_down: Some(HangingMossBlock::propagates_skylight_down),
    ..BlockClass::new(
        "net.minecraft.world.level.block.HangingMossBlock",
        &block::CLASS,
    )
};

impl HangingMossBlock {
    pub const TIP: &'static BooleanProperty = BlockStateProperties::TIP;

    pub fn new(properties: Properties) -> BlockData {
        HangingMossBlock::new_with(
            properties,
            &CLASS,
            HangingMossBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(HangingMossBlock::TIP, true),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[HangingMossBlock::TIP]);
    }
}

impl HangingMossBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}

static SHAPE_BASE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(14.0, 0.0, 16.0));
static SHAPE_TIP: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(14.0, 2.0, 16.0));

impl HangingMossBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        if state.get_value(HangingMossBlock::TIP) {
            SHAPE_TIP.clone()
        } else {
            SHAPE_BASE.clone()
        }
    }
}

impl HangingMossBlock {
    /// `animateTick`: its sound now and then, under a pale oak.
    fn animate_tick(
        _block: &BlockData,
        _state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if random.next_int_bounded(500) == 0 {
            let above = level.get_block_state(&pos.above());
            if above.is_tag(BlockTags::PALE_OAK_LOGS) || above.is(Blocks::PALE_OAK_LEAVES) {
                level.play_local_sound(
                    f64::from(pos.get_x()),
                    f64::from(pos.get_y()),
                    f64::from(pos.get_z()),
                    SoundEvents::PALE_HANGING_MOSS_IDLE,
                    SoundSource::AMBIENT,
                    1.0,
                    1.0,
                    false,
                );
            }
        }
    }
}
