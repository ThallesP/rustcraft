//! net/minecraft/world/level/block/BigDripleafBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `getFluidState` and its shapes (its leaf by its tilt, on its stem toward its
//! back) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::{
    self, HorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::tilt::Tilt;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct BigDripleafBlock {
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(BigDripleafBlock::get_shape),
    get_collision_shape: Some(BigDripleafBlock::get_collision_shape),
    get_fluid_state: Some(BigDripleafBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BigDripleafBlock",
        &horizontal_directional_block::CLASS,
    )
};

impl BigDripleafBlock {
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;
    pub const TILT: &'static EnumProperty<Tilt> = BlockStateProperties::TILT;

    pub fn new(properties: Properties) -> BlockData {
        BigDripleafBlock::new_with(
            properties,
            &CLASS,
            BigDripleafBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = HorizontalDirectionalBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(BigDripleafBlock::WATERLOGGED, false)
                .set_value(HorizontalDirectionalBlock::FACING, Direction::NORTH)
                .set_value(BigDripleafBlock::TILT, Tilt::NONE),
        );
        let shapes = BigDripleafBlock::make_shapes(&block);
        block.set_fields(BigDripleafBlock { shapes });
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            BigDripleafBlock::WATERLOGGED,
            HorizontalDirectionalBlock::FACING,
            BigDripleafBlock::TILT,
        ]);
    }
}

impl BigDripleafBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

/// `SHAPE_LEAF`, by the leaf's tilt.
fn shape_leaf(tilt: Tilt) -> VoxelShape {
    static LEAF: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 11.0, 15.0));
    static PARTIAL: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 11.0, 13.0));
    match tilt {
        Tilt::NONE | Tilt::UNSTABLE => LEAF.clone(),
        Tilt::PARTIAL => PARTIAL.clone(),
        Tilt::FULL => Shapes::empty(),
    }
}

impl BigDripleafBlock {
    /// `makeShapes`: the leaf with the stem a quarter block toward the back, for each state but
    /// for its water.
    fn make_shapes(block: &BlockData) -> Vec<VoxelShape> {
        let stems: Vec<(Direction, VoxelShape)> = Shapes::rotate_horizontal(
            &Block::column(6.0, 0.0, 13.0)
                .move_xyz(0.0, 0.0, 0.25)
                .optimize(),
        );
        Block::get_shape_for_each_state_ignoring(
            block,
            |state| {
                Shapes::or(
                    &shape_leaf(state.get_value(BigDripleafBlock::TILT)),
                    Shapes::get(&stems, state.get_value(HorizontalDirectionalBlock::FACING)),
                )
            },
            &[BigDripleafBlock::WATERLOGGED],
        )
    }

    fn get_collision_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        shape_leaf(state.get_value(BigDripleafBlock::TILT))
    }

    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<BigDripleafBlock>().shapes[state.index() as usize].clone()
    }
}
