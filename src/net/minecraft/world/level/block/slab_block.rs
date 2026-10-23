//! net/minecraft/world/level/block/SlabBlock.java
//!
//! What decides its blocks' states (its properties and its default state) and its shapes: a half
//! block at the bottom or the top, or a whole one. Not ported yet: the rest of its behaviour.

use std::sync::LazyLock;

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::slab_type::SlabType;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

static SHAPE_BOTTOM: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 8.0));
static SHAPE_TOP: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 8.0, 16.0));

pub struct SlabBlock;

pub static CLASS: BlockClass = BlockClass {
    get_fluid_state: Some(SlabBlock::get_fluid_state),
    get_shape: Some(SlabBlock::get_shape),
    ..BlockClass::new("net.minecraft.world.level.block.SlabBlock", &block::CLASS)
};

impl SlabBlock {
    pub const TYPE: &'static EnumProperty<SlabType> = BlockStateProperties::SLAB_TYPE;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        SlabBlock::new_with(properties, &CLASS, SlabBlock::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(
            block
                .default_block_state()
                .set_value(SlabBlock::TYPE, SlabType::BOTTOM)
                .set_value(SlabBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[SlabBlock::TYPE, SlabBlock::WATERLOGGED]);
    }
}

impl SlabBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        match state.get_value(SlabBlock::TYPE) {
            SlabType::TOP => SHAPE_TOP.clone(),
            SlabType::BOTTOM => SHAPE_BOTTOM.clone(),
            SlabType::DOUBLE => Shapes::block(),
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
