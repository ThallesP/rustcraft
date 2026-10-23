//! net/minecraft/world/level/block/BedBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `color`.
//!
//! Of its behaviour, its shape (the mattress on two legs at the end away from the other half) is
//! ported.

use crate::com::mojang::math::octahedral_group::OctahedralGroup;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::abstract_bed_block::{self, AbstractBedBlock};
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct BedBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(BedBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BedBlock",
        &abstract_bed_block::CLASS,
    )
};

impl BedBlock {
    pub fn new(properties: Properties) -> BlockData {
        BedBlock::new_with(
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

static SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> = LazyLock::new(|| {
    let north_west_leg = Block::r#box(0.0, 0.0, 0.0, 3.0, 3.0, 3.0);
    let north_east_leg = Shapes::rotate(&north_west_leg, OctahedralGroup::BLOCK_ROT_Y_90);
    Shapes::rotate_horizontal(&Shapes::or_tail(
        &Block::column(16.0, 3.0, 9.0),
        &[north_west_leg, north_east_leg],
    ))
});

impl BedBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(
            &SHAPES,
            AbstractBedBlock::get_connected_direction(state).get_opposite(),
        )
        .clone()
    }
}
