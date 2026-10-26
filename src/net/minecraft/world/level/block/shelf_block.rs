//! net/minecraft/world/level/block/ShelfBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `getFluidState`, `useShapeForLightOcclusion` and its shape (its back with a
//! top and a bottom shelf, toward where it faces) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::side_chain_part::SideChainPart;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct ShelfBlock;

pub static CLASS: BlockClass = BlockClass {
    update_shape: Some(ShelfBlock::update_shape),
    get_shape: Some(ShelfBlock::get_shape),
    use_shape_for_light_occlusion: Some(ShelfBlock::use_shape_for_light_occlusion),
    get_fluid_state: Some(ShelfBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.ShelfBlock",
        &base_entity_block::CLASS,
    )
};

impl ShelfBlock {
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::HORIZONTAL_FACING;
    pub const SIDE_CHAIN_PART: &'static EnumProperty<SideChainPart> =
        BlockStateProperties::SIDE_CHAIN_PART;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        ShelfBlock::new_with(
            properties,
            &CLASS,
            ShelfBlock::create_block_state_definition,
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
                .set_value(ShelfBlock::FACING, Direction::NORTH)
                .set_value(ShelfBlock::POWERED, false)
                .set_value(ShelfBlock::SIDE_CHAIN_PART, SideChainPart::UNCONNECTED)
                .set_value(ShelfBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            ShelfBlock::FACING,
            ShelfBlock::POWERED,
            ShelfBlock::SIDE_CHAIN_PART,
            ShelfBlock::WATERLOGGED,
        ]);
    }
}

impl ShelfBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

impl ShelfBlock {
    /// `useShapeForLightOcclusion`.
    fn use_shape_for_light_occlusion(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}

static SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> = LazyLock::new(|| {
    Shapes::rotate_horizontal(&Shapes::or_tail(
        &Block::r#box(0.0, 12.0, 11.0, 16.0, 16.0, 13.0),
        &[
            Block::r#box(0.0, 0.0, 13.0, 16.0, 16.0, 16.0),
            Block::r#box(0.0, 0.0, 11.0, 16.0, 4.0, 13.0),
        ],
    ))
});

impl ShelfBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(&SHAPES, state.get_value(ShelfBlock::FACING)).clone()
    }
}

impl ShelfBlock {
    /// `updateShape`: its parent's. The water's tick it schedules when it's waterlogged goes
    /// nowhere on the client.
    #[allow(clippy::too_many_arguments)]
    fn update_shape(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction_to_neighbour: Direction,
        neighbour_pos: &BlockPos,
        neighbour_state: BlockState,
        random: &mut dyn RandomSource,
    ) -> BlockState {
        Block::super_update_shape(
            block,
            &CLASS,
            state,
            level,
            pos,
            direction_to_neighbour,
            neighbour_pos,
            neighbour_state,
            random,
        )
    }
}
