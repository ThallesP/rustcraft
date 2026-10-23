//! net/minecraft/world/level/block/NetherPortalBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shape (a pane along its axis) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct NetherPortalBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(NetherPortalBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.NetherPortalBlock",
        &block::CLASS,
    )
};

impl NetherPortalBlock {
    pub const AXIS: &'static EnumProperty<Axis> = BlockStateProperties::HORIZONTAL_AXIS;

    pub fn new(properties: Properties) -> BlockData {
        NetherPortalBlock::new_with(
            properties,
            &CLASS,
            NetherPortalBlock::create_block_state_definition,
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
                .set_value(NetherPortalBlock::AXIS, Axis::X),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[NetherPortalBlock::AXIS]);
    }
}

static SHAPES: LazyLock<Vec<(Axis, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal_axis(&Block::column_xz(4.0, 16.0, 0.0, 16.0)));

impl NetherPortalBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(
            &SHAPES,
            state.get_value(BlockStateProperties::HORIZONTAL_AXIS),
        )
        .clone()
    }
}
