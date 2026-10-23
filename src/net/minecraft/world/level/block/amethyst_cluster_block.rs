//! net/minecraft/world/level/block/AmethystClusterBlock.java
//!
//! Only what decides its blocks' states so far: its properties, its default state, and its
//! constructor's `height` and `width`, which make its shapes. Not ported yet: its behaviour.
//!
//! Of its behaviour, `getFluidState` and its shape (a cluster as tall and wide as its constructor
//! says, growing from the face it's on) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::amethyst_block::{self, AmethystBlock};
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct AmethystClusterBlock {
    height: f32,
    width: f32,
    /// `shapes`, by the face it grows from.
    shapes: Vec<(Direction, VoxelShape)>,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(AmethystClusterBlock::get_shape),
    get_fluid_state: Some(AmethystClusterBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.AmethystClusterBlock",
        &amethyst_block::CLASS,
    )
};

impl AmethystClusterBlock {
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::FACING;

    pub fn new(height: f32, width: f32, properties: Properties) -> BlockData {
        AmethystClusterBlock::new_with(
            height,
            width,
            properties,
            &CLASS,
            AmethystClusterBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        height: f32,
        width: f32,
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = AmethystBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .default_block_state()
                .set_value(AmethystClusterBlock::WATERLOGGED, false)
                .set_value(AmethystClusterBlock::FACING, Direction::UP),
        );
        block.set_fields(AmethystClusterBlock {
            height,
            width,
            shapes: Shapes::rotate_all(&Block::box_z_square(
                f64::from(width),
                f64::from(16.0 - height),
                16.0,
            )),
        });
        block
    }

    pub fn height(block: &BlockData) -> f32 {
        block.fields::<AmethystClusterBlock>().height
    }

    pub fn width(block: &BlockData) -> f32 {
        block.fields::<AmethystClusterBlock>().width
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            AmethystClusterBlock::WATERLOGGED,
            AmethystClusterBlock::FACING,
        ]);
    }
}

impl AmethystClusterBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

impl AmethystClusterBlock {
    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(
            &block.fields::<AmethystClusterBlock>().shapes,
            state.get_value(AmethystClusterBlock::FACING),
        )
        .clone()
    }
}
