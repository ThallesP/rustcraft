//! net/minecraft/world/level/block/TurtleEggBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shape (one egg, or a clutch) is ported.

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
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct TurtleEggBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(TurtleEggBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.TurtleEggBlock",
        &block::CLASS,
    )
};

impl TurtleEggBlock {
    pub const HATCH: &'static IntegerProperty = BlockStateProperties::HATCH;
    pub const EGGS: &'static IntegerProperty = BlockStateProperties::EGGS;

    pub fn new(properties: Properties) -> BlockData {
        TurtleEggBlock::new_with(
            properties,
            &CLASS,
            TurtleEggBlock::create_block_state_definition,
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
                .set_value(TurtleEggBlock::HATCH, 0)
                .set_value(TurtleEggBlock::EGGS, 1),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[TurtleEggBlock::HATCH, TurtleEggBlock::EGGS]);
    }
}

static SHAPE_SINGLE: LazyLock<VoxelShape> =
    LazyLock::new(|| Block::r#box(3.0, 0.0, 3.0, 12.0, 7.0, 12.0));
static SHAPE_MULTIPLE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(14.0, 0.0, 7.0));

impl TurtleEggBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        if state.get_value(TurtleEggBlock::EGGS) == 1 {
            SHAPE_SINGLE.clone()
        } else {
            SHAPE_MULTIPLE.clone()
        }
    }
}
