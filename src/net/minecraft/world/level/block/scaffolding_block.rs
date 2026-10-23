//! net/minecraft/world/level/block/ScaffoldingBlock.java
//!
//! What decides its blocks' states (its properties and its default state) and its shapes: a top
//! and four legs (and a frame at the bottom of one that hangs), which hold up what's above it
//! unless it's descending, and a bottom one holds up what's on its frame. Not ported yet: the rest
//! of its behaviour.

use std::sync::LazyLock;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::references::block_item_ids::BlockItemIds;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

/// The union of a shape's four horizontal rotations, in `EnumMap` order
/// (`rotateHorizontal(…).values().stream().reduce(Shapes.empty(), Shapes::or)`).
fn horizontal_union(north: &VoxelShape) -> VoxelShape {
    Shapes::rotate_horizontal(north)
        .iter()
        .fold(Shapes::empty(), |shape, (_, rotated)| {
            Shapes::or(&shape, rotated)
        })
}

static SHAPE_STABLE: LazyLock<VoxelShape> = LazyLock::new(|| {
    Shapes::or(
        &Block::column(16.0, 14.0, 16.0),
        &horizontal_union(&Block::r#box(0.0, 0.0, 0.0, 2.0, 16.0, 2.0)),
    )
});
static SHAPE_UNSTABLE_BOTTOM: LazyLock<VoxelShape> =
    LazyLock::new(|| Block::column(16.0, 0.0, 2.0));
static SHAPE_UNSTABLE: LazyLock<VoxelShape> = LazyLock::new(|| {
    Shapes::or_tail(
        &SHAPE_STABLE,
        &[
            SHAPE_UNSTABLE_BOTTOM.clone(),
            horizontal_union(&Block::box_z(16.0, 0.0, 2.0, 0.0, 2.0)),
        ],
    )
});
static SHAPE_BELOW_BLOCK: LazyLock<VoxelShape> =
    LazyLock::new(|| Shapes::block().move_xyz(0.0, -1.0, 0.0).optimize());

pub struct ScaffoldingBlock;

pub static CLASS: BlockClass = BlockClass {
    get_interaction_shape: Some(ScaffoldingBlock::get_interaction_shape),
    get_fluid_state: Some(ScaffoldingBlock::get_fluid_state),
    get_shape: Some(ScaffoldingBlock::get_shape),
    get_collision_shape: Some(ScaffoldingBlock::get_collision_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.ScaffoldingBlock",
        &block::CLASS,
    )
};

impl ScaffoldingBlock {
    pub const DISTANCE: &'static IntegerProperty = BlockStateProperties::STABILITY_DISTANCE;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;
    pub const BOTTOM: &'static BooleanProperty = BlockStateProperties::BOTTOM;

    pub fn new(properties: Properties) -> BlockData {
        ScaffoldingBlock::new_with(
            properties,
            &CLASS,
            ScaffoldingBlock::create_block_state_definition,
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
                .set_value(ScaffoldingBlock::DISTANCE, 7)
                .set_value(ScaffoldingBlock::WATERLOGGED, false)
                .set_value(ScaffoldingBlock::BOTTOM, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            ScaffoldingBlock::DISTANCE,
            ScaffoldingBlock::WATERLOGGED,
            ScaffoldingBlock::BOTTOM,
        ]);
    }
}

impl ScaffoldingBlock {
    /// `getShape`: the whole block to one holding scaffolding, else its frame.
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        context: &dyn CollisionContext,
    ) -> VoxelShape {
        // `state.getBlock().asItem()`: scaffolding's item.
        if !context.is_holding_item(BlockItemIds::SCAFFOLDING.item()) {
            if state.get_value(ScaffoldingBlock::BOTTOM) {
                SHAPE_UNSTABLE.clone()
            } else {
                SHAPE_STABLE.clone()
            }
        } else {
            Shapes::block()
        }
    }

    /// `getCollisionShape`: its top and legs under what's above it and not descending, its bottom
    /// frame under what's above that, else nothing.
    fn get_collision_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        pos: &BlockPos,
        context: &dyn CollisionContext,
    ) -> VoxelShape {
        if context.is_placement() {
            Shapes::empty()
        } else if context.is_above(&Shapes::block(), pos, true) && !context.is_descending() {
            SHAPE_STABLE.clone()
        } else if state.get_value(ScaffoldingBlock::DISTANCE) != 0
            && state.get_value(ScaffoldingBlock::BOTTOM)
            && context.is_above(&SHAPE_BELOW_BLOCK, pos, true)
        {
            SHAPE_UNSTABLE_BOTTOM.clone()
        } else {
            Shapes::empty()
        }
    }

    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

impl ScaffoldingBlock {
    fn get_interaction_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> VoxelShape {
        Shapes::block()
    }
}
