//! net/minecraft/world/level/block/StrawBedBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, its shape (a low base, with a pillow at the head) is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::abstract_bed_block::{self, AbstractBedBlock};
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::bed_part::BedPart;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct StrawBedBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(StrawBedBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.StrawBedBlock",
        &abstract_bed_block::CLASS,
    )
};

impl StrawBedBlock {
    pub fn new(properties: Properties) -> BlockData {
        StrawBedBlock::new_with(
            properties,
            &CLASS,
            AbstractBedBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        AbstractBedBlock::new_with(properties, class, definition)
    }
}

static BASE_SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 4.0));
static PILLOW_SHAPE: LazyLock<VoxelShape> =
    LazyLock::new(|| Block::r#box(0.0, 0.0, 0.0, 16.0, 5.0, 8.0));
static FOOT_SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal(&BASE_SHAPE));
static HEAD_SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal(&Shapes::or(&BASE_SHAPE, &PILLOW_SHAPE)));

impl StrawBedBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        let shapes = if state.get_value(AbstractBedBlock::PART) == BedPart::HEAD {
            &HEAD_SHAPES
        } else {
            &FOOT_SHAPES
        };
        Shapes::get(
            shapes,
            AbstractBedBlock::get_connected_direction(state).get_opposite(),
        )
        .clone()
    }
}
