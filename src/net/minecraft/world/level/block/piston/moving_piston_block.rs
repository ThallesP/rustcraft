//! net/minecraft/world/level/block/piston/MovingPistonBlock.java
//!
//! Its properties and default state, its shapes (its collision shape its block entity's, what it
//! moves where it is so far, nothing without one), its render shape (invisible: its block entity
//! draws what moves) and its block entity's ticker, on both sides; `newBlockEntity` makes none,
//! a piston's block event makes it (`PistonMovingBlockEntity::new_moving`, Java's
//! `newMovingBlockEntity`). Not ported yet: `destroy`, `useWithoutItem` and its drops, the
//! server's.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::DirectionalBlock;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntityData;
use crate::net::minecraft::world::level::block::entity::block_entity_ticker::BlockEntityTicker;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::entity::block_entity_types::BlockEntityTypes;
use crate::net::minecraft::world::level::block::piston::piston_head_block::PistonHeadBlock;
use crate::net::minecraft::world::level::block::piston::piston_moving_block_entity::PistonMovingBlockEntity;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_behaviour::RenderShape;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::piston_type::PistonType;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct MovingPistonBlock;

pub static CLASS: BlockClass = BlockClass {
    get_ticker: Some(MovingPistonBlock::get_ticker),
    get_shape: Some(MovingPistonBlock::get_shape),
    get_collision_shape: Some(MovingPistonBlock::get_collision_shape),
    get_render_shape: Some(MovingPistonBlock::get_render_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.piston.MovingPistonBlock",
        &base_entity_block::CLASS,
    )
};

impl MovingPistonBlock {
    pub const FACING: &'static EnumProperty<Direction> = DirectionalBlock::FACING;
    pub const TYPE: &'static EnumProperty<PistonType> = PistonHeadBlock::TYPE;

    pub fn new(properties: Properties) -> BlockData {
        MovingPistonBlock::new_with(
            properties,
            &CLASS,
            MovingPistonBlock::create_block_state_definition,
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
                .set_value(MovingPistonBlock::FACING, Direction::NORTH)
                .set_value(MovingPistonBlock::TYPE, PistonType::DEFAULT),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[MovingPistonBlock::FACING, MovingPistonBlock::TYPE]);
    }
}

impl MovingPistonBlock {
    fn get_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::empty()
    }

    /// `getCollisionShape`: its block entity's, or nothing without one.
    fn get_collision_shape(
        _block: &BlockData,
        _state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        let entity = match level
            .get_block_entity(pos)
            .as_deref()
            .map(|entity| &entity.data)
        {
            Some(BlockEntityData::Piston(entity)) => entity.clone(),
            _ => return Shapes::empty(),
        };
        entity.get_collision_shape(level, pos)
    }

    /// `getTicker`: the moving block's `tick`, on both sides.
    fn get_ticker(
        _block: &BlockData,
        _state: BlockState,
        r#type: BlockEntityType,
    ) -> Option<BlockEntityTicker> {
        base_entity_block::create_ticker_helper(
            r#type,
            BlockEntityTypes::PISTON,
            PistonMovingBlockEntity::tick,
        )
    }

    fn get_render_shape(_block: &BlockData, _state: BlockState) -> RenderShape {
        RenderShape::INVISIBLE
    }
}
