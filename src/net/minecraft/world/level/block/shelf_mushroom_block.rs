//! net/minecraft/world/level/block/ShelfMushroomBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, its shape (two shelves out from the log it faces from, bigger when it's grown)
//! is ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::{
    self, HorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct ShelfMushroomBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(ShelfMushroomBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.ShelfMushroomBlock",
        &horizontal_directional_block::CLASS,
    )
};

impl ShelfMushroomBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_1;

    pub fn new(properties: Properties) -> BlockData {
        ShelfMushroomBlock::new_with(
            properties,
            &CLASS,
            ShelfMushroomBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = HorizontalDirectionalBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(HorizontalDirectionalBlock::FACING, Direction::NORTH)
                .set_value(ShelfMushroomBlock::AGE, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[HorizontalDirectionalBlock::FACING, ShelfMushroomBlock::AGE]);
    }
}

static SHAPES: LazyLock<Vec<Vec<(Direction, VoxelShape)>>> = LazyLock::new(|| {
    (0..=1)
        .map(|i| {
            let i_f = f64::from(i);
            Shapes::rotate_horizontal(&Shapes::or(
                &Block::column_xz(
                    f64::from(10 + i * 4),
                    f64::from(7 + i * 3),
                    f64::from(1 + i),
                    f64::from(3 + i * 2),
                )
                .move_xyz(0.0, f64::from(8 - i * 2) / 16.0, -(i_f * 1.5 - 4.5) / 16.0)
                .optimize(),
                &Block::column_xz(
                    f64::from(6 + i * 2),
                    f64::from(4 + i * 2),
                    0.0,
                    f64::from(1 + i),
                )
                .move_xyz(0.0, f64::from(8 - i * 2) / 16.0, f64::from(-(i - 6)) / 16.0)
                .optimize(),
            ))
        })
        .collect()
});

impl ShelfMushroomBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        Shapes::get(
            &SHAPES[state.get_value(ShelfMushroomBlock::AGE) as usize],
            state.get_value(HorizontalDirectionalBlock::FACING),
        )
        .clone()
    }
}
