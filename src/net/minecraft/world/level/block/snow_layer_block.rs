//! net/minecraft/world/level/block/SnowLayerBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `useShapeForLightOcclusion`, its shapes (two sixteenths a layer, and a layer
//! less to collide with) and its shade brightness (a full block's when it's eight layers) are
//! ported.

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

pub struct SnowLayerBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(SnowLayerBlock::get_shape),
    get_collision_shape: Some(SnowLayerBlock::get_collision_shape),
    get_block_support_shape: Some(SnowLayerBlock::get_block_support_shape),
    get_visual_shape: Some(SnowLayerBlock::get_shape),
    get_shade_brightness: Some(SnowLayerBlock::get_shade_brightness),
    use_shape_for_light_occlusion: Some(SnowLayerBlock::use_shape_for_light_occlusion),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SnowLayerBlock",
        &block::CLASS,
    )
};

impl SnowLayerBlock {
    pub const LAYERS: &'static IntegerProperty = BlockStateProperties::LAYERS;

    pub fn new(properties: Properties) -> BlockData {
        SnowLayerBlock::new_with(
            properties,
            &CLASS,
            SnowLayerBlock::create_block_state_definition,
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
                .set_value(SnowLayerBlock::LAYERS, 1),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[SnowLayerBlock::LAYERS]);
    }
}

impl SnowLayerBlock {
    /// `useShapeForLightOcclusion`.
    fn use_shape_for_light_occlusion(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}

static SHAPES: LazyLock<Vec<VoxelShape>> =
    LazyLock::new(|| Block::boxes(8, |height| Block::column(16.0, 0.0, f64::from(height * 2))));

impl SnowLayerBlock {
    /// `getShape`, and `getVisualShape`, which is the same.
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPES[state.get_value(SnowLayerBlock::LAYERS) as usize].clone()
    }

    fn get_collision_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPES[(state.get_value(SnowLayerBlock::LAYERS) - 1) as usize].clone()
    }

    fn get_block_support_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> VoxelShape {
        SHAPES[state.get_value(SnowLayerBlock::LAYERS) as usize].clone()
    }

    fn get_shade_brightness(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> f32 {
        if state.get_value(SnowLayerBlock::LAYERS) == 8 {
            0.2
        } else {
            1.0
        }
    }
}
