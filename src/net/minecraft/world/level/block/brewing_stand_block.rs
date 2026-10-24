//! net/minecraft/world/level/block/BrewingStandBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, its shape (a rod on a base), and its smoke (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct BrewingStandBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(BrewingStandBlock::animate_tick),
    get_shape: Some(BrewingStandBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BrewingStandBlock",
        &base_entity_block::CLASS,
    )
};

impl BrewingStandBlock {
    pub const HAS_BOTTLE: [&'static BooleanProperty; 3] = [
        BlockStateProperties::HAS_BOTTLE_0,
        BlockStateProperties::HAS_BOTTLE_1,
        BlockStateProperties::HAS_BOTTLE_2,
    ];

    pub fn new(properties: Properties) -> BlockData {
        BrewingStandBlock::new_with(
            properties,
            &CLASS,
            BrewingStandBlock::create_block_state_definition,
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
                .get_state_definition()
                .any()
                .set_value(BrewingStandBlock::HAS_BOTTLE[0], false)
                .set_value(BrewingStandBlock::HAS_BOTTLE[1], false)
                .set_value(BrewingStandBlock::HAS_BOTTLE[2], false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            BrewingStandBlock::HAS_BOTTLE[0],
            BrewingStandBlock::HAS_BOTTLE[1],
            BrewingStandBlock::HAS_BOTTLE[2],
        ]);
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| {
    Shapes::or(
        &Block::column(2.0, 2.0, 14.0),
        &Block::column(14.0, 0.0, 2.0),
    )
});

impl BrewingStandBlock {
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

impl BrewingStandBlock {
    /// `animateTick`: smoke above its rod.
    fn animate_tick(
        _block: &BlockData,
        _state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        let x = f64::from(pos.get_x()) + 0.4 + f64::from(random.next_float()) * 0.2;
        let y = f64::from(pos.get_y()) + 0.7 + f64::from(random.next_float()) * 0.3;
        let z = f64::from(pos.get_z()) + 0.4 + f64::from(random.next_float()) * 0.2;
        level.add_particle(ParticleTypes::SMOKE, x, y, z, 0.0, 0.0, 0.0);
    }
}
