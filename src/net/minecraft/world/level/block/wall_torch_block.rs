//! net/minecraft/world/level/block/WallTorchBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, its shape (leaning from the wall it faces from), and its smoke and flame
//! (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::particle_type::ParticleType;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::torch_block::{self, TorchBlock};
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct WallTorchBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(WallTorchBlock::animate_tick),
    get_shape: Some(WallTorchBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WallTorchBlock",
        &torch_block::CLASS,
    )
};

impl WallTorchBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;

    pub fn new(flame_particle: ParticleType, properties: Properties) -> BlockData {
        WallTorchBlock::new_with(
            properties,
            &CLASS,
            WallTorchBlock::create_block_state_definition,
            flame_particle,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
        flame_particle: ParticleType,
    ) -> BlockData {
        let mut block = TorchBlock::new_with(properties, class, definition, flame_particle);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(WallTorchBlock::FACING, Direction::NORTH),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[WallTorchBlock::FACING]);
    }
}

static SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal(&Block::box_z(5.0, 3.0, 13.0, 11.0, 16.0)));

impl WallTorchBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        WallTorchBlock::get_shape_of(state)
    }

    /// The static `getShape(BlockState)`, which redstone wall torches answer too.
    pub fn get_shape_of(state: BlockState) -> VoxelShape {
        Shapes::get(&SHAPES, state.get_value(WallTorchBlock::FACING)).clone()
    }
}

impl WallTorchBlock {
    /// `animateTick`: smoke and its flame (`flameParticle`) at its tip, off the wall.
    fn animate_tick(
        block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        _random: &mut dyn RandomSource,
    ) {
        let direction = state.get_value(WallTorchBlock::FACING);
        let x = f64::from(pos.get_x()) + 0.5;
        let y = f64::from(pos.get_y()) + 0.7;
        let z = f64::from(pos.get_z()) + 0.5;
        let h = 0.22;
        let r = 0.27;
        let opposite = direction.get_opposite();
        let px = x + r * f64::from(opposite.get_step_x());
        let pz = z + r * f64::from(opposite.get_step_z());
        level.add_particle(ParticleTypes::SMOKE, px, y + h, pz, 0.0, 0.0, 0.0);
        level.add_particle(
            *block.fields::<ParticleType>(),
            px,
            y + h,
            pz,
            0.0,
            0.0,
            0.0,
        );
    }
}
