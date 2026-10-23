//! net/minecraft/world/level/block/PitcherCropBlock.java
//!
//! Only what decides its blocks' states so far: its properties, its default state and which of its
//! states tick. Not ported yet: its behaviour.
//!
//! Of its behaviour, which states tick and its shapes (a bulb, then a growing plant across both
//! halves; the lower part to collide with) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::double_plant_block::{self, DoublePlantBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::double_block_half::DoubleBlockHalf;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct PitcherCropBlock {
    /// `shapes`, by state index.
    shapes: Vec<VoxelShape>,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(PitcherCropBlock::get_shape),
    get_collision_shape: Some(PitcherCropBlock::get_collision_shape),
    is_randomly_ticking: Some(PitcherCropBlock::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.PitcherCropBlock",
        &double_plant_block::CLASS,
    )
};

impl PitcherCropBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_4;
    pub const HALF: &'static EnumProperty<DoubleBlockHalf> = DoublePlantBlock::HALF;

    pub fn new(properties: Properties) -> BlockData {
        PitcherCropBlock::new_with(
            properties,
            &CLASS,
            PitcherCropBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = DoublePlantBlock::new_with(properties, class, definition);
        let shapes = PitcherCropBlock::make_shapes(&block);
        block.set_fields(PitcherCropBlock { shapes });
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[PitcherCropBlock::AGE]);
        DoublePlantBlock::create_block_state_definition(builder);
    }

    /// The lower half ticks until it's grown.
    fn is_randomly_ticking(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(PitcherCropBlock::HALF) == DoubleBlockHalf::LOWER
            && !PitcherCropBlock::is_max_age(state)
    }

    fn is_max_age(state: BlockState) -> bool {
        state.get_value(PitcherCropBlock::AGE) >= 4
    }
}

static SHAPE_BULB: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(6.0, -1.0, 3.0));
static SHAPE_CROP: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(10.0, -1.0, 5.0));

impl PitcherCropBlock {
    /// `makeShapes`: as tall as the age's plant across both halves, a sixteenth into the
    /// ground.
    fn make_shapes(block: &BlockData) -> Vec<VoxelShape> {
        let plant_heights = [0, 9, 11, 22, 26];
        Block::get_shape_for_each_state(block, |state| {
            let age = state.get_value(PitcherCropBlock::AGE);
            let height = if age == 0 { 4 } else { 6 } + plant_heights[age as usize];
            let width = if age == 0 { 6 } else { 10 };
            match state.get_value(PitcherCropBlock::HALF) {
                DoubleBlockHalf::LOWER => {
                    Block::column(f64::from(width), -1.0, f64::from(16.min(-1 + height)))
                }
                DoubleBlockHalf::UPPER => {
                    Block::column(f64::from(width), 0.0, f64::from(0.max(-1 + height - 16)))
                }
            }
        })
    }

    fn get_shape(
        block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<PitcherCropBlock>().shapes[state.index() as usize].clone()
    }

    fn get_collision_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        if state.get_value(PitcherCropBlock::HALF) == DoubleBlockHalf::LOWER {
            if state.get_value(PitcherCropBlock::AGE) == 0 {
                SHAPE_BULB.clone()
            } else {
                SHAPE_CROP.clone()
            }
        } else {
            Shapes::empty()
        }
    }
}
