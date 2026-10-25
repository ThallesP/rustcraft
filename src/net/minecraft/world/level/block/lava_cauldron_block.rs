//! net/minecraft/world/level/block/LavaCauldronBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Of its behaviour, what an entity
//! has to touch to be in it (the cauldron, filled 15 sixteenths up) is ported. Not ported yet:
//! the rest of its behaviour (what it does to an entity inside is the entity's fire and the
//! server's damage), and its constructor's `interactionMap`.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::entity::entity::EntityClass;
use crate::net::minecraft::world::level::block::abstract_cauldron_block::{
    self, AbstractCauldronBlock,
};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct LavaCauldronBlock;

pub static CLASS: BlockClass = BlockClass {
    get_entity_inside_collision_shape: Some(LavaCauldronBlock::get_entity_inside_collision_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.LavaCauldronBlock",
        &abstract_cauldron_block::CLASS,
    )
};

impl LavaCauldronBlock {
    pub fn new(properties: Properties) -> BlockData {
        LavaCauldronBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        AbstractCauldronBlock::new_with(properties, class, definition)
    }
}

static SHAPE_INSIDE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(12.0, 4.0, 15.0));
static FILLED_SHAPE: LazyLock<VoxelShape> =
    LazyLock::new(|| Shapes::or(&abstract_cauldron_block::SHAPE, &SHAPE_INSIDE));

impl LavaCauldronBlock {
    /// `getEntityInsideCollisionShape`: the cauldron, filled.
    fn get_entity_inside_collision_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _entity: &dyn EntityClass,
    ) -> VoxelShape {
        FILLED_SHAPE.clone()
    }
}
