//! net/minecraft/world/level/block/ShulkerBoxBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `color`.
//!
//! Of its behaviour, `propagatesSkylightDown` and its shapes are ported as for a level without its
//! block entity: the whole block, which is what a closed box's lid gives. Block entities aren't
//! kept yet, so an open box's lid and its support from the opened side aren't.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::DirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct ShulkerBoxBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(ShulkerBoxBlock::get_shape),
    get_block_support_shape: Some(ShulkerBoxBlock::get_block_support_shape),
    propagates_skylight_down: Some(ShulkerBoxBlock::propagates_skylight_down),
    ..BlockClass::new(
        "net.minecraft.world.level.block.ShulkerBoxBlock",
        &base_entity_block::CLASS,
    )
};

impl ShulkerBoxBlock {
    pub const FACING: &'static EnumProperty<Direction> = DirectionalBlock::FACING;

    pub fn new(properties: Properties) -> BlockData {
        ShulkerBoxBlock::new_with(
            properties,
            &CLASS,
            ShulkerBoxBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseEntityBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(ShulkerBoxBlock::FACING, Direction::UP),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[ShulkerBoxBlock::FACING]);
    }
}

impl ShulkerBoxBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        false
    }
}

impl ShulkerBoxBlock {
    /// `getBlockSupportShape`: the whole block but for an open box's block entity (not kept yet).
    fn get_block_support_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> VoxelShape {
        Shapes::block()
    }

    /// `getShape`: the block entity's box, as far as its lid is open, or the whole block without
    /// one (block entities aren't kept yet).
    fn get_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::block()
    }
}
