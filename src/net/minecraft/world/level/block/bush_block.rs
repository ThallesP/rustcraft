//! net/minecraft/world/level/block/BushBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, its shape (13 sixteenths tall, `DEFAULT_SHAPE_HEIGHT`; nothing makes one with
//! another height) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct BushBlock {
    shape: VoxelShape,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(BushBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BushBlock",
        &vegetation_block::CLASS,
    )
};

impl BushBlock {
    pub fn new(properties: Properties) -> BlockData {
        BushBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = VegetationBlock::new_with(properties, class, definition);
        block.set_fields(BushBlock {
            shape: Block::column(16.0, 0.0, f64::from(BushBlock::DEFAULT_SHAPE_HEIGHT)),
        });
        block
    }
}

impl BushBlock {
    pub const DEFAULT_SHAPE_HEIGHT: i32 = 13;

    fn get_shape(
        block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        block.fields::<BushBlock>().shape.clone()
    }
}
