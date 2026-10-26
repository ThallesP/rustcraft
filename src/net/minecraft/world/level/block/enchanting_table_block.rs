//! net/minecraft/world/level/block/EnchantingTableBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, `useShapeForLightOcclusion` and its shape, and its letters (`animateTick`) are
//! ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::core::vec3i::Vec3i;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::entity::block_entity_ticker::BlockEntityTicker;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::entity::block_entity_types::BlockEntityTypes;
use crate::net::minecraft::world::level::block::entity::enchanting_table_block_entity::EnchantingTableBlockEntity;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct EnchantingTableBlock;

pub static CLASS: BlockClass = BlockClass {
    get_ticker: Some(EnchantingTableBlock::get_ticker),
    animate_tick: Some(EnchantingTableBlock::animate_tick),
    get_shape: Some(EnchantingTableBlock::get_shape),
    use_shape_for_light_occlusion: Some(EnchantingTableBlock::use_shape_for_light_occlusion),
    ..BlockClass::new(
        "net.minecraft.world.level.block.EnchantingTableBlock",
        &base_entity_block::CLASS,
    )
};

impl EnchantingTableBlock {
    pub fn new(properties: Properties) -> BlockData {
        EnchantingTableBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        BaseEntityBlock::new_with(properties, class, definition)
    }
}

impl EnchantingTableBlock {
    /// `useShapeForLightOcclusion`.
    fn use_shape_for_light_occlusion(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 12.0));

impl EnchantingTableBlock {
    fn get_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE.clone()
    }
}

impl EnchantingTableBlock {
    /// `animateTick`: its parent's (nothing), and now and then letters flying from each bookshelf around it that nothing blocks.
    fn animate_tick(
        block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        Block::super_animate_tick(block, &CLASS, state, level, pos, random);

        for offset in BOOKSHELF_OFFSETS.iter() {
            if random.next_int_bounded(16) == 0
                && EnchantingTableBlock::is_valid_book_shelf(level, pos, offset)
            {
                let xd = f64::from(offset.get_x() as f32 + random.next_float()) - 0.5;
                let yd = f64::from(offset.get_y() as f32 - random.next_float() - 1.0);
                let zd = f64::from(offset.get_z() as f32 + random.next_float()) - 0.5;
                level.add_particle(
                    ParticleTypes::ENCHANT,
                    f64::from(pos.get_x()) + 0.5,
                    f64::from(pos.get_y()) + 2.0,
                    f64::from(pos.get_z()) + 0.5,
                    xd,
                    yd,
                    zd,
                );
            }
        }
    }

    /// `isValidBookShelf`: a bookshelf there, with nothing in the way.
    pub fn is_valid_book_shelf(
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        offset: &BlockPos,
    ) -> bool {
        level
            .get_block_state(&pos.offset(offset))
            .is_tag(BlockTags::ENCHANTMENT_POWER_PROVIDER)
            && level
                .get_block_state(&pos.offset(&Vec3i::new(
                    offset.get_x() / 2,
                    offset.get_y(),
                    offset.get_z() / 2,
                )))
                .is_tag(BlockTags::ENCHANTMENT_POWER_TRANSMITTER)
    }
}

/// `BOOKSHELF_OFFSETS`: the blocks two from the table, at its height and one up, in
/// `betweenClosedStream`'s order (x fastest, then y, then z).
pub static BOOKSHELF_OFFSETS: LazyLock<Vec<BlockPos>> = LazyLock::new(|| {
    let mut offsets = Vec::new();
    for z in -2_i32..=2 {
        for y in 0..=1 {
            for x in -2_i32..=2 {
                if x.abs() == 2 || z.abs() == 2 {
                    offsets.push(BlockPos::new(x, y, z));
                }
            }
        }
    }
    offsets
});

impl EnchantingTableBlock {}

impl EnchantingTableBlock {
    /// `getTicker`: on the client, `bookAnimationTick`.
    fn get_ticker(
        _block: &BlockData,
        _state: BlockState,
        r#type: BlockEntityType,
    ) -> Option<BlockEntityTicker> {
        base_entity_block::create_ticker_helper(
            r#type,
            BlockEntityTypes::ENCHANTING_TABLE,
            EnchantingTableBlockEntity::book_animation_tick,
        )
    }
}
