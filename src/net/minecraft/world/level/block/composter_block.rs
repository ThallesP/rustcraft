//! net/minecraft/world/level/block/ComposterBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shapes (a box hollowed down to its fill level, empty to collide with, and
//! whole to interact with) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct ComposterBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(ComposterBlock::get_shape),
    get_interaction_shape: Some(ComposterBlock::get_interaction_shape),
    get_collision_shape: Some(ComposterBlock::get_collision_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.ComposterBlock",
        &block::CLASS,
    )
};

impl ComposterBlock {
    pub const LEVEL: &'static IntegerProperty = BlockStateProperties::LEVEL_COMPOSTER;

    pub fn new(properties: Properties) -> BlockData {
        ComposterBlock::new_with(
            properties,
            &CLASS,
            ComposterBlock::create_block_state_definition,
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
                .set_value(ComposterBlock::LEVEL, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[ComposterBlock::LEVEL]);
    }
}

static SHAPES: LazyLock<Vec<VoxelShape>> = LazyLock::new(|| {
    let mut shapes = Block::boxes(8, |level| {
        Shapes::join(
            &Shapes::block(),
            &Block::column(12.0, f64::from((1 + level * 2).clamp(2, 16)), 16.0),
            BooleanOp::ONLY_FIRST,
        )
    });
    shapes[8] = shapes[7].clone();
    shapes
});

impl ComposterBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPES[state.get_value(ComposterBlock::LEVEL) as usize].clone()
    }

    fn get_interaction_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> VoxelShape {
        Shapes::block()
    }

    fn get_collision_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPES[0].clone()
    }
}
