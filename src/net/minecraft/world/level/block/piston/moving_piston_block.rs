//! net/minecraft/world/level/block/piston/MovingPistonBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shapes and render shape (invisible: its block entity draws what moves) are
//! ported, with its collision shape as for a level without its block entity, nothing: block
//! entities aren't kept yet, so a moving block's collision isn't.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::DirectionalBlock;
use crate::net::minecraft::world::level::block::piston::piston_head_block::PistonHeadBlock;
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

    /// `getCollisionShape`: its block entity's, or nothing without one (block entities aren't
    /// kept yet).
    fn get_collision_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::empty()
    }

    fn get_render_shape(_block: &BlockData, _state: BlockState) -> RenderShape {
        RenderShape::INVISIBLE
    }
}
