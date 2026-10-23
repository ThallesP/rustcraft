//! net/minecraft/world/level/block/LightBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `propagatesSkylightDown`, its shape (the whole block to whoever holds a light,
//! nothing to anyone else), its render shape (invisible) and its shade brightness are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::references::block_item_ids::BlockItemIds;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_behaviour::RenderShape;
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

pub struct LightBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(LightBlock::get_shape),
    get_render_shape: Some(LightBlock::get_render_shape),
    get_shade_brightness: Some(LightBlock::get_shade_brightness),
    propagates_skylight_down: Some(LightBlock::propagates_skylight_down),
    get_fluid_state: Some(LightBlock::get_fluid_state),
    ..BlockClass::new("net.minecraft.world.level.block.LightBlock", &block::CLASS)
};

impl LightBlock {
    pub const LEVEL: &'static IntegerProperty = BlockStateProperties::LEVEL;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        LightBlock::new_with(
            properties,
            &CLASS,
            LightBlock::create_block_state_definition,
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
                .set_value(LightBlock::LEVEL, 15)
                .set_value(LightBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[LightBlock::LEVEL, LightBlock::WATERLOGGED]);
    }

    /// `LIGHT_EMISSION`: its level.
    pub fn light_emission(state: BlockState) -> i32 {
        state.get_value(LightBlock::LEVEL)
    }
}

impl LightBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

impl LightBlock {
    /// `propagatesSkylightDown`: without a fluid.
    fn propagates_skylight_down(block: &BlockData, state: BlockState) -> bool {
        block.get_fluid_state_of(state).is_empty()
    }
}

impl LightBlock {
    fn get_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        context: &dyn CollisionContext,
    ) -> VoxelShape {
        if context.is_holding_item(BlockItemIds::LIGHT.item()) {
            Shapes::block()
        } else {
            Shapes::empty()
        }
    }

    fn get_render_shape(_block: &BlockData, _state: BlockState) -> RenderShape {
        RenderShape::INVISIBLE
    }

    fn get_shade_brightness(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> f32 {
        1.0
    }
}
