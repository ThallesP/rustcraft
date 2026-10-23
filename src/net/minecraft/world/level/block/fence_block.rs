//! net/minecraft/world/level/block/FenceBlock.java
//!
//! What decides its blocks' states (its properties and its default state), the sizes its
//! parent's shapes are made with (4, 16, 4, 16 and 24 sixteenths), and its occlusion shapes (a
//! post 2 sixteenths wide, with arms from 6 to 15 sixteenths up), which the light engine and face
//! culling read. Not ported yet: the rest of its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::cross_collision_block::{
    self, CrossCollisionBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct FenceBlock;

pub static CLASS: BlockClass = BlockClass {
    get_occlusion_shape: Some(FenceBlock::get_occlusion_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.FenceBlock",
        &cross_collision_block::CLASS,
    )
};

impl FenceBlock {
    pub fn new(properties: Properties) -> BlockData {
        FenceBlock::new_with(
            properties,
            &CLASS,
            FenceBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = CrossCollisionBlock::new_with(
            4.0, 16.0, 4.0, 16.0, 24.0, properties, class, definition,
        );
        let occlusion_shapes = CrossCollisionBlock::make_shapes(&block, 4.0, 16.0, 2.0, 6.0, 15.0);
        let mut fields = block.fields::<CrossCollisionBlock>().clone();
        fields.occlusion_shapes = Some(occlusion_shapes);
        block.set_fields(fields);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(CrossCollisionBlock::NORTH, false)
                .set_value(CrossCollisionBlock::EAST, false)
                .set_value(CrossCollisionBlock::SOUTH, false)
                .set_value(CrossCollisionBlock::WEST, false)
                .set_value(CrossCollisionBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            CrossCollisionBlock::NORTH,
            CrossCollisionBlock::EAST,
            CrossCollisionBlock::WEST,
            CrossCollisionBlock::SOUTH,
            CrossCollisionBlock::WATERLOGGED,
        ]);
    }
}

impl FenceBlock {
    /// `getOcclusionShape`: the state's of `occlusionShapes`.
    fn get_occlusion_shape(block: &BlockData, state: BlockState) -> VoxelShape {
        block
            .fields::<CrossCollisionBlock>()
            .occlusion_shapes
            .as_ref()
            .expect("a fence makes its occlusion shapes")[state.index() as usize]
            .clone()
    }
}
