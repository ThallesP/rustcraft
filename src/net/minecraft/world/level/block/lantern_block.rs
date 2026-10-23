//! net/minecraft/world/level/block/LanternBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `getFluidState` and its shape (a sixteenth higher when it hangs) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct LanternBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(LanternBlock::get_shape),
    get_fluid_state: Some(LanternBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.LanternBlock",
        &block::CLASS,
    )
};

impl LanternBlock {
    pub const HANGING: &'static BooleanProperty = BlockStateProperties::HANGING;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        LanternBlock::new_with(
            properties,
            &CLASS,
            LanternBlock::create_block_state_definition,
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
                .set_value(LanternBlock::HANGING, false)
                .set_value(LanternBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[LanternBlock::HANGING, LanternBlock::WATERLOGGED]);
    }
}

impl LanternBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

static SHAPE_STANDING: LazyLock<VoxelShape> =
    LazyLock::new(|| Shapes::or(&Block::column(4.0, 7.0, 9.0), &Block::column(6.0, 0.0, 7.0)));
static SHAPE_HANGING: LazyLock<VoxelShape> =
    LazyLock::new(|| SHAPE_STANDING.move_xyz(0.0, 0.0625, 0.0).optimize());

impl LanternBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        if state.get_value(LanternBlock::HANGING) {
            SHAPE_HANGING.clone()
        } else {
            SHAPE_STANDING.clone()
        }
    }
}
