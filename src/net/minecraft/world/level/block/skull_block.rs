//! net/minecraft/world/level/block/SkullBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour. Its constructor's `type` is its parent's.
//!
//! Of its behaviour, its shapes (a head, a piglin's wider, a dragon's taller to see) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::abstract_skull_block::{self, AbstractSkullBlock};
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
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

pub struct SkullBlock;

/// `SkullBlock.Types`: the kinds of heads, each its own `Type` in Java.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Types {
    SKELETON,
    WITHER_SKELETON,
    PLAYER,
    ZOMBIE,
    CREEPER,
    PIGLIN,
    DRAGON,
}

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(SkullBlock::get_shape),
    get_collision_shape: Some(SkullBlock::get_collision_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SkullBlock",
        &abstract_skull_block::CLASS,
    )
};

impl SkullBlock {
    pub const ROTATION: &'static IntegerProperty = BlockStateProperties::ROTATION_16;

    pub fn new(skull_type: Types, properties: Properties) -> BlockData {
        SkullBlock::new_with(
            skull_type,
            properties,
            &CLASS,
            SkullBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        skull_type: Types,
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = AbstractSkullBlock::new_with(skull_type, properties, class, definition);
        block.register_default_state(
            block
                .default_block_state()
                .set_value(SkullBlock::ROTATION, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        AbstractSkullBlock::create_block_state_definition(builder);
        builder.add(&[SkullBlock::ROTATION]);
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(8.0, 0.0, 8.0));
static SHAPE_PIGLIN: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(10.0, 0.0, 8.0));
static SHAPE_DRAGON_OUTLINE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(8.0, 0.0, 8.5));

impl SkullBlock {
    fn get_shape(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        context: &dyn CollisionContext,
    ) -> VoxelShape {
        if AbstractSkullBlock::get_type(block) == Types::DRAGON {
            SHAPE_DRAGON_OUTLINE.clone()
        } else {
            state.get_collision_shape_in(level, pos, context)
        }
    }

    fn get_collision_shape(
        block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        if AbstractSkullBlock::get_type(block) == Types::PIGLIN {
            SHAPE_PIGLIN.clone()
        } else {
            SHAPE.clone()
        }
    }
}
